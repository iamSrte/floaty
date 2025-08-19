use crate::db::models::{NewReminder, Reminder};
use crate::db::schema::reminders::dsl::*;
use crate::error::AppError;
use crate::state::{get_state_guard, AppState};
use diesel::prelude::*;
use tauri::State;

#[tauri::command]
pub fn create_reminder(
    state: State<AppState>,
    new_reminder: NewReminder,
) -> Result<Reminder, AppError> {
    let conn = &mut get_state_guard(&state)?.db_connection;
    let reminder = diesel::insert_into(reminders)
        .values(&new_reminder)
        .get_result::<Reminder>(conn)?;

    Ok(reminder)
}

#[tauri::command]
pub fn get_all_reminders(state: State<AppState>) -> Result<Vec<Reminder>, AppError> {
    let conn = &mut get_state_guard(&state)?.db_connection;
    let results = reminders.load::<Reminder>(conn)?;
    Ok(results)
}

#[tauri::command]
pub fn update_reminder(
    state: State<AppState>,
    reminder_id: i32,
    updated_reminder: NewReminder,
) -> Result<Reminder, AppError> {
    let conn = &mut get_state_guard(&state)?.db_connection;
    let reminder = diesel::update(reminders.find(reminder_id))
        .set((
            title.eq(updated_reminder.title),
            body.eq(updated_reminder.body),
            trigger_time.eq(updated_reminder.trigger_time),
            is_recurring.eq(updated_reminder.is_recurring),
            recurrence_rule.eq(updated_reminder.recurrence_rule),
            is_active.eq(updated_reminder.is_active),
        ))
        .get_result::<Reminder>(conn)?;
    Ok(reminder)
}

#[tauri::command]
pub fn delete_reminder(state: State<AppState>, reminder_id: i32) -> Result<usize, AppError> {
    let conn = &mut get_state_guard(&state)?.db_connection;
    let count = diesel::delete(reminders.find(reminder_id)).execute(conn)?;
    Ok(count)
}
