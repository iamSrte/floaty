// @generated automatically by Diesel CLI.

diesel::table! {
    floats (id) {
        id -> Integer,
        label -> Nullable<Text>,
        timer_id -> Nullable<Integer>,
        note_content -> Nullable<Text>,
        window_x -> Integer,
        window_y -> Integer,
        window_width -> Integer,
        window_height -> Integer,
        is_visible -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    reminders (id) {
        id -> Integer,
        title -> Text,
        body -> Nullable<Text>,
        trigger_time -> Timestamp,
        is_recurring -> Integer,
        recurrence_rule -> Nullable<Text>,
        is_active -> Integer,
        created_at -> Timestamp,
    }
}

diesel::table! {
    timers (id) {
        id -> Integer,
        label -> Nullable<Text>,
        duration_seconds -> Integer,
        timer_type -> Text,
        start_time -> Nullable<Timestamp>,
        end_time -> Nullable<Timestamp>,
        status -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(floats -> timers (timer_id));

diesel::allow_tables_to_appear_in_same_query!(
    floats,
    reminders,
    timers,
);
