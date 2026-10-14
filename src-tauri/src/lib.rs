mod attention;
mod commands;
mod database;
mod history;
mod models;
mod reminder;
mod scheduler;
mod window;

use tauri::Manager;


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let database = database::Database::open(&data_dir.join("capsule.sqlite3"))
                .map_err(std::io::Error::other)?;
            let scheduler = scheduler::Scheduler::new(&database).map_err(std::io::Error::other)?;
            let attention = attention::Attention::new(&database).map_err(std::io::Error::other)?;
            app.manage(database);
            app.manage(scheduler);
            app.manage(attention);
            let capsule = app
                .get_webview_window("main")
                .expect("capsule window is missing");
            window::position_initial(&capsule)?;
            capsule.show()?;
            scheduler::Scheduler::start(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::set_capsule_mode,
            commands::get_today_tasks,
            commands::get_capsule_theme,
            commands::set_capsule_theme,
            commands::get_carry_candidates,
            commands::carry_tasks,
            commands::defer_task_to_tomorrow,
            commands::create_task,
            commands::set_task_completed,
            commands::delete_task,
            commands::move_task,
            commands::get_reminder_snapshot,
            commands::update_reminder_rule,
            commands::update_quiet_hours,
            commands::update_attention_settings,
            commands::complete_reminders,
            commands::snooze_reminders,
            commands::get_history
        ])
        .run(tauri::generate_context!())
        .expect("error while running Capsule");
}
