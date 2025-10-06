use diesel::sqlite::SqliteConnection;
use std::sync::{Mutex, MutexGuard};
use tauri::State;

use crate::error::AppError;
use crate::services::scheduler::Scheduler;

pub struct AppStateInner {
    pub scheduler: Scheduler,
    pub db_connection: SqliteConnection,
}

pub type AppState = Mutex<AppStateInner>;

/// Returns a guard for the application's state mutex with custom error type.
/// On failure returns a `AppError::LockError`.
pub fn get_state_guard<'a>(
    state: &'a State<AppState>,
) -> Result<MutexGuard<'a, AppStateInner>, AppError> {
    let guard = state.lock().map_err(|_| AppError::LockError)?;
    Ok(guard)
}
