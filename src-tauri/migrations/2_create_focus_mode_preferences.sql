-- Migration: 2_create_focus_mode_preferences.sql
-- Adds tables to persist Focus Mode preferences: settings, allowed categories, and allowed apps

BEGIN TRANSACTION;

-- Simple key/value settings table for small flags like whether focus mode is enabled
CREATE TABLE IF NOT EXISTS focus_mode_settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Store which category ids the user has allowed while focus mode is enabled
CREATE TABLE IF NOT EXISTS focus_mode_allowed_categories (
  category_id TEXT PRIMARY KEY
);

-- Store allowed applications (by app name or pattern). An optional expires_at (unix epoch seconds)
-- can be used for temporary allowances (e.g. 30 minutes). If expires_at is NULL, allowance is permanent.
CREATE TABLE IF NOT EXISTS focus_mode_allowed_apps (
  app_pattern TEXT PRIMARY KEY,
  expires_at INTEGER
);

-- Insert default value for enabled flag if not present
INSERT OR IGNORE INTO focus_mode_settings (key, value) VALUES ('enabled', '0');

COMMIT;
