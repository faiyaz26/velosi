-- Migration 5: Create pomodoro_sessions table for tracking pomodoro timer sessions

CREATE TABLE IF NOT EXISTS pomodoro_sessions (
    id TEXT PRIMARY KEY,
    session_type TEXT NOT NULL, -- 'work' or 'break'
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration_minutes INTEGER NOT NULL, -- planned duration
    actual_duration_seconds INTEGER, -- actual time spent (for tracking interruptions)
    work_description TEXT, -- optional description of what user was working on
    completed INTEGER NOT NULL DEFAULT 0, -- 1 if session was completed, 0 if interrupted
    focus_mode_enabled INTEGER NOT NULL DEFAULT 0, -- 1 if focus mode was enabled
    app_tracking_enabled INTEGER NOT NULL DEFAULT 0 -- 1 if app tracking was enabled
);

CREATE TABLE IF NOT EXISTS pomodoro_settings (
    id TEXT PRIMARY KEY DEFAULT 'default',
    work_duration_minutes INTEGER NOT NULL DEFAULT 25,
    break_duration_minutes INTEGER NOT NULL DEFAULT 5,
    enable_focus_mode INTEGER NOT NULL DEFAULT 0,
    enable_app_tracking INTEGER NOT NULL DEFAULT 0,
    auto_start_breaks INTEGER NOT NULL DEFAULT 1,
    auto_start_work INTEGER NOT NULL DEFAULT 1,
    updated_at TEXT NOT NULL
);

-- Insert default settings
INSERT OR IGNORE INTO pomodoro_settings (id, work_duration_minutes, break_duration_minutes, enable_focus_mode, enable_app_tracking, auto_start_breaks, auto_start_work, updated_at)
VALUES ('default', 25, 5, 0, 0, 1, 1, datetime('now'));

CREATE INDEX IF NOT EXISTS idx_pomodoro_sessions_start_time ON pomodoro_sessions(start_time);
CREATE INDEX IF NOT EXISTS idx_pomodoro_sessions_session_type ON pomodoro_sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_pomodoro_sessions_completed ON pomodoro_sessions(completed);