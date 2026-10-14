use std::sync::Mutex;

use chrono::{DateTime, Duration, Utc};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

use crate::{database::{Database, DbResult}, window};

const DISPLAY_SECONDS: i64 = 10;
const REPEAT_SECONDS: i64 = 90;

#[derive(Clone)]
struct Notice {
    event_ids: Vec<String>,
    due_at: DateTime<Utc>,
    first_at: DateTime<Utc>,
    is_repeat: bool,
}

#[derive(Default)]
struct Cycle {
    current: Option<Notice>,
    pending: Vec<Notice>,
    visible_until: Option<DateTime<Utc>>,
    origin: Option<PhysicalPosition<i32>>,
}

impl Cycle {
    fn take_ready(&mut self, now: DateTime<Utc>) -> Option<Notice> {
        let index = self.pending.iter().enumerate()
            .filter(|(_, notice)| notice.due_at <= now)
            .min_by_key(|(_, notice)| notice.due_at)
            .map(|(index, _)| index)?;
        Some(self.pending.remove(index))
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

    fn show_next(app: &AppHandle, state: &mut AttentionState, now: DateTime<Utc>) -> DbResult<()> {
        if state.cycle.current.is_some() { return Ok(()); }
        let Some(notice) = state.cycle.take_ready(now) else { return Ok(()); };
        let window = match capsule(app) {
            Ok(window) => window,
            Err(error) => {
                state.cycle.pending.push(notice);
                return Err(error);
            }
        };
        let original = match window::center_attention(&window, notice.event_ids.len()) {
            Ok(original) => original,
            Err(error) => {
                state.cycle.pending.push(notice);
                return Err(error.to_string());
            }
        };
        state.cycle.origin = Some(original);
        state.cycle.visible_until = Some(now + Duration::seconds(DISPLAY_SECONDS));
        state.cycle.current = Some(notice);
        app.emit("reminder://attention", true).map_err(|error| error.to_string())?;
        if state.sound { play_system_chime(); }
        Ok(())
    }

    fn hide(app: &AppHandle, state: &mut AttentionState, schedule_repeat: bool) -> DbResult<()> {
        let Some(notice) = state.cycle.current.as_ref() else { return Ok(()); };
        if let Some(origin) = state.cycle.origin {
            window::restore_attention(&capsule(app)?, origin).map_err(|error| error.to_string())?;
        }
        let notice = notice.clone();
        state.cycle.current = None;
        state.cycle.origin = None;
        state.cycle.visible_until = None;
        if schedule_repeat && state.repeat && !notice.is_repeat {
            state.cycle.pending.push(Notice {
                event_ids: notice.event_ids.clone(),
                due_at: notice.first_at + Duration::seconds(REPEAT_SECONDS),
                first_at: notice.first_at,
                is_repeat: true,
            });
        }
        app.emit("reminder://attention", false).map_err(|error| error.to_string())
    }

    pub fn triggered(&self, app: &AppHandle, now: DateTime<Utc>, event_ids: Vec<String>) -> DbResult<()> {
        if event_ids.is_empty() { return Ok(()); }
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        state.cycle.pending.push(Notice { event_ids, due_at: now, first_at: now, is_repeat: false });
        Self::show_next(app, &mut state, now)
    }

    pub fn tick(&self, app: &AppHandle, now: DateTime<Utc>, active: bool, quiet: bool, reset: bool) -> DbResult<()> {
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        if quiet || reset || !active {
            Self::hide(app, &mut state, false)?;
            state.cycle.pending.clear();
            return Ok(());
        }
        if state.cycle.visible_until.is_some_and(|until| now >= until) {
            Self::hide(app, &mut state, true)?;
        }
        Self::show_next(app, &mut state, now)
    }

    pub fn dismiss(&self, app: &AppHandle) -> DbResult<bool> {
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        let visible = state.cycle.current.is_some();
        Self::hide(app, &mut state, true)?;
        Ok(visible)
    }

    pub fn resolved(&self, app: &AppHandle, active_ids: &[String]) -> DbResult<()> {
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        state.cycle.pending.retain_mut(|notice| {
            notice.event_ids.retain(|id| active_ids.contains(id));
            !notice.event_ids.is_empty()
        });
        if let Some(current) = state.cycle.current.as_mut() {
            current.event_ids.retain(|id| active_ids.contains(id));
            if current.event_ids.is_empty() { Self::hide(app, &mut state, false)?; }
        }
        Self::show_next(app, &mut state, Utc::now())
    }

    pub fn update_settings(&self, sound: bool, repeat: bool) -> DbResult<()> {
        let mut state = self.state.lock().map_err(|error| error.to_string())?;
        state.sound = sound;
        state.repeat = repeat;
        if !repeat { state.cycle.pending.retain(|notice| !notice.is_repeat); }
        Ok(())
    }

    pub fn visible_ids(&self) -> DbResult<Vec<String>> {
        let state = self.state.lock().map_err(|error| error.to_string())?;
        Ok(state.cycle.current.as_ref().map(|notice| notice.event_ids.clone()).unwrap_or_default())
    }

    pub fn visible(&self) -> DbResult<bool> {
        let state = self.state.lock().map_err(|error| error.to_string())?;
        Ok(state.cycle.current.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_notices_keep_water_separate_from_a_paired_break() {
        let now = Utc::now();
        let mut cycle = Cycle::default();
        cycle.pending.push(Notice { event_ids: vec!["eye".into(), "stand".into()], due_at: now, first_at: now, is_repeat: false });
        cycle.pending.push(Notice { event_ids: vec!["water".into()], due_at: now, first_at: now, is_repeat: false });
        cycle.pending.push(Notice { event_ids: vec!["repeat".into()], due_at: now + Duration::seconds(90), first_at: now, is_repeat: true });
        assert_eq!(cycle.take_ready(now).unwrap().event_ids, ["eye", "stand"]);
        assert_eq!(cycle.take_ready(now).unwrap().event_ids, ["water"]);
        assert!(cycle.take_ready(now).is_none());
        assert_eq!(cycle.take_ready(now + Duration::seconds(90)).unwrap().event_ids, ["repeat"]);
    }
}
