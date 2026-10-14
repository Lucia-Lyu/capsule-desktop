use std::{collections::HashMap, sync::Mutex, thread, time::Duration as StdDuration};

use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime, Utc};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    attention::Attention,
    database::{Database, DbResult},
    models::{NextReminder, ReminderRule, ReminderSnapshot},
};

const MERGE_MINUTES: i64 = 10;

struct RuleTiming {
    next_at: Option<DateTime<Utc>>,
    active_event_id: Option<String>,
    snoozed_event_id: Option<String>,
}

struct Engine {
    rules: Vec<ReminderRule>,
    timing: HashMap<String, RuleTiming>,
    quiet_start: NaiveTime,
    quiet_end: NaiveTime,
    last_tick: DateTime<Utc>,
    last_day: NaiveDate,
    was_quiet: bool,
}

pub struct Scheduler {
    engine: Mutex<Engine>,
}

fn quiet_at(time: NaiveTime, start: NaiveTime, end: NaiveTime) -> bool {
    if start == end { return false; }
    if start < end { time >= start && time < end } else { time >= start || time < end }
}

fn next_after(now: DateTime<Utc>, minutes: i64) -> DateTime<Utc> {
    now + Duration::minutes(minutes)
}

fn resumed_after_sleep(last: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now - last > Duration::minutes(2) || now < last
}

fn merge_deadline(
    eye: Option<DateTime<Utc>>,
    stand: Option<DateTime<Utc>>,
    quiet_start: NaiveTime,
    quiet_end: NaiveTime,
) -> Option<DateTime<Utc>> {
    let (eye, stand) = (eye?, stand?);
    let (earlier, later) = if eye <= stand { (eye, stand) } else { (stand, eye) };
    let first_local = earlier.with_timezone(&Local);
    let last_local = later.with_timezone(&Local);
    if later - earlier > Duration::minutes(MERGE_MINUTES)
        || first_local.date_naive() != last_local.date_naive()
        || quiet_at(last_local.time(), quiet_start, quiet_end)
    {
        return None;
    }
    Some(later)
}

impl Engine {
    fn pair_deadline(&self) -> Option<DateTime<Utc>> {
        let enabled = |id| self.rules.iter().any(|rule| rule.id == id && rule.enabled);
        if !enabled("eye") || !enabled("stand") { return None; }
        let next = |id| self.timing.get(id).and_then(|timing| {
            if timing.active_event_id.is_none() { timing.next_at } else { None }
        });
        merge_deadline(next("eye"), next("stand"), self.quiet_start, self.quiet_end)
    }
}

impl Scheduler {
    pub fn new(database: &Database) -> DbResult<Self> {
        let rules = database.reminder_rules()?;
        let (start, end) = database.quiet_hours()?;
        let quiet_start = NaiveTime::parse_from_str(&start, "%H:%M").map_err(|error| error.to_string())?;
        let quiet_end = NaiveTime::parse_from_str(&end, "%H:%M").map_err(|error| error.to_string())?;
        let local = Local::now();
        let now = Utc::now();
        let was_quiet = quiet_at(local.time(), quiet_start, quiet_end);
        let active = database.active_reminder_events()?;
        let mut timing = HashMap::new();
        for rule in &rules {
            let active_event_id = active.iter().find(|event| event.reminder_id == rule.id).map(|event| event.id.clone());
            let pending = database.pending_snoozed_reminder(&rule.id)?;
            let pending = if let Some((id, at)) = pending {
                if rule.enabled && !was_quiet && at > now { Some((id, at)) }
                else { database.skip_snoozed_reminder(&id)?; None }
            } else { None };
            timing.insert(rule.id.clone(), RuleTiming {
                next_at: if rule.enabled && !was_quiet && active_event_id.is_none() {
                    Some(pending.as_ref().map(|(_, at)| *at).unwrap_or_else(|| next_after(now, rule.interval_minutes)))
                } else { None },
                active_event_id,
                snoozed_event_id: pending.map(|(id, _)| id),
            });
        }
        Ok(Self { engine: Mutex::new(Engine {
            rules, timing, quiet_start, quiet_end, last_tick: now,
            last_day: local.date_naive(), was_quiet,
        }) })
    }

    pub fn start(app: AppHandle) {
        thread::spawn(move || loop {
            thread::sleep(StdDuration::from_secs(1));
            let scheduler = app.state::<Scheduler>();
            let database = app.state::<Database>();
            if let Err(error) = scheduler.tick(&database, &app) {
                eprintln!("Capsule scheduler: {error}");
            }
        });
    }

    pub fn tick(&self, database: &Database, app: &AppHandle) -> DbResult<()> {
        let now = Utc::now();
        let local = Local::now();
        let mut engine = self.engine.lock().map_err(|error| error.to_string())?;

        // The date event refreshes Today's tasks and manual carry candidates.
        if local.date_naive() != engine.last_day {
            engine.last_day = local.date_naive();
            let _ = app.emit("day://changed", ());
        }

        let quiet = quiet_at(local.time(), engine.quiet_start, engine.quiet_end);
        let reset = resumed_after_sleep(engine.last_tick, now) || quiet != engine.was_quiet;
        engine.last_tick = now;
        engine.was_quiet = quiet;
        let rules = engine.rules.clone();
        let attention = app.state::<Attention>();
        if reset {
            // A wake or quiet-hours boundary starts a fresh cycle; missed ticks are never replayed.
            for rule in &rules {
                if let Some(timing) = engine.timing.get_mut(&rule.id) {
                    if let Some(id) = timing.snoozed_event_id.take() {
                        database.skip_snoozed_reminder(&id)?;
                    }
                    if timing.active_event_id.is_none() {
                        timing.next_at = if rule.enabled && !quiet {
                            Some(next_after(now, rule.interval_minutes))
                        } else { None };
                    }
                }
            }
            let _ = app.emit("reminder://updated", ());
        }
        let mut batches = Vec::new();
        if !quiet {
            let pair_at = engine.pair_deadline();
            if pair_at.is_some_and(|at| at <= now) {
                let scheduled = ["eye", "stand"].iter().map(|id| {
                    let timing = engine.timing.get(*id).ok_or("Reminder timing not found")?;
                    let next_at = timing.next_at.ok_or("Paired reminder deadline missing")?;
                    Ok((id.to_string(), next_at.to_rfc3339(), timing.snoozed_event_id.clone()))
                }).collect::<DbResult<Vec<_>>>()?;
                let pair = database.trigger_reminders(&scheduled)?;
                for event in &pair {
                    let timing = engine.timing.get_mut(&event.reminder_id).ok_or("Reminder timing not found")?;
                    timing.next_at = None;
                    timing.snoozed_event_id = None;
                    timing.active_event_id = Some(event.id.clone());
                }
                batches.push(pair);
            }
            for rule in &rules {
                if !rule.enabled { continue; }
                if pair_at.is_some() && (rule.id == "eye" || rule.id == "stand") { continue; }
                let Some(timing) = engine.timing.get_mut(&rule.id) else { continue; };
                let Some(next_at) = timing.next_at else { continue; };
                if timing.active_event_id.is_some() || next_at > now { continue; }
                let event = database.trigger_reminder(&rule.id, &next_at.to_rfc3339(), timing.snoozed_event_id.as_deref())?;
                timing.next_at = None;
                timing.snoozed_event_id = None;
                timing.active_event_id = Some(event.id.clone());
                batches.push(vec![event]);
            }
        }
        let active = engine.timing.values().any(|timing| timing.active_event_id.is_some());
        drop(engine);
        attention.tick(app, now, active, quiet, reset)?;
        for batch in batches {
            for event in &batch { let _ = app.emit("reminder://triggered", event); }
            let _ = app.emit("reminder://updated", ());
            attention.triggered(app, now, batch.iter().map(|event| event.id.clone()).collect())?;
        }
        Ok(())
    }

    pub fn snapshot(&self, database: &Database) -> DbResult<ReminderSnapshot> {
        let engine = self.engine.lock().map_err(|error| error.to_string())?;
        let pair_at = engine.pair_deadline();
        let mut next = engine.timing.iter().filter_map(|(id, timing)| {
            timing.next_at.map(|at| NextReminder {
                reminder_id: id.clone(),
                next_trigger_at: if id == "eye" || id == "stand" { pair_at.unwrap_or(at) } else { at }.to_rfc3339(),
            })
        }).collect::<Vec<_>>();
        next.sort_by(|a, b| a.next_trigger_at.cmp(&b.next_trigger_at));
        let (attention_sound, attention_repeat) = database.attention_settings()?;
        Ok(ReminderSnapshot {
            rules: engine.rules.clone(), next,
            active: database.active_reminder_events()?,
            quiet_start: engine.quiet_start.format("%H:%M").to_string(),
            quiet_end: engine.quiet_end.format("%H:%M").to_string(),
            attention_sound,
            attention_repeat,
            attention_visible: false,
            attention_event_ids: Vec::new(),
        })
    }

    pub fn update_rule(&self, database: &Database, rule: ReminderRule) -> DbResult<()> {
        let mut engine = self.engine.lock().map_err(|error| error.to_string())?;
        let now = Utc::now();
        let quiet = quiet_at(Local::now().time(), engine.quiet_start, engine.quiet_end);
        let active = database.active_reminder_events()?.into_iter()
            .find(|event| event.reminder_id == rule.id).map(|event| event.id);
        let timing = engine.timing.entry(rule.id.clone()).or_insert(RuleTiming {
            next_at: None, active_event_id: None, snoozed_event_id: None,
        });
        if let Some(id) = timing.snoozed_event_id.take() {
            database.skip_snoozed_reminder(&id)?;
        }
        timing.active_event_id = active;
        timing.next_at = if rule.enabled && !quiet && timing.active_event_id.is_none() {
            Some(next_after(now, rule.interval_minutes))
        } else { None };
        if let Some(existing) = engine.rules.iter_mut().find(|existing| existing.id == rule.id) {
            *existing = rule;
        }
        Ok(())
    }

    pub fn update_quiet_hours(&self, start: &str, end: &str) -> DbResult<()> {
        let start = NaiveTime::parse_from_str(start, "%H:%M").map_err(|error| error.to_string())?;
        let end = NaiveTime::parse_from_str(end, "%H:%M").map_err(|error| error.to_string())?;
        let mut engine = self.engine.lock().map_err(|error| error.to_string())?;
        engine.quiet_start = start;
        engine.quiet_end = end;
        engine.was_quiet = !quiet_at(Local::now().time(), start, end);
        Ok(())
    }

    pub fn complete(&self, rule_id: &str, interval_minutes: i64) -> DbResult<()> {
        let mut engine = self.engine.lock().map_err(|error| error.to_string())?;
        let quiet = quiet_at(Local::now().time(), engine.quiet_start, engine.quiet_end);
        let timing = engine.timing.get_mut(rule_id).ok_or("Reminder timing not found")?;
        timing.active_event_id = None;
        timing.next_at = if quiet { None } else { Some(next_after(Utc::now(), interval_minutes)) };
        Ok(())
    }

    pub fn snooze(&self, rule_id: &str, id: &str, until: DateTime<Utc>) -> DbResult<()> {
        let mut engine = self.engine.lock().map_err(|error| error.to_string())?;
        let timing = engine.timing.get_mut(rule_id).ok_or("Reminder timing not found")?;
        timing.active_event_id = None;
        timing.snoozed_event_id = Some(id.to_string());
        timing.next_at = Some(until);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use rusqlite::Connection;

    #[test]
    fn quiet_hours_wrap_midnight_without_including_end() {
        let start = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        assert!(quiet_at(start, start, end));
        assert!(quiet_at(start.with_hour(3).unwrap(), start, end));
        assert!(!quiet_at(end, start, end));
        assert!(!quiet_at(start.with_hour(14).unwrap(), start, end));
    }

    #[test]
    fn wake_gap_restarts_cycle() {
        let now = Utc::now();
        assert!(resumed_after_sleep(now - Duration::hours(2), now));
        assert!(!resumed_after_sleep(now - Duration::seconds(30), now));
        assert_eq!(next_after(now, 30), now + Duration::minutes(30));
    }

    #[test]
    fn disabled_rule_has_no_next_trigger_and_interval_change_reschedules() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        db.set_quiet_hours("00:00", "00:00").unwrap();
        let scheduler = Scheduler::new(&db).unwrap();
        assert_eq!(scheduler.snapshot(&db).unwrap().next.len(), 3);

        let rule = db.update_reminder_rule("eye", false, 30, 10).unwrap();
        scheduler.update_rule(&db, rule).unwrap();
        assert!(!scheduler.snapshot(&db).unwrap().next.iter().any(|next| next.reminder_id == "eye"));

        let rule = db.update_reminder_rule("water", true, 60, 10).unwrap();
        scheduler.update_rule(&db, rule).unwrap();
        let next = scheduler.snapshot(&db).unwrap().next.into_iter().find(|next| next.reminder_id == "water").unwrap();
        let at = DateTime::parse_from_rfc3339(&next.next_trigger_at).unwrap().with_timezone(&Utc);
        assert!((at - Utc::now()).num_minutes() >= 59);
    }

    #[test]
    fn snooze_deadline_survives_scheduler_restart() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        db.set_quiet_hours("00:00", "00:00").unwrap();
        let event = db.trigger_reminder("eye", &Utc::now().to_rfc3339(), None).unwrap();
        let (_, _, until) = db.snooze_reminders(&[event.id.clone()]).unwrap().remove(0);
        let restarted = Scheduler::new(&db).unwrap();
        let eye = restarted.snapshot(&db).unwrap().next.into_iter()
            .find(|next| next.reminder_id == "eye").unwrap();
        assert_eq!(eye.next_trigger_at, until.to_rfc3339());
    }

    #[test]
    fn eye_and_stand_wait_for_the_later_deadline_within_ten_minutes() {
        let eye = DateTime::parse_from_rfc3339("2026-10-14T14:30:00+08:00").unwrap().with_timezone(&Utc);
        let stand = eye + Duration::minutes(2);
        let no_quiet = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        assert_eq!(merge_deadline(Some(eye), Some(stand), no_quiet, no_quiet), Some(stand));
        assert_eq!(merge_deadline(Some(stand), Some(eye), no_quiet, no_quiet), Some(stand));
        assert_eq!(merge_deadline(Some(eye), Some(eye + Duration::minutes(10)), no_quiet, no_quiet), Some(eye + Duration::minutes(10)));
        assert_eq!(merge_deadline(Some(eye), Some(eye + Duration::minutes(11)), no_quiet, no_quiet), None);
        assert_eq!(merge_deadline(Some(eye), None, no_quiet, no_quiet), None);
        let quiet_start = stand.with_timezone(&Local).time();
        let quiet_end = (stand + Duration::hours(1)).with_timezone(&Local).time();
        assert_eq!(merge_deadline(Some(eye), Some(stand), quiet_start, quiet_end), None);
    }

    #[test]
    fn countdown_uses_group_display_time_and_disabling_restores_independent_time() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        db.set_quiet_hours("00:00", "00:00").unwrap();
        let scheduler = Scheduler::new(&db).unwrap();
        let stand = db.update_reminder_rule("stand", true, 32, 10).unwrap();
        scheduler.update_rule(&db, stand).unwrap();
        let next = scheduler.snapshot(&db).unwrap().next;
        let eye_at = next.iter().find(|item| item.reminder_id == "eye").unwrap().next_trigger_at.clone();
        let stand_at = next.iter().find(|item| item.reminder_id == "stand").unwrap().next_trigger_at.clone();
        assert_eq!(eye_at, stand_at);
        let eye = db.update_reminder_rule("eye", false, 30, 10).unwrap();
        scheduler.update_rule(&db, eye).unwrap();
        let next = scheduler.snapshot(&db).unwrap().next;
        assert!(!next.iter().any(|item| item.reminder_id == "eye"));
        assert!(next.iter().any(|item| item.reminder_id == "stand"));
    }
}
