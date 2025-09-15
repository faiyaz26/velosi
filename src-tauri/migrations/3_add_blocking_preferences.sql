-- Migration: 3_add_blocking_preferences.sql
-- Adds blocking preferences to focus_mode_settings table

BEGIN TRANSACTION;

-- Add blocking preferences to the focus_mode_settings table
INSERT OR IGNORE INTO focus_mode_settings (key, value) VALUES ('app_blocking_enabled', '1');
INSERT OR IGNORE INTO focus_mode_settings (key, value) VALUES ('website_blocking_enabled', '1');

COMMIT;
