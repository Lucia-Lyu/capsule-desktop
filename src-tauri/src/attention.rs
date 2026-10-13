use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

use crate::{database::{Database, DbResult}, window};

const DISPLAY_SECONDS: i64 = 10;
const REPEAT_SECONDS: i64 = 90;
const GROUP_MINUTES: i64 = 5;

#[derive(Default)]
struct Cycle {
    first_at: Option<DateTime<Utc>>,
    visible_until: Option<DateTime<Utc>>,
    repeat_at: Option<DateTime<Utc>>,
    origin: Option<PhysicalPosition<i32>>,
}

impl Cycle {
    fn accepts_new_round(&self, now: DateTime<Utc>) -> bool {
        self.first_at.is_none_or(|first| now - first >= Duration::minutes(GROUP_MINUTES))
    }

    fn start(&mut self, now: DateTime<Utc>, repeat: bool) {
        self.first_at = Some(now);
        self.visible_until = Some(now + Duration::seconds(DISPLAY_SECONDS));
        self.repeat_at = repeat.then_some(now + Duration::seconds(REPEAT_SECONDS));
    }
}

struct AttentionState {
    cycle: Cycle,
    sound: bool,
    repeat: bool,
}

pub struct Attention {
    state: Mutex<AttentionState>,
}

fn capsule(app: &AppHandle) -> DbResult<WebviewWindow> {
    app.get_webview_window("main").ok_or_else(|| "Capsule window is missing".into())
}

#[cfg(target_os = "windows")]
fn play_system_chime() {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBeep(kind: u32) -> i32;
    }
    // The Windows information chime follows the user's system sound and volume settings.
    unsafe { MessageBeep(0x0000_0040); }
}

#[cfg(not(target_os = "windows"))]
fn play_system_chime() {}

impl Attention {
    pub fn new(database: &Database) -> DbResult<Self> {
        let (sound, repeat) = database.attention_settings()?;
        Ok(Self { state: Mutex::new(AttentionState {
            cycle: Cycle::default(), sound, repeat,
        }) })
    }

    fn show(app: &AppHandle, state: &mut AttentionState, refresh_origin: bool) -> DbResult<()> {
        let window = capsule(app)?;
        let original = window::center_attention(&window).map_err(|error| error.to_string())?;
        if refresh_origin || state.cycle.origin.is_none() { state.cycle.origin = Some(original); }
        app.emit("reminder://attention", true).map_err(|error| error.to_string())?;
        if state.sound { play_system_chime(); }
        Ok(())
    }

    fn hide(app: &AppHandle, state: &mut AttentionState) -> DbResult<()> {
        if state.cycle.visible_until.is_none() { return Ok(()); }
        if let Some(origin) = state.cycle.origin {
            window::restore_attention(&capsule(app)?, origin).map_err(|error| error.to_string())?;
        }
        state.cycle.visible_until = None;
        app.emit("reminder://attention", false).map_err(|error| error.to_string())
    }

    pub fn triggered(&self, app: &AppHandle, now: DateTime<Utc>) -> DbResult<()> {
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        if !state.cycle.accepts_new_round(now) { return Ok(()); }
        let refresh_origin = state.cycle.visible_until.is_none();
        let repeat = state.repeat;
        state.cycle.start(now, repeat);
        Self::show(app, &mut state, refresh_origin)
    }

    pub fn tick(&self, app: &AppHandle, now: DateTime<Utc>, active: bool, quiet: bool, reset: bool) -> DbResult<()> {
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        if quiet || reset || !active {
            Self::hide(app, &mut state)?;
            state.cycle = Cycle::default();
            return Ok(());
        }
        if state.cycle.visible_until.is_some_and(|until| now >= until) {
            Self::hide(app, &mut state)?;
        }
        if state.cycle.repeat_at.is_some_and(|at| now >= at) {
            state.cycle.repeat_at = None;
            state.cycle.visible_until = Some(now + Duration::seconds(DISPLAY_SECONDS));
            Self::show(app, &mut state, true)?;
        }
        Ok(())
    }

    pub fn dismiss(&self, app: &AppHandle) -> DbResult<bool> {
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        let visible = state.cycle.visible_until.is_some();
        Self::hide(app, &mut state)?;
        Ok(visible)
    }

    pub fn resolved(&self, app: &AppHandle, any_active: bool) -> DbResult<()> {
        if any_active { return Ok(()); }
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        Self::hide(app, &mut state)?;
        state.cycle = Cycle::default();
        Ok(())
    }

    pub fn update_settings(&self, sound: bool, repeat: bool) -> DbResult<()> {
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        state.sound = sound;
        state.repeat = repeat;
        if !repeat { state.cycle.repeat_at = None; }
        Ok(())
    }

    pub fn visible(&self) -> DbResult<bool> {
        let state = self.state.lock().map_err(|error| error.to_string())?;
        Ok(state.cycle.visible_until.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attention_cycle_has_ten_second_card_and_one_ninety_second_repeat() {
        let now = Utc::now();
        let mut cycle = Cycle::default();
        assert!(cycle.accepts_new_round(now));
        cycle.start(now, true);
        assert_eq!(cycle.visible_until, Some(now + Duration::seconds(10)));
        assert_eq!(cycle.repeat_at, Some(now + Duration::seconds(90)));
        assert!(!cycle.accepts_new_round(now + Duration::minutes(4)));
        assert!(cycle.accepts_new_round(now + Duration::minutes(5)));
        cycle.start(now, false);
        assert_eq!(cycle.repeat_at, None);
    }
}
