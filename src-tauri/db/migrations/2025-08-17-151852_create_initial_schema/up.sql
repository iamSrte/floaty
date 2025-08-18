
CREATE TABLE reminders (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    trigger_time TEXT NOT NULL,
    is_recurring INTEGER NOT NULL DEFAULT 0,
    recurrence_rule TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE timers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    label TEXT,
    duration_seconds INTEGER NOT NULL,
    timer_type TEXT NOT NULL,
    start_time TEXT,
    end_time TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE floats (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    label TEXT,
    timer_id INTEGER,
    note_content TEXT,
    window_x INTEGER NOT NULL DEFAULT 100,
    window_y INTEGER NOT NULL DEFAULT 100,
    window_width INTEGER NOT NULL DEFAULT 300,
    window_height INTEGER NOT NULL DEFAULT 150,
    is_visible INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (timer_id) REFERENCES timers (id) ON DELETE SET NULL
);
