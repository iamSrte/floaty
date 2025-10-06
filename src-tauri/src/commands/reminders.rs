use chrono::Utc;
use diesel::prelude::*;
use tauri::State;

use crate::db::models::{NewReminder, Reminder};
use crate::db::schema::reminders::dsl::*;
use crate::error::AppError;
use crate::services::scheduler::{ScheduleItem, ScheduleItemIdentifier, ScheduleItemType};
use crate::state::{get_state_guard, AppState};

#[tauri::command]
pub async fn create_reminder(
    state: State<'_, AppState>,
    new_reminder: NewReminder,
) -> Result<Reminder, AppError> {
    let reminder = {
        let mut state_guard = get_state_guard(&state)?;
        let conn = &mut state_guard.db_connection;
        diesel::insert_into(reminders)
            .values(&new_reminder)
            .get_result::<Reminder>(conn)?
    };

    if reminder.is_active == 1 {
        if reminder.trigger_time.and_utc() > Utc::now() {
            let scheduler = {
                let state_guard = get_state_guard(&state)?;
                state_guard.scheduler.clone()
            };

            scheduler
                .add_item(ScheduleItem::from(reminder.clone()))
                .await;
        }
    }

    Ok(reminder)
}

#[tauri::command]
pub fn get_all_reminders(
    state: State<AppState>,
    only_active: bool,
) -> Result<Vec<Reminder>, AppError> {
    let conn = &mut get_state_guard(&state)?.db_connection;

    let mut query = reminders.into_boxed();
    if only_active {
        query = query.filter(is_active.eq(1));
    }

    let results = query.load::<Reminder>(conn)?;
    Ok(results)
}

#[tauri::command]
pub async fn update_reminder(
    state: State<'_, AppState>,
    reminder_id: i32,
    updated_reminder: NewReminder,
) -> Result<Reminder, AppError> {
    let reminder = {
        let mut state_guard = get_state_guard(&state)?;
        let conn = &mut state_guard.db_connection;
        diesel::update(reminders.find(reminder_id))
            .set((
                title.eq(updated_reminder.title),
                body.eq(updated_reminder.body),
                trigger_time.eq(updated_reminder.trigger_time),
                is_recurring.eq(updated_reminder.is_recurring),
                recurrence_rule.eq(updated_reminder.recurrence_rule),
                is_active.eq(updated_reminder.is_active),
            ))
            .get_result::<Reminder>(conn)?
    };

    let identifier = ScheduleItemIdentifier::from(reminder.clone());
    let scheduler = {
        let state_guard = get_state_guard(&state)?;
        state_guard.scheduler.clone()
    };

    if reminder.is_active == 1 {
        let trigger_time_utc = reminder.trigger_time.and_utc();
        if trigger_time_utc > Utc::now() {
            let schedule_item = ScheduleItem::from(reminder.clone());
            scheduler.update_item(identifier, schedule_item).await;
        } else {
            scheduler.remove_item(identifier).await;
        }
    } else {
        scheduler.remove_item(identifier).await;
    }

    Ok(reminder)
}

#[tauri::command]
pub async fn delete_reminder(
    state: State<'_, AppState>,
    reminder_id: i32,
) -> Result<usize, AppError> {
    let count = {
        let mut state_guard = get_state_guard(&state)?;
        let conn = &mut state_guard.db_connection;
        diesel::delete(reminders.find(reminder_id)).execute(conn)?
    };

    let scheduler = {
        let state_guard = get_state_guard(&state)?;
        state_guard.scheduler.clone()
    };
    let identifier = ScheduleItemIdentifier {
        item_type: ScheduleItemType::Reminder,
        item_id: reminder_id,
    };
    scheduler.remove_item(identifier).await;

    Ok(count)
}
