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

            // Initialize the database pool
            let db_pool = db::create_pool();

            // Run migrations
            let pool_for_migration = db_pool.clone();
            tauri::async_runtime::block_on(async {
                let conn = pool_for_migration
                    .get()
                    .await
                    .expect("Failed to get connection for migrations");

                conn.interact(|conn| {
                    db::run_migrations(conn).expect("Failed to run database migrations")
                })
                .await
                .expect("Failed to interact with database for migrations");
            });

            // Initialize the scheduler
            let scheduler = Scheduler::new(app.handle());

            // Store the app state
            let app_state = state::AppStateInner {
                scheduler: scheduler.clone(),
                db_pool: db_pool.clone(),
            };
            app.manage(app_state);

            // Reload scheduler from database
            tauri::async_runtime::spawn(async move {
                match scheduler.reload_from_db(db_pool).await {
                    Ok(count) => {
                        log::info!("Successfully loaded {} scheduled items", count);
                    }
                    Err(e) => {
                        log::error!("Failed to reload scheduler from database: {}", e);
                    }
                }
            });

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
