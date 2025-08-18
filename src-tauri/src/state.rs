use std::sync::Mutex;
use diesel::sqlite::SqliteConnection;

pub struct AppStateInner {
    pub db_connection: SqliteConnection,
}

pub type AppState = Mutex<AppStateInner>;   