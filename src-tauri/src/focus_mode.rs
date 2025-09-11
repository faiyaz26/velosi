use crate::{cache::CacheManager, AppState};
use tauri::{AppHandle, Emitter, Manager};

pub struct FocusMode {
    app_handle: AppHandle,
    cache_manager: CacheManager,
}

impl FocusMode {
    pub fn new(app_handle: AppHandle) -> Self {
        let cache_manager = CacheManager::new(app_handle.clone());
        Self {
            app_handle,
            cache_manager,
        }
    }

    pub async fn check_and_block_app(
        &self,
        app_name: &str,
        bundle_id: Option<&str>,
    ) -> Result<bool, String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        // Check if focus mode is enabled (use cache)
        let focus_enabled = {
            let focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
            *focus_enabled
        };

        if !focus_enabled {
            return Ok(true); // App is allowed
        }

        // Always allow velosi app itself
        if app_name.to_lowercase().contains("velosi")
            || bundle_id.map_or(false, |bid| bid.to_lowercase().contains("velosi"))
        {
            return Ok(true); // Velosi app is always allowed
        }

        // Check if app is temporarily allowed (use cache first)
        let is_allowed = self.cache_manager.is_app_allowed_cached(app_name).await?;

        println!(
            "🔍 Focus mode check for '{}': Cached allowed = {}",
            app_name, is_allowed
        );

        if is_allowed {
            println!("✅ App '{}' is allowed (cached)", app_name);
            return Ok(true); // App is allowed
        }

        // Get allowed categories from cache
        let allowed_categories = {
            let allowed_categories = state
                .focus_mode_allowed_categories
                .lock()
                .map_err(|e| e.to_string())?;
            allowed_categories.clone()
        };

        // If no categories are specified, block everything
        if allowed_categories.is_empty() {
            return self
                .block_app_with_notification(app_name, "No categories allowed in focus mode")
                .await;
        }

        // Get app mappings from cache
        let app_mappings = self.cache_manager.get_app_mappings_cached().await?;

        // Find the category for this app
        for mapping in app_mappings {
            let patterns: Vec<&str> = mapping.app_pattern.split('|').collect();
            for pattern in patterns {
                if app_name
                    .to_lowercase()
                    .contains(&pattern.trim().to_lowercase())
                    || bundle_id.map_or(false, |bid| {
                        bid.to_lowercase().contains(&pattern.trim().to_lowercase())
                    })
                {
                    // App matches this category
                    if allowed_categories.contains(&mapping.category_id) {
                        return Ok(true); // App is allowed
                    } else {
                        return self
                            .block_app_with_notification(
                                app_name,
                                &format!(
                                    "Category '{}' is not allowed in focus mode",
                                    mapping.category_id
                                ),
                            )
                            .await;
                    }
                }
            }
        }

        // App not found in mappings, block by default in focus mode
        self.block_app_with_notification(app_name, "App not categorized - blocked in focus mode")
            .await
    }

    async fn block_app_with_notification(
        &self,
        app_name: &str,
        reason: &str,
    ) -> Result<bool, String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        // Check if this app was recently blocked to prevent duplicate processing
        let now = tokio::time::Instant::now();
        {
            let mut blocked_apps = state
                .recently_blocked_apps
                .lock()
                .map_err(|e| e.to_string())?;

            // Clean up old entries (older than 30 seconds)
            blocked_apps
                .retain(|_, &mut last_blocked| now.duration_since(last_blocked).as_secs() < 30);

            // Check if this app was recently blocked (within last 10 seconds)
            if let Some(&last_blocked) = blocked_apps.get(app_name) {
                if now.duration_since(last_blocked).as_secs() < 10 {
                    return Ok(false); // App is blocked, but don't process again
                }
            }

            // Record that this app is being blocked now
            blocked_apps.insert(app_name.to_string(), now);
        }

        // Show notification and popup without hiding the app
        let app_name_clone = app_name.to_string();
        let reason_clone = reason.to_string();
        let focus_mode_clone = FocusMode::new(self.app_handle.clone());

        tokio::spawn(async move {
            // Just show notification - don't hide the app to avoid permission requests
            let _ = focus_mode_clone
                .show_block_notification(&app_name_clone, &reason_clone)
                .await;
        });

        Ok(false) // App is blocked
    }

    async fn show_block_notification(&self, app_name: &str, reason: &str) -> Result<(), String> {
        // Emit event to frontend for notification
        self.app_handle
            .emit(
                "app-blocked",
                serde_json::json!({
                    "app_name": app_name,
                    "reason": reason,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }),
            )
            .map_err(|e| e.to_string())?;

        // Show the overlay window instead of AppleScript popup
        crate::commands::show_focus_overlay(
            self.app_handle.clone(),
            app_name.to_string(),
            reason.to_string(),
        )
        .await?;

        Ok(())
    }
}
