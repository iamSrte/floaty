use std::cmp::Ordering;
use std::sync::Arc;
use std::time::Duration;

use log::error;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, Notify};

use chrono::{DateTime, Utc};
use min_heap::MinHeap;

use crate::commands::reminders::get_all_reminders;
use crate::db::models::Reminder;
use crate::db::DbPool;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleItemType {
    Reminder,
    Timer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleItemIdentifier {
    pub item_type: ScheduleItemType,
    pub item_id: i32,
}

impl From<Reminder> for ScheduleItemIdentifier {
    fn from(reminder: Reminder) -> Self {
        ScheduleItemIdentifier {
            item_type: ScheduleItemType::Reminder,
            item_id: reminder.id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduleItem {
    pub identifier: ScheduleItemIdentifier,
    pub trigger_time: DateTime<Utc>,
}

impl Ord for ScheduleItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.trigger_time
            .cmp(&other.trigger_time)
            .then_with(|| self.identifier.item_id.cmp(&other.identifier.item_id))
    }
}

impl PartialOrd for ScheduleItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Reminder> for ScheduleItem {
    fn from(reminder: Reminder) -> Self {
        ScheduleItem {
            identifier: ScheduleItemIdentifier {
                item_type: ScheduleItemType::Reminder,
                item_id: reminder.id,
            },
            trigger_time: reminder.trigger_time.and_utc(),
        }
    }
}

#[derive(Clone)]
pub struct Scheduler {
    heap: Arc<Mutex<MinHeap<ScheduleItem>>>,
    app_handle: AppHandle,
    wakeup_notify: Arc<Notify>,
    scheduler_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl Scheduler {
    pub fn new(app_handle: &AppHandle) -> Self {
        let mut scheduler = Self {
            heap: Arc::new(Mutex::new(MinHeap::new())),
            app_handle: app_handle.clone(),
            wakeup_notify: Arc::new(Notify::new()),
            scheduler_task: Arc::new(Mutex::new(None)),
        };

        // Start the main scheduler task
        scheduler.scheduler_task = Arc::new(Mutex::new(Some(scheduler.spawn_task())));
        scheduler
    }

    fn spawn_task(&self) -> JoinHandle<()> {
        let heap = Arc::clone(&self.heap);
        let notify = Arc::clone(&self.wakeup_notify);
        let app_handle = self.app_handle.clone();

        tauri::async_runtime::spawn(async move {
            loop {
                let sleep_duration = {
                    let heap_guard = heap.lock().await;

                    if let Some(next_item) = heap_guard.peek() {
                        let now = Utc::now();

                        if next_item.trigger_time <= now {
                            Duration::from_secs(0)
                        } else {
                            let duration = next_item.trigger_time - now;
                            duration.to_std().unwrap_or(Duration::from_secs(0))
                        }
                    } else {
                        // No items in heap, sleep for a while
                        Duration::from_secs(3600) // 1 hour
                    }
                };

                // Sleep until next item or until interrupted
                tokio::select! {
                    _ = tokio::time::sleep(sleep_duration) => {
                        trigger_due_items(&heap, &app_handle).await;
                    }
                    _ = notify.notified() => {
                        continue;
                    }
                }
            }
        })
    }

    pub async fn add_item(&self, item: ScheduleItem) {
        let mut heap_guard = self.heap.lock().await;
        heap_guard.push(item);
        drop(heap_guard);

        self.wakeup_notify.notify_one();
    }

    /// Remove an item from the schedule
    pub async fn remove_item(&self, identifier: ScheduleItemIdentifier) {
        let mut heap_guard = self.heap.lock().await;

        heap_guard.retain(|item| item.identifier != identifier);
        drop(heap_guard);

        self.wakeup_notify.notify_one();
    }

    /// Update an existing item's schedule
    pub async fn update_item(&self, identifier: ScheduleItemIdentifier, new_item: ScheduleItem) {
        let mut heap_guard = self.heap.lock().await;

        heap_guard.retain(|item| item.identifier != identifier);
        heap_guard.push(new_item);
        drop(heap_guard);

        self.wakeup_notify.notify_one();
    }

    /// Get current schedule status (for debugging)
    pub async fn get_schedule_status(&self) -> (usize, Option<DateTime<Utc>>) {
        let heap_guard = self.heap.lock().await;
        let count = heap_guard.len();
        let next_trigger = heap_guard.peek().map(|item| item.trigger_time);
        (count, next_trigger)
    }

    pub async fn reload_from_db(&self, db_pool: DbPool) -> Result<usize, AppError> {
        let heap = Arc::clone(&self.heap);

        let conn = db_pool.get().await?;

        // Temporarily untill i add a queries module
        use diesel::prelude::*;
        use crate::db::schema::reminders::dsl::*;
        let active_reminders = conn
            .interact(|conn| {
                reminders
                    .filter(is_active.eq(1))
                    .select(Reminder::as_select())
                    .load::<Reminder>(conn)
            })
            .await??;

        let now = Utc::now();
        let schedule_items: Vec<ScheduleItem> = active_reminders
            .into_iter()
            .filter_map(|reminder| {
                let trigger_time_utc = reminder.trigger_time.and_utc();
                if trigger_time_utc > now {
                    Some(ScheduleItem::from(reminder))
                } else {
                    None
                }
            })
            .collect();

        let count = schedule_items.len();

        let mut heap_guard = heap.lock().await;
        heap_guard.clear();

        for item in schedule_items {
            heap_guard.push(item);
        }
        drop(heap_guard);

        self.wakeup_notify.notify_one();
        Ok(count)
    }
}

async fn trigger_due_items(heap: &Arc<Mutex<MinHeap<ScheduleItem>>>, app_handle: &AppHandle) {
    let now = Utc::now();
    let mut heap_guard = heap.lock().await;

    // Process all due items
    while let Some(next_item) = heap_guard.peek() {
        if next_item.trigger_time <= now {
            let item = heap_guard.pop().unwrap();
            drop(heap_guard);
            trigger_item(&item, app_handle).await;
            heap_guard = heap.lock().await;
        } else {
            break;
        }
    }
}

async fn trigger_item(item: &ScheduleItem, app_handle: &AppHandle) {
    match item.identifier.item_type {
        ScheduleItemType::Reminder => {
            // Emit event to frontend
            let _ = app_handle.emit("reminder_triggered", &item.identifier.item_id);
            println!("🔔 Reminder triggered (ID: {})", item.identifier.item_id);
        }
        ScheduleItemType::Timer => {
            // Emit event to frontend
            let _ = app_handle.emit("timer_completed", &item.identifier.item_id);
            println!("⏰ Timer completed (ID: {})", item.identifier.item_id);
        }
    }
}
