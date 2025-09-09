-- Migration 1: Create initial tables for velosi tracker

CREATE TABLE IF NOT EXISTS activity_entries (
    id TEXT PRIMARY KEY,
    start_time TEXT NOT NULL,
    end_time TEXT,
    app_name TEXT NOT NULL,
    app_bundle_id TEXT,
    window_title TEXT NOT NULL,
    url TEXT,
    category TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS activity_segments (
    id TEXT PRIMARY KEY,
    activity_id TEXT NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT,
    segment_type TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT,
    file_path TEXT,
    metadata TEXT,
    FOREIGN KEY (activity_id) REFERENCES activity_entries (id)
);

CREATE TABLE IF NOT EXISTS user_categories (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    parent_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES user_categories (id)
);

CREATE TABLE IF NOT EXISTS app_mappings (
    id TEXT PRIMARY KEY,
    app_pattern TEXT NOT NULL,
    category_id TEXT NOT NULL,
    is_custom INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS url_mappings (
    id TEXT PRIMARY KEY,
    url_pattern TEXT NOT NULL,
    category_id TEXT NOT NULL,
    is_custom INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_start_time ON activity_entries(start_time);
CREATE INDEX IF NOT EXISTS idx_app_name ON activity_entries(app_name);
CREATE INDEX IF NOT EXISTS idx_category ON activity_entries(category);
CREATE INDEX IF NOT EXISTS idx_segment_activity_id ON activity_segments(activity_id);
CREATE INDEX IF NOT EXISTS idx_segment_start_time ON activity_segments(start_time);
CREATE INDEX IF NOT EXISTS idx_segment_type ON activity_segments(segment_type);
CREATE INDEX IF NOT EXISTS idx_user_categories_parent ON user_categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_app_mappings_pattern ON app_mappings(app_pattern);
CREATE INDEX IF NOT EXISTS idx_app_mappings_category ON app_mappings(category_id);
