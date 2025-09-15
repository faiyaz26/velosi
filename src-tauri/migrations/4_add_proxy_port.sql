-- Migration: 4_add_proxy_port.sql
-- Adds proxy port setting to focus_mode_settings table

BEGIN TRANSACTION;

-- Add proxy port setting with default value
INSERT OR IGNORE INTO focus_mode_settings (key, value) VALUES ('proxy_port', '62828');

COMMIT;
