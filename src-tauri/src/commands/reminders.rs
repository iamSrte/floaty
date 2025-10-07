use chrono::Utc;
use diesel::prelude::*;
use tauri::State;

use crate::db::models::{NewReminder, Reminder};
use crate::db::schema::reminders::dsl::*;
use crate::error::AppError;
use crate::services::scheduler::{ScheduleItem, ScheduleItemIdentifier, ScheduleItemType};
use crate::state::{get_state, AppState};

#[tauri::command]
pub async fn create_reminder(
    state: State<'_, AppState>,
    new_reminder: NewReminder,
) -> Result<Reminder, AppError> {
    let app_state = get_state(&state);
    let conn = app_state.db_pool.get().await?;

    let reminder = conn
        .interact(move |conn| {
            diesel::insert_into(reminders)
                .values(&new_reminder)
                .get_result::<Reminder>(conn)
        })
        .await??;

    if reminder.is_active == 1 && reminder.trigger_time.and_utc() > Utc::now() {
        app_state
            .scheduler
            .add_item(ScheduleItem::from(reminder.clone()))
            .await;
    }

    Ok(reminder)
}

#[tauri::command]
pub async fn get_all_reminders(
    state: State<'_, AppState>,
    only_active: bool,
) -> Result<Vec<Reminder>, AppError> {
    let app_state = get_state(&state);
    let conn = app_state.db_pool.get().await?;

    let mut query = reminders.into_boxed();
    if only_active {
        query = query.filter(is_active.eq(1));
    }

    let result = conn
        .interact(move |conn| query.load::<Reminder>(conn))
        .await??;

    Ok(result)
}

#[tauri::command]
pub async fn update_reminder(
    state: State<'_, AppState>,
    reminder_id: i32,
    updated_reminder: NewReminder,
) -> Result<Reminder, AppError> {
    let app_state = get_state(&state);
    let conn = app_state.db_pool.get().await?;

    let reminder = conn
        .interact(move |conn| {
            diesel::update(reminders.find(reminder_id))
                .set((
                    title.eq(updated_reminder.title),
                    body.eq(updated_reminder.body),
                    trigger_time.eq(updated_reminder.trigger_time),
                    is_recurring.eq(updated_reminder.is_recurring),
                    recurrence_rule.eq(updated_reminder.recurrence_rule),
                    is_active.eq(updated_reminder.is_active),
                ))
                .get_result::<Reminder>(conn)
        })
        .await??;

    let identifier = ScheduleItemIdentifier::from(reminder.clone());

    if reminder.is_active == 1 {
        if reminder.trigger_time.and_utc() > Utc::now() {
            let schedule_item = ScheduleItem::from(reminder.clone());
            app_state
                .scheduler
                .update_item(identifier, schedule_item)
                .await;
        } else {
            app_state.scheduler.remove_item(identifier).await;
        }
    } else {
        app_state.scheduler.remove_item(identifier).await;
    }

    Ok(reminder)
}

#[tauri::command]
pub async fn delete_reminder(state: State<'_, AppState>, reminder_id: i32) -> Result<(), AppError> {
    let app_state = get_state(&state);
    let conn = app_state.db_pool.get().await?;

    conn.interact(move |conn| diesel::delete(reminders.find(reminder_id)).execute(conn))
        .await??;

    let identifier = ScheduleItemIdentifier {
        item_type: ScheduleItemType::Reminder,
        item_id: reminder_id,
    };
    app_state.scheduler.remove_item(identifier).await;

    Ok(())
}
