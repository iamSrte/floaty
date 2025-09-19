use std::sync::Mutex;
use tauri::Manager;

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

            let app_state = state::AppStateInner {
                db_connection: connection,
            };
            app.manage(Mutex::new(app_state));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
