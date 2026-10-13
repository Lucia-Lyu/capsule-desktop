use std::collections::HashMap;

use chrono::{DateTime, Duration, Local, NaiveDate, Utc};

use crate::{
    database::{Database, DbResult},
    models::{HistoryDay, ReminderCount, ReminderEvent, Task},
};

fn rate(completed: usize, total: usize) -> u8 {
    if total == 0 { 0 } else { ((completed * 100) / total) as u8 }
}

fn calculate_history(today: NaiveDate, days: i64, tasks: Vec<Task>, events: Vec<ReminderEvent>) -> Vec<HistoryDay> {
    let first = today - Duration::days(days - 1);
    let mut tasks_by_date: HashMap<String, Vec<Task>> = HashMap::new();
    for task in tasks {
        tasks_by_date.entry(task.task_date.clone()).or_default().push(task);
    }
    let mut counts: HashMap<(String, String), (usize, usize)> = HashMap::new();
    for event in events {
        let Ok(scheduled) = DateTime::parse_from_rfc3339(&event.scheduled_at) else { continue; };
        let date = scheduled.with_timezone(&Local).date_naive();
        if date < first || date > today { continue; }
        let count = counts.entry((date.to_string(), event.reminder_id)).or_default();
        count.1 += 1;
        if event.action == "completed" { count.0 += 1; }
    }
    (0..days).map(|offset| {
        let date = (today - Duration::days(offset)).to_string();
        let tasks = tasks_by_date.remove(&date).unwrap_or_default();
        let tasks_total = tasks.len();
        let tasks_completed = tasks.iter().filter(|task| task.status == "completed").count();
        let reminders = ["eye", "water", "stand"].iter().map(|id| {
            let (completed, total) = counts.get(&(date.clone(), id.to_string())).copied().unwrap_or_default();
            ReminderCount { reminder_id: id.to_string(), completed, total, completion_rate: rate(completed, total) }
        }).collect();
        HistoryDay { date, tasks, tasks_completed, tasks_total, task_completion_rate: rate(tasks_completed, tasks_total), reminders }
    }).collect()
}

pub fn history(database: &Database, days: i64) -> DbResult<Vec<HistoryDay>> {
    if ![1, 7, 30].contains(&days) { return Err("History range must be 1, 7, or 30 days".into()); }
    let today = Local::now().date_naive();
    let first = today - Duration::days(days - 1);
    let tasks = database.tasks_since(&first.to_string())?;
    // Two extra UTC days cover the oldest local day in every supported time zone.
    let since = (Utc::now() - Duration::days(days + 2)).to_rfc3339();
    let events = database.reminder_events_since(&since)?;
    Ok(calculate_history(today, days, tasks, events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn history_counts_tasks_reminders_and_filters_dates() {
        let today = Local::now().date_naive();
        let yesterday = today - Duration::days(1);
        let make_task = |date: NaiveDate, status: &str| Task {
            id: format!("{date}-{status}"), title: "Task".into(), task_date: date.to_string(),
            status: status.into(), sort_order: 0, created_at: String::new(), completed_at: None,
            origin_created_at: None, carried_to_date: None,
        };
        let local_noon = Local.from_local_datetime(&yesterday.and_hms_opt(12, 0, 0).unwrap()).single().unwrap();
        let event = |action: &str| ReminderEvent {
            id: action.into(), reminder_id: "eye".into(), scheduled_at: local_noon.with_timezone(&Utc).to_rfc3339(),
            triggered_at: None, action: action.into(), completed_at: None, snooze_count: 0,
        };
        let result = calculate_history(today, 7,
            vec![make_task(yesterday, "completed"), make_task(yesterday, "pending"), make_task(today - Duration::days(8), "completed")],
            vec![event("completed"), event("snoozed")]);
        let day = &result[1];
        assert_eq!((day.tasks_completed, day.tasks_total, day.task_completion_rate), (1, 2, 50));
        assert_eq!((day.reminders[0].completed, day.reminders[0].total, day.reminders[0].completion_rate), (1, 2, 50));
        assert_eq!(result.len(), 7);
        assert_eq!(result[0].tasks_total, 0);
    }
}
