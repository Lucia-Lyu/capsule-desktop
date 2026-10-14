use chrono::{Days, Local};
use tauri::{AppHandle, Emitter, State, WebviewWindow};

use crate::{
    attention::Attention,
    database::Database,
    history,
    models::{HistoryDay, ReminderSnapshot, Task},
    scheduler::Scheduler,
    window,
};

#[tauri::command]
pub fn set_capsule_mode(window: WebviewWindow, mode: String, grouped: bool, attention: State<'_, Attention>, app: AppHandle) -> Result<(), String> {
    if mode == "collapsed" && attention.dismiss(&app)? { return Ok(()); }
    let (width, height) = match mode.as_str() {
        "collapsed" => (110, 32),
        "expanded" => (280, 360),
        "reminder" => (280, if grouped { 180 } else { 140 }),
        "history" | "settings" => (320, 440),
        _ => return Err("Unknown capsule mode".into()),
    };
    window::resize_capsule(&window, width, height).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_today_tasks(database: State<'_, Database>) -> Result<Vec<Task>, String> {
    database.today_tasks()
}

#[tauri::command]
pub fn get_capsule_theme(database: State<'_, Database>) -> Result<String, String> {
    database.capsule_theme()
}

#[tauri::command]
pub fn set_capsule_theme(theme: String, database: State<'_, Database>) -> Result<(), String> {
    database.set_capsule_theme(&theme)
}

#[tauri::command]
pub fn get_carry_candidates(database: State<'_, Database>) -> Result<Vec<Task>, String> {
    database.carry_candidates()
}

#[tauri::command]
pub fn carry_tasks(ids: Vec<String>, database: State<'_, Database>, app: AppHandle) -> Result<(), String> {
    let today = Local::now().date_naive().to_string();
    database.carry_tasks(&ids, &today)?;
    let _ = app.emit("task://updated", ());
    Ok(())
}

#[tauri::command]
pub fn defer_task_to_tomorrow(id: String, database: State<'_, Database>, app: AppHandle) -> Result<(), String> {
    let today = Local::now().date_naive();
    let tomorrow = today.checked_add_days(Days::new(1)).ok_or("Invalid next date")?;
    database.defer_task_to_tomorrow(&id, &today.to_string(), &tomorrow.to_string())?;
    let _ = app.emit("task://updated", ());
    Ok(())
}

#[tauri::command]
pub fn create_task(
    title: String,
    database: State<'_, Database>,
    app: AppHandle,
) -> Result<Task, String> {
    let today = Local::now().date_naive().to_string();
    let task = database.create_task(&title, &today)?;
    let _ = app.emit("task://updated", ());
    Ok(task)
}

#[tauri::command]
pub fn set_task_completed(
    id: String,
    completed: bool,
    database: State<'_, Database>,
    app: AppHandle,
) -> Result<(), String> {
    database.set_task_completed(&id, completed)?;
    let _ = app.emit("task://updated", ());
    Ok(())
}

#[tauri::command]
pub fn delete_task(
    id: String,
    database: State<'_, Database>,
    app: AppHandle,
) -> Result<(), String> {
    database.delete_task(&id)?;
    let _ = app.emit("task://updated", ());
    Ok(())
}

#[tauri::command]
pub fn move_task(
    id: String,
    direction: i64,
    database: State<'_, Database>,
    app: AppHandle,
) -> Result<(), String> {
    database.move_task(&id, direction)?;
    let _ = app.emit("task://updated", ());
    Ok(())
}

#[tauri::command]
pub fn get_reminder_snapshot(
    database: State<'_, Database>,
    scheduler: State<'_, Scheduler>,
    attention: State<'_, Attention>,
) -> Result<ReminderSnapshot, String> {
    let mut snapshot = scheduler.snapshot(&database)?;
    snapshot.attention_visible = attention.visible()?;
    snapshot.attention_event_ids = attention.visible_ids()?;
    Ok(snapshot)
}

#[tauri::command]
pub fn update_reminder_rule(
    id: String,
    enabled: bool,
    interval_minutes: i64,
    snooze_minutes: i64,
    database: State<'_, Database>,
    scheduler: State<'_, Scheduler>,
    attention: State<'_, Attention>,
    app: AppHandle,
) -> Result<(), String> {
    let rule = database.update_reminder_rule(&id, enabled, interval_minutes, snooze_minutes)?;
    scheduler.update_rule(&database, rule)?;
    attention.resolved(&app, &database.active_reminder_events()?.into_iter().map(|event| event.id).collect::<Vec<_>>())?;
    let _ = app.emit("reminder://updated", ());
    Ok(())
}

#[tauri::command]
pub fn update_quiet_hours(
    start: String,
    end: String,
    database: State<'_, Database>,
    scheduler: State<'_, Scheduler>,
    app: AppHandle,
) -> Result<(), String> {
    database.set_quiet_hours(&start, &end)?;
    scheduler.update_quiet_hours(&start, &end)?;
    let _ = app.emit("reminder://updated", ());
    Ok(())
}

#[tauri::command]
pub fn update_attention_settings(
    sound: bool,
    repeat: bool,
    database: State<'_, Database>,
    attention: State<'_, Attention>,
    app: AppHandle,
) -> Result<(), String> {
    database.set_attention_settings(sound, repeat)?;
    attention.update_settings(sound, repeat)?;
    let _ = app.emit("reminder://updated", ());
    Ok(())
}

#[tauri::command]
pub fn complete_reminders(
    ids: Vec<String>,
    database: State<'_, Database>,
    scheduler: State<'_, Scheduler>,
    attention: State<'_, Attention>,
    app: AppHandle,
) -> Result<(), String> {
    for rule_id in database.complete_reminders(&ids)? {
        let rule = database.reminder_rule(&rule_id)?;
        scheduler.complete(&rule_id, rule.interval_minutes)?;
    }
    attention.resolved(&app, &database.active_reminder_events()?.into_iter().map(|event| event.id).collect::<Vec<_>>())?;
    let _ = app.emit("reminder://updated", ());
    Ok(())
}

#[tauri::command]
pub fn snooze_reminders(
    ids: Vec<String>,
    database: State<'_, Database>,
    scheduler: State<'_, Scheduler>,
    attention: State<'_, Attention>,
    app: AppHandle,
) -> Result<(), String> {
    for (id, rule_id, until) in database.snooze_reminders(&ids)? {
        scheduler.snooze(&rule_id, &id, until)?;
    }
    attention.resolved(&app, &database.active_reminder_events()?.into_iter().map(|event| event.id).collect::<Vec<_>>())?;
    let _ = app.emit("reminder://updated", ());
    Ok(())
}

#[tauri::command]
pub fn get_history(days: i64, database: State<'_, Database>) -> Result<Vec<HistoryDay>, String> {
    history::history(&database, days)
}
