use std::{collections::HashSet, fs, path::Path, sync::Mutex};

use chrono::{Local, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::models::Task;

pub type DbResult<T> = Result<T, String>;

pub struct Database {
    pub(crate) connection: Mutex<Connection>,
}

pub(crate) fn task_from_row(row: &Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        task_date: row.get(2)?,
        status: row.get(3)?,
        sort_order: row.get(4)?,
        created_at: row.get(5)?,
        completed_at: row.get(6)?,
        origin_created_at: row.get(7)?,
        carried_to_date: row.get(8)?,
    })
}

impl Database {
    pub fn open(path: &Path) -> DbResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let connection = Connection::open(path).map_err(|error| error.to_string())?;
        Self::from_connection(connection)
    }

    pub(crate) fn from_connection(connection: Connection) -> DbResult<Self> {
        connection
            .execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")
            .map_err(|error| error.to_string())?;
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|error| error.to_string())?;
        if version < 1 {
            connection.execute_batch(
                "BEGIN;
                 CREATE TABLE IF NOT EXISTS tasks (
                   id TEXT PRIMARY KEY,
                   title TEXT NOT NULL,
                   task_date TEXT NOT NULL,
                   status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'completed')),
                   sort_order INTEGER NOT NULL DEFAULT 0,
                   created_at TEXT NOT NULL,
                   completed_at TEXT
                 );
                 CREATE INDEX IF NOT EXISTS idx_tasks_date_order ON tasks(task_date, sort_order);
                 PRAGMA user_version = 1;
                 COMMIT;",
            ).map_err(|error| error.to_string())?;
        }
        if version < 2 {
            connection.execute_batch(
                "BEGIN;
                 CREATE TABLE IF NOT EXISTS reminder_rules (
                   id TEXT PRIMARY KEY,
                   name TEXT NOT NULL,
                   icon TEXT NOT NULL,
                   interval_minutes INTEGER NOT NULL,
                   snooze_minutes INTEGER NOT NULL DEFAULT 10,
                   enabled INTEGER NOT NULL DEFAULT 1,
                   updated_at TEXT NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS reminder_events (
                   id TEXT PRIMARY KEY,
                   reminder_id TEXT NOT NULL,
                   scheduled_at TEXT NOT NULL,
                   triggered_at TEXT,
                   action TEXT NOT NULL CHECK (action IN ('triggered', 'completed', 'snoozed', 'skipped')),
                   completed_at TEXT,
                   snooze_count INTEGER NOT NULL DEFAULT 0,
                   FOREIGN KEY (reminder_id) REFERENCES reminder_rules(id)
                 );
                 CREATE INDEX IF NOT EXISTS idx_reminder_events_scheduled ON reminder_events(scheduled_at);
                 CREATE TABLE IF NOT EXISTS app_settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 PRAGMA user_version = 2;
                 COMMIT;",
            ).map_err(|error| error.to_string())?;
        }
        if version < 3 {
            connection.execute_batch(
                "BEGIN;
                 ALTER TABLE reminder_events ADD COLUMN snooze_until TEXT;
                 PRAGMA user_version = 3;
                 COMMIT;",
            ).map_err(|error| error.to_string())?;
        }
        if version < 4 {
            connection.execute_batch(
                "BEGIN;
                 ALTER TABLE tasks ADD COLUMN origin_created_at TEXT;
                 ALTER TABLE tasks ADD COLUMN carried_to_date TEXT;
                 CREATE INDEX IF NOT EXISTS idx_tasks_carry_candidates
                   ON tasks(status, carried_to_date, task_date);
                 PRAGMA user_version = 4;
                 COMMIT;",
            ).map_err(|error| error.to_string())?;
        }
        let now = Utc::now().to_rfc3339();
        for (id, name, icon, minutes) in [
            ("eye", "Eye", "👁", 30),
            ("water", "Water", "💧", 45),
            ("stand", "Stand", "🚶", 60),
        ] {
            connection.execute(
                "INSERT OR IGNORE INTO reminder_rules (id, name, icon, interval_minutes, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, name, icon, minutes, now],
            ).map_err(|error| error.to_string())?;
        }
        for (key, value) in [
            ("quiet_start", "22:00"),
            ("quiet_end", "08:00"),
            ("attention_sound", "true"),
            ("attention_repeat", "true"),
            ("capsule_theme", "sage"),
        ] {
            connection.execute(
                "INSERT OR IGNORE INTO app_settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            ).map_err(|error| error.to_string())?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn today_tasks(&self) -> DbResult<Vec<Task>> {
        self.tasks_for_date(&Local::now().date_naive().to_string())
    }

    pub fn capsule_theme(&self) -> DbResult<String> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        connection.query_row(
            "SELECT value FROM app_settings WHERE key = 'capsule_theme'",
            [], |row| row.get(0),
        ).map_err(|error| error.to_string())
    }

    pub fn set_capsule_theme(&self, theme: &str) -> DbResult<()> {
        if !["sage", "charcoal", "mist", "blue", "yellow"].contains(&theme) {
            return Err("Unknown capsule color".into());
        }
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        connection.execute(
            "INSERT INTO app_settings (key, value) VALUES ('capsule_theme', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![theme],
        ).map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn carry_candidates(&self) -> DbResult<Vec<Task>> {
        let today = Local::now().date_naive().to_string();
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let mut statement = connection.prepare(
            "SELECT id, title, task_date, status, sort_order, created_at, completed_at, origin_created_at, carried_to_date
             FROM tasks WHERE task_date < ?1 AND status = 'pending' AND carried_to_date IS NULL
             ORDER BY task_date DESC, sort_order, created_at",
        ).map_err(|error| error.to_string())?;
        let tasks = statement.query_map(params![today], task_from_row)
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        Ok(tasks)
    }

    pub fn carry_tasks(&self, ids: &[String], today: &str) -> DbResult<()> {
        if ids.is_empty() || ids.iter().collect::<HashSet<_>>().len() != ids.len() {
            return Err("Choose one or more distinct tasks to carry forward".into());
        }
        let mut connection = self.connection.lock().map_err(|error| error.to_string())?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        let first_order: i64 = transaction.query_row(
            "SELECT COALESCE(MIN(sort_order), 0) FROM tasks WHERE task_date = ?1",
            params![today], |row| row.get(0),
        ).map_err(|error| error.to_string())?;
        let created_at = Utc::now().to_rfc3339();
        for (index, id) in ids.iter().enumerate() {
            let source: Option<(String, String, Option<String>)> = transaction.query_row(
                "SELECT title, created_at, origin_created_at FROM tasks
                 WHERE id = ?1 AND task_date < ?2 AND status = 'pending' AND carried_to_date IS NULL",
                params![id, today], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            ).optional().map_err(|error| error.to_string())?;
            let (title, source_created_at, origin_created_at) = source.ok_or("Task is no longer available to carry forward")?;
            transaction.execute(
                "UPDATE tasks SET carried_to_date = ?1 WHERE id = ?2",
                params![today, id],
            ).map_err(|error| error.to_string())?;
            let new_id: String = transaction.query_row(
                "SELECT lower(hex(randomblob(16)))", [], |row| row.get(0),
            ).map_err(|error| error.to_string())?;
            transaction.execute(
                "INSERT INTO tasks (id, title, task_date, status, sort_order, created_at, origin_created_at)
                 VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6)",
                params![
                    new_id, title, today, first_order - ids.len() as i64 + index as i64,
                    created_at, origin_created_at.unwrap_or(source_created_at),
                ],
            ).map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())
    }

    pub fn defer_task_to_tomorrow(&self, id: &str, today: &str, tomorrow: &str) -> DbResult<()> {
        if chrono::NaiveDate::parse_from_str(today, "%Y-%m-%d")
            .ok()
            .and_then(|day| day.succ_opt())
            .map(|day| day.to_string())
            .as_deref() != Some(tomorrow)
        {
            return Err("Target must be the next day".into());
        }
        {
            let connection = self.connection.lock().map_err(|error| error.to_string())?;
            let eligible: bool = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM tasks WHERE id = ?1 AND task_date = ?2 AND status = 'pending' AND carried_to_date IS NULL)",
                params![id, today], |row| row.get(0),
            ).map_err(|error| error.to_string())?;
            if !eligible { return Err("Only an unfinished task from today can be moved to tomorrow".into()); }
        }
        self.carry_tasks(&[id.to_string()], tomorrow)
    }

    pub fn tasks_for_date(&self, date: &str) -> DbResult<Vec<Task>> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT id, title, task_date, status, sort_order, created_at, completed_at, origin_created_at, carried_to_date
                 FROM tasks WHERE task_date = ?1
                 ORDER BY CASE status WHEN 'pending' THEN 0 ELSE 1 END, sort_order, created_at",
            )
            .map_err(|error| error.to_string())?;
        let tasks = statement
            .query_map(params![date], task_from_row)
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        Ok(tasks)
    }

    pub fn tasks_since(&self, date: &str) -> DbResult<Vec<Task>> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let mut statement = connection.prepare(
            "SELECT id, title, task_date, status, sort_order, created_at, completed_at, origin_created_at, carried_to_date
             FROM tasks WHERE task_date >= ?1
             ORDER BY task_date DESC, CASE status WHEN 'pending' THEN 0 ELSE 1 END, sort_order, created_at",
        ).map_err(|error| error.to_string())?;
        let tasks = statement.query_map(params![date], task_from_row)
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        Ok(tasks)
    }

    pub fn create_task(&self, title: &str, date: &str) -> DbResult<Task> {
        let title = title.trim();
        if title.is_empty() || title.chars().count() > 240 {
            return Err("Task title must contain 1 to 240 characters".into());
        }
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let order: i64 = connection
            .query_row(
                "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM tasks WHERE task_date = ?1",
                params![date],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        let id: String = connection
            .query_row("SELECT lower(hex(randomblob(16)))", [], |row| row.get(0))
            .map_err(|error| error.to_string())?;
        let created_at = Utc::now().to_rfc3339();
        connection
            .execute(
                "INSERT INTO tasks (id, title, task_date, sort_order, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, title, date, order, created_at],
            )
            .map_err(|error| error.to_string())?;
        Ok(Task {
            id,
            title: title.to_string(),
            task_date: date.to_string(),
            status: "pending".into(),
            sort_order: order,
            created_at,
            completed_at: None,
            origin_created_at: None,
            carried_to_date: None,
        })
    }

    pub fn set_task_completed(&self, id: &str, completed: bool) -> DbResult<()> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let (status, completed_at) = if completed {
            ("completed", Some(Utc::now().to_rfc3339()))
        } else {
            ("pending", None)
        };
        let changed = connection
            .execute(
                "UPDATE tasks SET status = ?1, completed_at = ?2 WHERE id = ?3 AND carried_to_date IS NULL",
                params![status, completed_at, id],
            )
            .map_err(|error| error.to_string())?;
        if changed == 0 {
            return Err("Task is unavailable or was already continued".into());
        }
        Ok(())
    }

    pub fn delete_task(&self, id: &str) -> DbResult<()> {
        let connection = self.connection.lock().map_err(|error| error.to_string())?;
        let changed = connection
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])
            .map_err(|error| error.to_string())?;
        if changed == 0 {
            return Err("Task not found".into());
        }
        Ok(())
    }

    pub fn move_task(&self, id: &str, direction: i64) -> DbResult<()> {
        if ![-1, 1].contains(&direction) {
            return Err("Invalid move direction".into());
        }
        let mut connection = self.connection.lock().map_err(|error| error.to_string())?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        let current: Option<(String, String, i64)> = transaction
            .query_row(
                "SELECT task_date, status, sort_order FROM tasks WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        let (date, status, order) = current.ok_or("Task not found")?;
        let neighbor: Option<(String, i64)> = if direction < 0 {
            transaction.query_row(
                "SELECT id, sort_order FROM tasks WHERE task_date = ?1 AND status = ?2 AND sort_order < ?3 ORDER BY sort_order DESC LIMIT 1",
                params![date, status, order],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
        } else {
            transaction.query_row(
                "SELECT id, sort_order FROM tasks WHERE task_date = ?1 AND status = ?2 AND sort_order > ?3 ORDER BY sort_order ASC LIMIT 1",
                params![date, status, order],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
        }
        .optional()
        .map_err(|error| error.to_string())?;
        if let Some((neighbor_id, neighbor_order)) = neighbor {
            transaction
                .execute(
                    "UPDATE tasks SET sort_order = ?1 WHERE id = ?2",
                    params![neighbor_order, id],
                )
                .map_err(|error| error.to_string())?;
            transaction
                .execute(
                    "UPDATE tasks SET sort_order = ?1 WHERE id = ?2",
                    params![order, neighbor_id],
                )
                .map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn tasks_create_complete_uncomplete_move_delete_and_filter_by_date() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let first = db.create_task(" First ", "2026-10-12").unwrap();
        let second = db.create_task("Second", "2026-10-12").unwrap();
        db.create_task("Tomorrow", "2026-10-13").unwrap();
        assert_eq!(db.tasks_for_date("2026-10-12").unwrap().len(), 2);
        assert_eq!(db.tasks_for_date("2026-10-13").unwrap().len(), 1);
        assert_eq!(first.title, "First");

        db.move_task(&second.id, -1).unwrap();
        assert_eq!(db.tasks_for_date("2026-10-12").unwrap()[0].id, second.id);
        db.set_task_completed(&second.id, true).unwrap();
        assert_eq!(db.tasks_for_date("2026-10-12").unwrap()[1].status, "completed");
        db.set_task_completed(&second.id, false).unwrap();
        assert_eq!(db.tasks_for_date("2026-10-12").unwrap()[0].status, "pending");
        db.delete_task(&first.id).unwrap();
        assert_eq!(db.tasks_for_date("2026-10-12").unwrap().len(), 1);
    }

    #[test]
    fn manual_carry_keeps_yesterdays_history_and_original_creation_date() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let today = Local::now().date_naive();
        let yesterday = (today - Duration::days(1)).to_string();
        let today = today.to_string();
        let pending = db.create_task("Read paper", &yesterday).unwrap();
        let completed = db.create_task("Send email", &yesterday).unwrap();
        db.set_task_completed(&completed.id, true).unwrap();
        db.create_task("Today task", &today).unwrap();

        assert_eq!(db.carry_candidates().unwrap().len(), 1);
        assert_eq!(db.tasks_for_date(&today).unwrap().len(), 1);
        db.carry_tasks(&[pending.id.clone()], &today).unwrap();

        let yesterday_tasks = db.tasks_for_date(&yesterday).unwrap();
        assert_eq!(yesterday_tasks.len(), 2);
        assert_eq!(yesterday_tasks.iter().filter(|task| task.status == "completed").count(), 1);
        let old = yesterday_tasks.iter().find(|task| task.id == pending.id).unwrap();
        assert_eq!(old.carried_to_date.as_deref(), Some(today.as_str()));
        assert_eq!(old.status, "pending");

        let today_tasks = db.tasks_for_date(&today).unwrap();
        let continued = today_tasks.iter().find(|task| task.title == pending.title).unwrap();
        assert_ne!(continued.id, pending.id);
        assert_eq!(continued.origin_created_at.as_deref(), Some(pending.created_at.as_str()));
        assert_eq!(continued.status, "pending");
        let history = crate::history::history(&db, 7).unwrap();
        assert_eq!((history[1].tasks_completed, history[1].tasks_total), (1, 2));
        assert_eq!(history[0].tasks_total, 2);
        assert!(db.carry_candidates().unwrap().is_empty());
        assert!(db.carry_tasks(&[pending.id], &today).is_err());
        assert_eq!(db.tasks_for_date(&today).unwrap().len(), 2);
        let tomorrow = (Local::now().date_naive() + Duration::days(1)).to_string();
        db.carry_tasks(&[continued.id.clone()], &tomorrow).unwrap();
        let next = db.tasks_for_date(&tomorrow).unwrap();
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].origin_created_at.as_deref(), Some(pending.created_at.as_str()));
    }

    #[test]
    fn carry_all_rolls_back_if_one_selected_task_is_unavailable() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let today = Local::now().date_naive();
        let yesterday = (today - Duration::days(1)).to_string();
        let pending = db.create_task("Still pending", &yesterday).unwrap();
        let ids = [pending.id, "missing".to_string()];
        assert!(db.carry_tasks(&ids, &today.to_string()).is_err());
        assert_eq!(db.carry_candidates().unwrap().len(), 1);
        assert!(db.tasks_for_date(&today.to_string()).unwrap().is_empty());
    }

    #[test]
    fn defer_selected_task_to_tomorrow_preserves_today_and_original_date() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let today = Local::now().date_naive();
        let tomorrow = (today + Duration::days(1)).to_string();
        let today = today.to_string();
        let task = db.create_task("Finish draft", &today).unwrap();

        db.defer_task_to_tomorrow(&task.id, &today, &tomorrow).unwrap();
        let source = db.tasks_for_date(&today).unwrap();
        assert_eq!(source.len(), 1);
        assert_eq!(source[0].status, "pending");
        assert_eq!(source[0].carried_to_date.as_deref(), Some(tomorrow.as_str()));
        let target = db.tasks_for_date(&tomorrow).unwrap();
        assert_eq!(target.len(), 1);
        assert_eq!(target[0].origin_created_at.as_deref(), Some(task.created_at.as_str()));
        assert!(db.defer_task_to_tomorrow(&task.id, &today, &tomorrow).is_err());
        assert!(db.set_task_completed(&task.id, true).is_err());
        assert!(db.defer_task_to_tomorrow(&target[0].id, &today, &tomorrow).is_err());
    }

    #[test]
    fn capsule_theme_defaults_and_persists_without_changing_on_invalid_input() {
        let db = Database::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        assert_eq!(db.capsule_theme().unwrap(), "sage");
        db.set_capsule_theme("charcoal").unwrap();
        assert_eq!(db.capsule_theme().unwrap(), "charcoal");
        assert!(db.set_capsule_theme("neon").is_err());
        assert_eq!(db.capsule_theme().unwrap(), "charcoal");
    }
}
