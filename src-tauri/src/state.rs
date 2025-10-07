use std::sync::Arc;
use tauri::State;

use crate::db::DbPool;
use crate::services::scheduler::Scheduler;

pub struct AppStateInner {
    pub scheduler: Scheduler,
    pub db_pool: DbPool,
}

pub type AppState = Arc<AppStateInner>;

pub fn get_state<'a>(
    state: &'a State<AppState>,
) -> &'a AppState {
    state.inner()
}
