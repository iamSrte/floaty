use std::cmp::Ordering;
use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, Notify};
use tokio::task::JoinHandle;

use chrono::{DateTime, Utc};
use min_heap::MinHeap;

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

pub struct Scheduler {
    heap: Arc<Mutex<MinHeap<ScheduleItem>>>,
    app_handle: AppHandle,
    wakeup_notify: Arc<Notify>,
    scheduler_task: Option<JoinHandle<()>>,
}

impl Scheduler {
    pub fn new(app_handle: AppHandle) -> Self {
        let mut scheduler = Self {
            heap: Arc::new(Mutex::new(MinHeap::new())),
            app_handle: app_handle.clone(),
            wakeup_notify: Arc::new(Notify::new()),
            scheduler_task: None,
        };

        // Start the main scheduler task
        scheduler.scheduler_task = Some(scheduler.spawn_scheduler_task());
        scheduler
    }

    fn spawn_scheduler_task(&self) -> JoinHandle<()> {
        let heap = Arc::clone(&self.heap);
        let notify = Arc::clone(&self.wakeup_notify);
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            loop {
                // Get next item and calculate sleep duration
                let sleep_duration = {
                    let heap_guard = heap.lock().await;

                    if let Some(next_item) = heap_guard.peek() {
                        let now = Utc::now();

                        if next_item.trigger_time <= now {
                            tokio::time::Duration::from_secs(0)
                        } else {
                            let duration = next_item.trigger_time - now;
                            duration
                                .to_std()
                                .unwrap_or(tokio::time::Duration::from_secs(0))
                        }
                    } else {
                        // No items in heap, sleep for a while
                        tokio::time::Duration::from_secs(3600) // 1 hour
                    }
                };

                // Sleep until next item or until interrupted
                tokio::select! {
                    _ = tokio::time::sleep(sleep_duration) => {
                        Self::trigger_due_items(&heap, &app_handle).await;
                    }
                    _ = notify.notified() => {
                        continue;
                    }
                }
            }
        })
    }

    async fn trigger_due_items(heap: &Arc<Mutex<MinHeap<ScheduleItem>>>, app_handle: &AppHandle) {
        let now = Utc::now();
        let mut heap_guard = heap.lock().await;

        // Process all due items
        while let Some(next_item) = heap_guard.peek() {
            if next_item.trigger_time <= now {
                let item = heap_guard.pop().unwrap();
                drop(heap_guard);
                Self::trigger_item(&item, app_handle).await;
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

                // Show system notification
                Self::show_notification(app_handle, "Reminder").await;

                println!("🔔 Reminder triggered (ID: {})", item.identifier.item_id);
            }
            ScheduleItemType::Timer => {
                // Emit event to frontend
                let _ = app_handle.emit("timer_completed", &item.identifier.item_id);

                // Show system notification
                Self::show_notification(app_handle, "Timer Completed").await;

                println!("⏰ Timer completed (ID: {})", item.identifier.item_id);
            }
        }
    }

    async fn show_notification(app_handle: &AppHandle, title: &str) {
        // TODO: Implement with tauri-plugin-notification
        //       Probably need to move to another module
    }

    pub async fn add_schedule(&self, item: ScheduleItem) {
        {
            let mut heap_guard = self.heap.lock().await;
            heap_guard.push(item);
        }

        // Wake up the scheduler to recalculate
        self.wakeup_notify.notify_one();
    }

    /// Remove an item from the schedule
    pub async fn remove_schedule(&self, identifier: ScheduleItemIdentifier) {
        {
            let mut heap_guard = self.heap.lock().await;

            // Rebuild heap without the target item
            let filtered_items: MinHeap<ScheduleItem> = heap_guard
                .drain()
                .filter(|item| !(item.identifier == identifier))
                .collect();

            *heap_guard = filtered_items;
        }

        // Wake up the scheduler
        self.wakeup_notify.notify_one();
    }

    /// Update an existing item's schedule
    pub async fn update_schedule(
        &self,
        old_identifier: ScheduleItemIdentifier,
        new_item: ScheduleItem,
    ) {
        // Remove old item
        self.remove_schedule(old_identifier).await;

        // Add updated item
        self.add_schedule(new_item).await;
    }

    /// Convenience method for updating schedule with separate parameters
    pub async fn update_schedule_by_params(
        &self,
        item_type: ScheduleItemType,
        item_id: i32,
        new_trigger_time: DateTime<Utc>,
        new_title: String,
    ) {
        let old_identifier = ScheduleItemIdentifier {
            item_type: item_type.clone(),
            item_id,
        };
        let new_identifier = ScheduleItemIdentifier { item_type, item_id };
        let new_item = ScheduleItem {
            identifier: new_identifier,
            trigger_time: new_trigger_time,
        };

        self.update_schedule(old_identifier, new_item).await;
    }

    /// Get current schedule status (for debugging)
    pub async fn get_schedule_status(&self) -> (usize, Option<DateTime<Utc>>) {
        let heap_guard = self.heap.lock().await;
        let count = heap_guard.len();
        let next_trigger = heap_guard.peek().map(|item| item.trigger_time);
        (count, next_trigger)
    }
}
