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

    pub async fn cleanup_expired_apps(&self) -> Result<(), String> {
        let state: tauri::State<AppState> = self.app_handle.state();
        state
            .db
            .cleanup_expired_focus_mode_apps()
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
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

    #[cfg(target_os = "macos")]
    async fn show_blocking_popup(&self, app_name: &str, reason: &str) -> Result<(), String> {
        use std::process::Command;

        let script = format!(
            r#"
            tell application "System Events"
                activate
                set userChoice to display dialog "🛡️ Velsoi - Focus Mode: {} Blocked

{}

Would you like to:" buttons {{"Stay Focused", "Disable Focus Mode", "Allow This App"}} default button "Stay Focused" with title "Velosi - Focus Mode" with icon caution giving up after 8
                
                if gave up of userChoice then
                    return "timeout"
                else if button returned of userChoice is "Disable Focus Mode" then
                    return "disable_focus"
                else if button returned of userChoice is "Allow This App" then
                    return "allow_app"
                else
                    return "stay_focused"
                end if
            end tell
            "#,
            app_name, reason
        );

        let output = Command::new("osascript").arg("-e").arg(&script).output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    let response_raw = String::from_utf8_lossy(&result.stdout);
                    let response = response_raw.trim();
                    println!("✅ User chose: {}", response);

                    match response {
                        "disable_focus" => {
                            // Disable focus mode
                            if let Err(e) = self.disable_focus_mode().await {
                                println!("⚠️ Failed to disable focus mode: {}", e);
                            }
                        }
                        "allow_app" => {
                            // Temporarily allow this app
                            if let Err(e) = self.temporarily_allow_app(app_name).await {
                                println!("⚠️ Failed to temporarily allow app: {}", e);
                            } else {
                                // Show a notification that the app is now allowed
                                #[cfg(target_os = "macos")]
                                {
                                    let notification_script = format!(
                                        r#"display notification "App temporarily allowed for 30 minutes" with title "Focus Mode" subtitle "{}""#,
                                        app_name
                                    );
                                    let _ = std::process::Command::new("osascript")
                                        .arg("-e")
                                        .arg(&notification_script)
                                        .output();
                                }
                            }
                        }
                        "timeout" => {
                            println!("⏰ Dialog timed out - staying in focus mode");
                        }
                        _ => {
                            println!("👍 User chose to stay focused");
                        }
                    }
                } else {
                    println!(
                        "⚠️ Failed to show blocking dialog: {:?}",
                        String::from_utf8_lossy(&result.stderr)
                    );
                }
            }
            Err(e) => {
                println!("⚠️ Error showing blocking dialog: {}", e);
            }
        }

        Ok(())
    }

    async fn temporarily_allow_app(&self, app_name: &str) -> Result<(), String> {
        let state: tauri::State<crate::AppState> = self.app_handle.state();

        // Persist to database (expires in 30 minutes)
        let expires_at = chrono::Utc::now().timestamp() + (30 * 60); // 30 minutes from now
        state
            .db
            .add_focus_mode_allowed_app(app_name, Some(expires_at))
            .await
            .map_err(|e| e.to_string())?;

        // Update cache immediately
        {
            let mut cache = state
                .focus_mode_allowed_apps_cache
                .lock()
                .map_err(|e| e.to_string())?;
            cache.insert(app_name.to_string(), Some(expires_at));
        }

        // Emit event to frontend to refresh allowed apps list
        self.app_handle
            .emit(
                "app-temporarily-allowed",
                serde_json::json!({
                    "app_name": app_name,
                    "expires_at": expires_at,
                }),
            )
            .map_err(|e| e.to_string())?;

        println!(
            "✅ Temporarily allowed {} for 30 minutes (cached)",
            app_name
        );
        Ok(())
    }

    async fn disable_focus_mode(&self) -> Result<(), String> {
        // Call the disable focus mode command
        let state: tauri::State<crate::AppState> = self.app_handle.state();
        crate::commands::disable_focus_mode(state, self.app_handle.clone()).await
    }

    #[cfg(target_os = "macos")]
    async fn show_macos_notification(&self, app_name: &str, reason: &str) -> Result<(), String> {
        use std::process::Command;

        let script = format!(
            r#"
            display notification "{}" with title "🛡️ Focus Mode Active" subtitle "Blocked: {}" sound name "Basso"
            "#,
            reason, app_name
        );

        let output = Command::new("osascript").arg("-e").arg(&script).output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ Showed focus mode notification for {}", app_name);
                } else {
                    println!(
                        "⚠️ Failed to show notification: {:?}",
                        String::from_utf8_lossy(&result.stderr)
                    );
                }
            }
            Err(e) => {
                println!("⚠️ Error showing notification: {}", e);
            }
        }

        Ok(())
    }

    pub async fn hide_blocked_app(&self, app_name: &str) -> Result<(), String> {
        // Record that this app was recently hidden
        let state: tauri::State<crate::AppState> = self.app_handle.state();
        {
            let mut hidden_apps = state
                .recently_hidden_apps
                .lock()
                .map_err(|e| e.to_string())?;
            hidden_apps.insert(app_name.to_string(), tokio::time::Instant::now());
        }

        #[cfg(target_os = "macos")]
        {
            self.hide_macos_app(app_name).await
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(())
        }
    }

    #[cfg(target_os = "macos")]
    async fn hide_macos_app(&self, app_name: &str) -> Result<(), String> {
        use std::process::Command;

        let app_name_owned = app_name.to_string();

        // Use a faster, simpler approach to hide the app
        let hide_script = format!(
            r#"tell application "System Events" to tell process "{}" to set value of attribute "AXMinimized" of window 1 to true"#,
            app_name_owned
        );

        // Run asynchronously without waiting for full completion
        tokio::spawn(async move {
            let output = Command::new("osascript")
                .arg("-e")
                .arg(&hide_script)
                .output();

            if let Ok(result) = output {
                if result.status.success() {
                    println!("✅ Successfully minimized app: {}", app_name_owned);
                } else {
                    // Try fallback method if first approach fails
                    let fallback_script = format!(
                        r#"tell application "{}" to set visible to false"#,
                        app_name_owned
                    );
                    let _ = Command::new("osascript")
                        .arg("-e")
                        .arg(&fallback_script)
                        .output();
                }
            } else if let Err(e) = output {
                println!("⚠️ Error trying to hide app {}: {}", app_name_owned, e);
            }

            // After hiding/minimizing, try to find any running application whose
            // name contains "velosi" (case-insensitive) and activate it. If we
            // can't find one, fall back to attempting "Velosi" then "velosi".
            let list_script =
                r#"tell application "System Events" to get name of application processes"#;

            if let Ok(list_result) = Command::new("osascript")
                .arg("-e")
                .arg(list_script)
                .output()
            {
                if list_result.status.success() {
                    let out = String::from_utf8_lossy(&list_result.stdout).to_string();
                    // Split on commas/newlines and clean up quotes/spaces
                    let candidates: Vec<String> = out
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if let Some(found) = candidates
                        .into_iter()
                        .find(|n| n.to_lowercase().contains("velosi"))
                    {
                        let activate_script =
                            format!(r#"tell application "{}" to activate"#, found);
                        let _ = Command::new("osascript")
                            .arg("-e")
                            .arg(&activate_script)
                            .output();
                    } else {
                        // Fallback: try the common name variants
                        let fallback = r#"try
                                tell application "Velosi" to activate
                            on error
                                try
                                    tell application "velosi" to activate
                                end try
                            end try"#;
                        let _ = Command::new("osascript").arg("-e").arg(fallback).output();
                    }
                } else {
                    // If listing failed, do the fallback activation attempts
                    let fallback = r#"try
                            tell application "Velosi" to activate
                        on error
                            try
                                tell application "velosi" to activate
                            end try
                        end try"#;
                    let _ = Command::new("osascript").arg("-e").arg(fallback).output();
                }
            } else {
                // If calling osascript failed entirely, try the simple fallback
                let fallback = r#"try
                        tell application "Velosi" to activate
                    on error
                        try
                            tell application "velosi" to activate
                        end try
                    end try"#;
                let _ = Command::new("osascript").arg("-e").arg(fallback).output();
            }
        });

        Ok(())
    }

    pub async fn show_blocked_app(&self, app_name: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            self.show_macos_app(app_name).await
        }

        #[cfg(not(target_os = "macos"))]
        {
            Ok(())
        }
    }

    #[cfg(target_os = "macos")]
    async fn show_macos_app(&self, app_name: &str) -> Result<(), String> {
        use std::process::Command;

        // Try to show/unhide the app and bring it to front
        let show_script = format!(
            r#"
            tell application "{}"
                try
                    set visible to true
                    activate
                on error
                    -- If direct activation doesn't work, try through System Events
                    tell application "System Events"
                        tell process "{}"
                            try
                                set value of attribute "AXMinimized" of window 1 to false
                                set frontmost to true
                            end try
                        end tell
                    end tell
                end try
            end tell
            "#,
            app_name, app_name
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&show_script)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("✅ Successfully showed app: {}", app_name);
                } else {
                    println!(
                        "⚠️ Could not show app {}: {:?}",
                        app_name,
                        String::from_utf8_lossy(&result.stderr)
                    );
                }
            }
            Err(e) => {
                println!("⚠️ Error trying to show app {}: {}", app_name, e);
            }
        }

        Ok(())
    }
}
