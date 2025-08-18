use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use crate::db::schema::{reminders, timers, floats};


// --- Reminder Models ---

#[derive(Queryable, Selectable)]
#[diesel(table_name = reminders)]
#[diesel(check_for_backend(Sqlite))]
pub struct Reminder {
    pub id: i32,
    pub title: String,
    pub body: Option<String>,
    pub trigger_time: String,
    pub is_recurring: i32,
    pub recurrence_rule: Option<String>,
    pub is_active: i32,
    pub created_at: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = reminders)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewReminder {
    pub title: String,
    pub body: Option<String>,
    pub trigger_time: String,
    pub is_recurring: i32,
    pub recurrence_rule: Option<String>,
    pub is_active: i32,
}


// --- Timer Models ---

#[derive(Queryable, Selectable)]
#[diesel(table_name = timers)]
#[diesel(check_for_backend(Sqlite))]
pub struct Timer {
    pub id: i32,
    pub label: Option<String>,
    pub duration_seconds: i32,
    pub timer_type: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = timers)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewTimer {
    pub label: Option<String>,
    pub duration_seconds: i32,
    pub timer_type: String,
    pub status: String,
}


// --- Float Models ---

#[derive(Queryable, Selectable)]
#[diesel(table_name = floats)]
#[diesel(check_for_backend(Sqlite))]
pub struct Float {
    pub id: i32,
    pub label: Option<String>,
    pub timer_id: Option<i32>,
    pub note_content: Option<String>,
    pub window_x: i32,
    pub window_y: i32,
    pub window_width: i32,
    pub window_height: i32,
    pub is_visible: i32,
    pub created_at: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = floats)]
#[diesel(check_for_backend(Sqlite))]
pub struct NewFloat {
    pub label: Option<String>,
    pub timer_id: Option<i32>,
    pub note_content: Option<String>,
    pub window_x: i32,
    pub window_y: i32,
    pub window_width: i32,
    pub window_height: i32,
    pub is_visible: i32,
}
