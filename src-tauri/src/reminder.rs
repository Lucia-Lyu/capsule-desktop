use chrono::{DateTime, Duration, NaiveTime, Utc};
use rusqlite::{params, OptionalExtension, Row};

use crate::{
    database::{Database, DbResult},
    models::{ReminderEvent, ReminderRule},
};

fn rule_from_row(row: &Row<'_>) -> rusqlite::Result<ReminderRule> {
    Ok(ReminderRule {
        id: row.get(0)?,
        name: row.get(1)?,
        icon: row.get(2)?,
        interval_minutes: row.get(3)?,
        snooze_minutes: row.get(4)?,
        enabled: row.get::<_, i64>(5)? != 0,
        updated_at: row.get(6)?,
    })
}

pub(crate) fn event_from_row(row: &Row<'_>) -> rusqlite::Result<ReminderEvent> {
    Ok(ReminderEvent {
        id: row.get(0)?,
        reminder_id: row.get(1)?,
        scheduled_at: row.get(2)?,
        triggered_at: row.get(3)?,
        action: row.get(4)?,
        completed_at: row.get(5)?,
        snooze_count: row.get(6)?,
    })
}

fn validate_resolution_group(selected: &[(String, String)]) -> DbResult<()> {
    if selected.len() == 1 { return Ok(()); }
    if selected.len() == 2
        && selected[0].1 == selected[1].1
        && ((selected[0].0 == "eye" && selected[1].0 == "stand")
            || (selected[0].0 == "stand" && selected[1].0 == "eye"))
    {
        return Ok(());
    }
    Err("Only an eye/stand pair triggered together can be resolved as a group".into())
}

impl Database {
    pub fn attention_settings(&self) -> DbResult<(bool, bool)> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let sound: String = connection.query_row(
            "SELECT value FROM app_settings WHERE key = 'attention_sound'", [], |row| row.get(0),
        ).map_err(|error| error.to_string())?;
        let repeat: String = connection.query_row(
            "SELECT value FROM app_settings WHERE key = 'attention_repeat'", [], |row| row.get(0),
        ).map_err(|error| error.to_string())?;
        Ok((sound == "true", repeat == "true"))
    }

    pub fn set_attention_settings(&self, sound: bool, repeat: bool) -> DbResult<()> {
        let mut connection = self.connection.lock().map_err(|error| error.to_string())?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        for (key, value) in [
            ("attention_sound", if sound { "true" } else { "false" }),
            ("attention_repeat", if repeat { "true" } else { "false" }),
        ] {
            transaction.execute(
                "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            ).map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())
    }

    pub fn reminder_rules(&self) -> DbResult<Vec<ReminderRule>> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let mut statement = connection.prepare(
            "SELECT id, name, icon, interval_minutes, snooze_minutes, enabled, updated_at
             FROM reminder_rules ORDER BY CASE id WHEN 'eye' THEN 0 WHEN 'water' THEN 1 ELSE 2 END",
        ).map_err(|error| error.to_string())?;
        let rules = statement.query_map([], rule_from_row)
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        Ok(rules)
    }

    pub fn reminder_rule(&self, id: &str) -> DbResult<ReminderRule> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        connection.query_row(
            "SELECT id, name, icon, interval_minutes, snooze_minutes, enabled, updated_at
             FROM reminder_rules WHERE id = ?1",
            params![id], rule_from_row,
        ).optional().map_err(|error| error.to_string())?
            .ok_or_else(|| "Reminder rule not found".into())
    }

    pub fn update_reminder_rule(&self, id: &str, enabled: bool, interval_minutes: i64, snooze_minutes: i64) -> DbResult<ReminderRule> {
        if !(1..=240).contains(&interval_minutes) || !(1..=120).contains(&snooze_minutes) {
            return Err("Intervals must be between 1 and 240 minutes; snooze between 1 and 120 minutes".into());
        }
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let changed = connection.execute(
            "UPDATE reminder_rules SET enabled = ?1, interval_minutes = ?2, snooze_minutes = ?3, updated_at = ?4 WHERE id = ?5",
            params![enabled as i64, interval_minutes, snooze_minutes, Utc::now().to_rfc3339(), id],
        ).map_err(|error| error.to_string())?;
        if changed == 0 { return Err("Reminder rule not found".into()); }
        if !enabled {
            connection.execute(
                "UPDATE reminder_events SET action = 'skipped' WHERE reminder_id = ?1 AND action = 'triggered'",
                params![id],
            ).map_err(|error| error.to_string())?;
        }
        connection.query_row(
            "SELECT id, name, icon, interval_minutes, snooze_minutes, enabled, updated_at FROM reminder_rules WHERE id = ?1",
            params![id], rule_from_row,
        ).map_err(|error| error.to_string())
    }

    pub fn quiet_hours(&self) -> DbResult<(String, String)> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let start: String = connection.query_row(
            "SELECT value FROM app_settings WHERE key = 'quiet_start'", [], |row| row.get(0),
        ).map_err(|error| error.to_string())?;
        let end: String = connection.query_row(
            "SELECT value FROM app_settings WHERE key = 'quiet_end'", [], |row| row.get(0),
        ).map_err(|error| error.to_string())?;
        Ok((start, end))
    }

    pub fn set_quiet_hours(&self, start: &str, end: &str) -> DbResult<()> {
        NaiveTime::parse_from_str(start, "%H:%M").map_err(|_| "Invalid quiet start".to_string())?;
        NaiveTime::parse_from_str(end, "%H:%M").map_err(|_| "Invalid quiet end".to_string())?;
        let mut connection = self.connection.lock().map_err(|error| error.to_string())?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        for (key, value) in [("quiet_start", start), ("quiet_end", end)] {
            transaction.execute(
                "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            ).map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())
    }

    pub fn active_reminder_events(&self) -> DbResult<Vec<ReminderEvent>> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let mut statement = connection.prepare(
            "SELECT id, reminder_id, scheduled_at, triggered_at, action, completed_at, snooze_count
             FROM reminder_events WHERE action = 'triggered' ORDER BY triggered_at",
        ).map_err(|error| error.to_string())?;
        let events = statement.query_map([], event_from_row)
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        Ok(events)
    }

    pub fn trigger_reminder(&self, reminder_id: &str, scheduled_at: &str, snoozed_event_id: Option<&str>) -> DbResult<ReminderEvent> {
        let scheduled = vec![(reminder_id.to_string(), scheduled_at.to_string(), snoozed_event_id.map(str::to_string))];
        self.trigger_reminders(&scheduled)?.into_iter().next().ok_or_else(|| "Reminder was not triggered".into())
    }

    pub fn trigger_reminders(&self, scheduled: &[(String, String, Option<String>)]) -> DbResult<Vec<ReminderEvent>> {
        let mut connection = self.connection.lock().map_err(|error| error.to_string())?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        let now = Utc::now().to_rfc3339();
        let mut events = Vec::with_capacity(scheduled.len());
        for (reminder_id, scheduled_at, snoozed_event_id) in scheduled {
            let id = if let Some(id) = snoozed_event_id {
                let changed = transaction.execute(
                    "UPDATE reminder_events SET action = 'triggered', triggered_at = ?1, snooze_until = NULL WHERE id = ?2 AND action = 'snoozed'",
                    params![now, id],
                ).map_err(|error| error.to_string())?;
                if changed == 0 { return Err("Snoozed reminder not found".into()); }
                id.to_string()
            } else {
                let id: String = transaction.query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
                    .map_err(|error| error.to_string())?;
                transaction.execute(
                    "INSERT INTO reminder_events (id, reminder_id, scheduled_at, triggered_at, action)
                     VALUES (?1, ?2, ?3, ?4, 'triggered')",
                    params![id, reminder_id, scheduled_at, now],
                ).map_err(|error| error.to_string())?;
                id
            };
            events.push(transaction.query_row(
                "SELECT id, reminder_id, scheduled_at, triggered_at, action, completed_at, snooze_count FROM reminder_events WHERE id = ?1",
                params![id], event_from_row,
            ).map_err(|error| error.to_string())?);
        }
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(events)
    }

    pub fn complete_reminders(&self, ids: &[String]) -> DbResult<Vec<String>> {
        if ids.is_empty() || ids.len() > 2 { return Err("Select one reminder or the eye/stand pair".into()); }
        let mut connection = self.connection.lock().map_err(|error| error.to_string())?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        let mut selected = Vec::with_capacity(ids.len());
        for id in ids {
            let event: (String, String) = transaction.query_row(
                "SELECT reminder_id, triggered_at FROM reminder_events WHERE id = ?1 AND action = 'triggered'",
                params![id], |row| Ok((row.get(0)?, row.get(1)?)),
            ).optional().map_err(|error| error.to_string())?.ok_or("Active reminder not found")?;
            selected.push(event);
        }
        validate_resolution_group(&selected)?;
        let completed_at = Utc::now().to_rfc3339();
        for id in ids {
            transaction.execute(
                "UPDATE reminder_events SET action = 'completed', completed_at = ?1 WHERE id = ?2",
                params![completed_at, id],
            ).map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(selected.into_iter().map(|(rule_id, _)| rule_id).collect())
    }

    pub fn snooze_reminders(&self, ids: &[String]) -> DbResult<Vec<(String, String, DateTime<Utc>)>> {
        if ids.is_empty() || ids.len() > 2 { return Err("Select one reminder or the eye/stand pair".into()); }
        let mut connection = self.connection.lock().map_err(|error| error.to_string())?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        let mut selected = Vec::with_capacity(ids.len());
        for id in ids {
            let event: (String, i64, String) = transaction.query_row(
                "SELECT e.reminder_id, r.snooze_minutes, e.triggered_at FROM reminder_events e
                 JOIN reminder_rules r ON r.id = e.reminder_id
                 WHERE e.id = ?1 AND e.action = 'triggered'",
                params![id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            ).optional().map_err(|error| error.to_string())?.ok_or("Active reminder not found")?;
            selected.push(event);
        }
        let group = selected.iter().map(|(rule_id, _, triggered_at)| (rule_id.clone(), triggered_at.clone())).collect::<Vec<_>>();
        validate_resolution_group(&group)?;
        let minutes = selected.iter().map(|(_, minutes, _)| *minutes).max().ok_or("No reminder selected")?;
        let until = Utc::now() + Duration::minutes(minutes);
        for id in ids {
            transaction.execute(
                "UPDATE reminder_events SET action = 'snoozed', snooze_count = snooze_count + 1, snooze_until = ?1 WHERE id = ?2",
                params![until.to_rfc3339(), id],
            ).map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(ids.iter().zip(selected).map(|(id, (rule_id, _, _))| (id.clone(), rule_id, until)).collect())
    }

    pub fn pending_snoozed_reminder(&self, reminder_id: &str) -> DbResult<Option<(String, DateTime<Utc>)>> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let pending: Option<(String, String)> = connection.query_row(
            "SELECT id, snooze_until FROM reminder_events
             WHERE reminder_id = ?1 AND action = 'snoozed' AND snooze_until IS NOT NULL
             ORDER BY triggered_at DESC LIMIT 1",
            params![reminder_id], |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional().map_err(|error| error.to_string())?;
        pending.map(|(id, until)| {
            let at = DateTime::parse_from_rfc3339(&until).map_err(|error| error.to_string())?;
            Ok((id, at.with_timezone(&Utc)))
        }).transpose()
    }

    pub fn skip_snoozed_reminder(&self, id: &str) -> DbResult<()> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        connection.execute(
            "UPDATE reminder_events SET action = 'skipped' WHERE id = ?1 AND action = 'snoozed'",
            params![id],
        ).map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn reminder_events_since(&self, since: &str) -> DbResult<Vec<ReminderEvent>> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let mut statement = connection.prepare(
            "SELECT id, reminder_id, scheduled_at, triggered_at, action, completed_at, snooze_count
             FROM reminder_events WHERE scheduled_at >= ?1 ORDER BY scheduled_at",
        ).map_err(|error| error.to_string())?;
        let events = statement.query_map(params![since], event_from_row)
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn snooze_reuses_event_and_completion_is_persistent() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let scheduled = Utc::now().to_rfc3339();
        let event = db.trigger_reminder("eye", &scheduled, None).unwrap();
        assert_eq!(db.active_reminder_events().unwrap().len(), 1);
        assert_eq!(db.snooze_reminders(&[event.id.clone()]).unwrap()[0].1, "eye");
        assert!(db.active_reminder_events().unwrap().is_empty());
        assert_eq!(db.pending_snoozed_reminder("eye").unwrap().unwrap().0, event.id);
        let retriggered = db.trigger_reminder("eye", &scheduled, Some(&event.id)).unwrap();
        assert_eq!(retriggered.id, event.id);
        assert_eq!(retriggered.snooze_count, 1);
        db.complete_reminders(&[event.id.clone()]).unwrap();
        let recorded = db.reminder_events_since("2000-01-01").unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].action, "completed");
    }

    #[test]
    fn disabling_rule_skips_active_event() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        db.trigger_reminder("water", &Utc::now().to_rfc3339(), None).unwrap();
        db.update_reminder_rule("water", false, 45, 10).unwrap();
        assert!(db.active_reminder_events().unwrap().is_empty());
        assert_eq!(db.reminder_events_since("2000-01-01").unwrap()[0].action, "skipped");
    }

    #[test]
    fn eye_and_stand_share_one_trigger_and_one_resolution_but_keep_separate_history() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        db.update_reminder_rule("eye", true, 30, 5).unwrap();
        db.update_reminder_rule("stand", true, 60, 15).unwrap();
        let eye_at = Utc::now().to_rfc3339();
        let stand_at = (Utc::now() + Duration::minutes(2)).to_rfc3339();
        let scheduled = vec![("eye".into(), eye_at.clone(), None), ("stand".into(), stand_at.clone(), None)];
        let events = db.trigger_reminders(&scheduled).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].triggered_at, events[1].triggered_at);
        assert_eq!(events[0].scheduled_at, eye_at);
        assert_eq!(events[1].scheduled_at, stand_at);
        let ids = events.iter().map(|event| event.id.clone()).collect::<Vec<_>>();
        let snoozed = db.snooze_reminders(&ids).unwrap();
        assert_eq!(snoozed[0].2, snoozed[1].2);
        assert!((snoozed[0].2 - Utc::now()).num_minutes() >= 14);
        let retry = snoozed.iter().map(|(id, rule, until)| (rule.clone(), until.to_rfc3339(), Some(id.clone()))).collect::<Vec<_>>();
        db.trigger_reminders(&retry).unwrap();
        db.complete_reminders(&ids).unwrap();
        let history = db.reminder_events_since("2000-01-01").unwrap();
        assert_eq!(history.len(), 2);
        assert!(history.iter().all(|event| event.action == "completed" && event.snooze_count == 1));
    }

    #[test]
    fn water_cannot_be_resolved_with_an_eye_reminder() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let scheduled = Utc::now().to_rfc3339();
        let events = db.trigger_reminders(&[("eye".into(), scheduled.clone(), None), ("water".into(), scheduled, None)]).unwrap();
        let ids = events.iter().map(|event| event.id.clone()).collect::<Vec<_>>();
        assert!(db.complete_reminders(&ids).is_err());
        assert_eq!(db.active_reminder_events().unwrap().len(), 2);
    }
}
