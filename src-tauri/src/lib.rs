use std::sync::Mutex;
use tauri::Manager;

use crate::services::scheduler::Scheduler;

mod commands;
mod db;
mod error;
mod services;
mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let mut connection = db::establish_connection();
            db::run_migrations(&mut connection).expect("Failed to run database migrations.");

            let scheduler = Scheduler::new(app.handle());
            scheduler.reload_from_db();

            let app_state = state::AppStateInner {
                scheduler: scheduler,
                db_connection: connection,
            };
            app.manage(Mutex::new(app_state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::reminders::create_reminder,
            commands::reminders::update_reminder,
            commands::reminders::delete_reminder,
            commands::reminders::get_all_reminders,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
