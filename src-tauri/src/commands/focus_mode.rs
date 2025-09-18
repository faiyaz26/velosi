use tauri::{AppHandle, Emitter, State};
use tokio::time::Instant;

use crate::tray::TrayManager;
use crate::AppState;

// Helper function to get pause info for tray menu
async fn get_pause_info(state: &AppState, is_tracking: bool) -> Option<(u64, bool)> {
    if !is_tracking {
        if let Ok(pause_until) = state.pause_until.lock() {
            if let Some(pause_time) = *pause_until {
                let now = Instant::now();
                let remaining_seconds = if pause_time > now {
                    (pause_time - now).as_secs()
                } else {
                    0
                };
                Some((remaining_seconds, false))
            } else {
                // Indefinite pause (pause_until is None but tracking is false)
                Some((0, true))
            }
        } else {
            None
        }
    } else {
        None
    }
}

#[tauri::command]
pub async fn enable_focus_mode(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Update in-memory state only (no database persistence)
    {
        let mut focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
        *focus_enabled = true;
    }

    // Update tray menu to reflect focus mode change
    let is_tracking = {
        let tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *tracking
    };
    let pause_info = get_pause_info(&state, is_tracking).await;
    TrayManager::update_menu(&app_handle, is_tracking, pause_info, Some(true)).await?;

    // Emit event to frontend
    app_handle
        .emit("focus-mode-changed", true)
        .map_err(|e| e.to_string())?;

    // Emit cache invalidation event
    app_handle
        .emit(
            "focus-cache-invalidate",
            serde_json::json!({
                "type": "focus_mode_enabled_changed",
                "enabled": true
            }),
        )
        .map_err(|e| e.to_string())?;

    // Initialize proxy server if not already initialized
    {
        let needs_init = {
            let website_blocker = state.website_blocker.lock().map_err(|e| e.to_string())?;
            website_blocker.is_none()
        };

        if needs_init {
            println!("🚀 Initializing proxy server for focus mode...");
            let proxy_blocker =
                crate::local_proxy_blocker::LocalProxyBlocker::with_app_handle(app_handle.clone())
                    .with_database(state.db.clone());

            // Start the proxy server
            if let Err(e) = proxy_blocker.start_proxy_server().await {
                println!("❌ Failed to start proxy server: {}", e);
                return Err(format!("Failed to start proxy server: {}", e));
            }

            // Store the initialized blocker in the state
            let mut website_blocker = state.website_blocker.lock().map_err(|e| e.to_string())?;
            *website_blocker = Some(proxy_blocker);
            println!("✅ Proxy server initialized for focus mode");
        }
    }

    // Start website blocker when focus mode is enabled (only if website blocking is enabled)
    let website_blocking_enabled = state
        .db
        .get_website_blocking_enabled()
        .await
        .unwrap_or(true); // Default to true if error

    if website_blocking_enabled {
        if let Err(e) = start_website_blocking_internal(&state, &app_handle).await {
            println!("⚠️ Warning: Failed to start website blocker: {}", e);

            // Emit a warning event to the frontend so users know about the permission issue
            app_handle
                .emit(
                    "website-blocking-warning",
                    serde_json::json!({
                        "message": e,
                        "type": "permission_error"
                    }),
                )
                .map_err(|e| e.to_string())?;

            // Don't fail the entire focus mode enable if website blocker fails
        }
    } else {
        println!("ℹ️ Website blocking disabled by user preference, skipping website blocker initialization");
    }

    Ok(())
}

#[tauri::command]
pub async fn disable_focus_mode(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Update in-memory state only (no database persistence)
    {
        let mut focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
        *focus_enabled = false;
    }

    // Update tray menu to reflect focus mode change
    let is_tracking = {
        let tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *tracking
    };
    let pause_info = get_pause_info(&state, is_tracking).await;
    TrayManager::update_menu(&app_handle, is_tracking, pause_info, Some(false)).await?;

    // Emit event to frontend
    app_handle
        .emit("focus-mode-changed", false)
        .map_err(|e| e.to_string())?;

    // Emit cache invalidation event
    app_handle
        .emit(
            "focus-cache-invalidate",
            serde_json::json!({
                "type": "focus_mode_enabled_changed",
                "enabled": false
            }),
        )
        .map_err(|e| e.to_string())?;

    // Stop website blocker when focus mode is disabled (only if website blocking was enabled)
    let website_blocking_enabled = state
        .db
        .get_website_blocking_enabled()
        .await
        .unwrap_or(true); // Default to true if error

    if website_blocking_enabled {
        if let Err(e) = stop_website_blocking_internal(&state).await {
            println!("⚠️ Warning: Failed to stop website blocker: {}", e);
            // Don't fail the entire focus mode disable if website blocker fails
        }
    } else {
        println!(
            "ℹ️ Website blocking disabled by user preference, skipping website blocker shutdown"
        );
    }

    Ok(())
}

#[tauri::command]
pub async fn get_focus_mode_status(state: State<'_, AppState>) -> Result<bool, String> {
    // Get from in-memory state (not persisted)
    let focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
    Ok(*focus_enabled)
}

#[tauri::command]
pub async fn set_focus_mode_categories(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    categories: Vec<String>,
) -> Result<(), String> {
    // Persist to database
    state
        .db
        .set_focus_mode_allowed_categories(&categories)
        .await
        .map_err(|e| e.to_string())?;

    // Emit cache invalidation event instead of updating cache directly
    app_handle
        .emit(
            "focus-cache-invalidate",
            serde_json::json!({
                "type": "allowed_categories_changed",
                "categories": categories
            }),
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_focus_mode_categories(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    // Get from database (authoritative source)
    state
        .db
        .get_focus_mode_allowed_categories()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_app_focus_allowed(
    state: State<'_, AppState>,
    app_name: String,
    bundle_id: Option<String>,
) -> Result<bool, String> {
    // If focus mode is disabled, all apps are allowed
    let focus_enabled = {
        let focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
        *focus_enabled
    };

    if !focus_enabled {
        return Ok(true);
    }

    // Always allow velosi app itself
    if app_name.to_lowercase().contains("velosi")
        || bundle_id
            .as_ref()
            .map_or(false, |bid| bid.to_lowercase().contains("velosi"))
    {
        return Ok(true); // Velosi app is always allowed
    }

    // Check if app is allowed in database
    let is_allowed = state
        .db
        .is_focus_mode_app_allowed(&app_name)
        .await
        .map_err(|e| e.to_string())?;

    if is_allowed {
        return Ok(true);
    }

    let allowed_categories = {
        let allowed_categories = state
            .focus_mode_allowed_categories
            .lock()
            .map_err(|e| e.to_string())?;
        allowed_categories.clone()
    };

    // If no categories are specified, block everything
    if allowed_categories.is_empty() {
        return Ok(false);
    }

    // Get app mappings to determine category
    let app_mappings = state
        .db
        .get_app_mappings()
        .await
        .map_err(|e| e.to_string())?;

    // Find the category for this app
    for mapping in app_mappings {
        let patterns: Vec<&str> = mapping.app_pattern.split('|').collect();
        for pattern in patterns {
            if app_name
                .to_lowercase()
                .contains(&pattern.trim().to_lowercase())
                || bundle_id.as_ref().map_or(false, |bid| {
                    bid.to_lowercase().contains(&pattern.trim().to_lowercase())
                })
            {
                // App matches this category
                return Ok(allowed_categories.contains(&mapping.category_id));
            }
        }
    }

    // App not found in mappings, block by default in focus mode
    Ok(false)
}

#[tauri::command]
pub async fn allow_app(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    app_name: String,
    duration_minutes: Option<u32>,
) -> Result<(), String> {
    println!(
        "🔄 allow_app called for: '{}' with duration: {:?} minutes",
        app_name, duration_minutes
    );

    let expires_at = if let Some(duration) = duration_minutes {
        Some(chrono::Utc::now().timestamp() + (duration as i64 * 60))
    } else {
        None // Allow indefinitely
    };

    println!("📅 Expires at timestamp: {:?}", expires_at);

    // Store in database
    println!("💾 Adding to database...");
    state
        .db
        .add_focus_mode_allowed_app(&app_name, expires_at)
        .await
        .map_err(|e| {
            println!("❌ Database error: {}", e);
            e.to_string()
        })?;

    println!("✅ Successfully added to database");

    // Verify it was added by checking the database
    let is_allowed = state
        .db
        .is_focus_mode_app_allowed(&app_name)
        .await
        .map_err(|e| e.to_string())?;

    println!(
        "🔍 Verification: App '{}' is_allowed = {}",
        app_name, is_allowed
    );

    // Emit event to notify frontend to refresh allowed apps list
    println!("📡 Emitting app-temporarily-allowed event...");
    app_handle
        .emit(
            "app-temporarily-allowed",
            serde_json::json!({
                "app_name": app_name,
                "expires_at": expires_at
            }),
        )
        .map_err(|e| {
            println!("❌ Event emission error: {}", e);
            e.to_string()
        })?;

    // Also emit cache invalidation event for backend cache system
    println!("📡 Emitting focus-cache-invalidate event...");
    app_handle
        .emit(
            "focus-cache-invalidate",
            serde_json::json!({
                "type": "allowed_apps_changed",
                "app_name": app_name,
                "expires_at": expires_at
            }),
        )
        .map_err(|e| {
            println!("❌ Cache invalidation event emission error: {}", e);
            e.to_string()
        })?;

    println!("📡 Both events emitted successfully");

    if let Some(duration) = duration_minutes {
        println!("✅ Allowed app: {} for {} minutes", app_name, duration);
    } else {
        println!("✅ Allowed app: {} indefinitely", app_name);
    }

    Ok(())
}

#[tauri::command]
pub async fn get_focus_mode_allowed_apps(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    println!("🔍 get_focus_mode_allowed_apps called");

    // Get from database (authoritative source)
    let apps = state
        .db
        .get_focus_mode_allowed_apps()
        .await
        .map_err(|e| e.to_string())?;

    println!("📝 Current allowed apps from DB: {:?}", apps);

    // Also check cache
    let cache = state
        .focus_mode_allowed_apps_cache
        .lock()
        .map_err(|e| e.to_string())?;
    println!("💾 Current allowed apps cache: {:?}", *cache);

    Ok(apps)
}

#[derive(serde::Serialize)]
pub struct AllowedAppInfo {
    pub app_name: String,
    pub expires_at: Option<i64>,
    pub is_indefinite: bool,
    pub expires_in_minutes: Option<i64>,
}

#[tauri::command]
pub async fn get_focus_mode_allowed_apps_detailed(
    state: State<'_, AppState>,
) -> Result<Vec<AllowedAppInfo>, String> {
    println!("🔍 get_focus_mode_allowed_apps_detailed called");

    // Get from database with expiry info
    let apps_with_expiry = state
        .db
        .get_focus_mode_allowed_apps_with_expiry()
        .await
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().timestamp();

    let detailed_apps: Vec<AllowedAppInfo> = apps_with_expiry
        .into_iter()
        .map(|(app_name, expires_at)| {
            let is_indefinite = expires_at.is_none();
            let expires_in_minutes = expires_at.map(|exp| {
                let remaining_seconds = exp - now;
                remaining_seconds / 60 // Convert to minutes
            });

            AllowedAppInfo {
                app_name,
                expires_at,
                is_indefinite,
                expires_in_minutes,
            }
        })
        .collect();

    println!("📝 Detailed allowed apps: {:?}", detailed_apps.len());

    Ok(detailed_apps)
}

#[tauri::command]
pub async fn remove_focus_mode_allowed_app(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    app_name: String,
) -> Result<(), String> {
    // Remove from database
    state
        .db
        .remove_focus_mode_allowed_app(&app_name)
        .await
        .map_err(|e| e.to_string())?;

    // Emit cache invalidation event instead of updating cache directly
    app_handle
        .emit(
            "focus-cache-invalidate",
            serde_json::json!({
                "type": "allowed_apps_changed",
                "app_name": app_name,
                "removed": true
            }),
        )
        .map_err(|e| e.to_string())?;

    println!("✅ Removed allowed app: {}", app_name);
    Ok(())
}

#[tauri::command]
pub async fn show_focus_overlay(
    app_handle: AppHandle,
    app_name: String,
    reason: String,
) -> Result<(), String> {
    use tauri::Manager;

    // Create overlay window with app info as URL params
    let url = format!(
        "/focus-overlay?app_name={}&reason={}",
        urlencoding::encode(&app_name),
        urlencoding::encode(&reason)
    );

    println!("Creating overlays with URL: {}", url);

    // Get all available monitors
    let monitors = match app_handle.available_monitors() {
        Ok(monitors) => monitors,
        Err(e) => {
            println!(
                "Failed to get monitors, falling back to single overlay: {}",
                e
            );
            vec![] // Fall back to single overlay
        }
    };

    println!("Found {} monitor(s)", monitors.len());

    // If we have multiple monitors, create overlay on each one
    if monitors.len() > 1 {
        for (index, monitor) in monitors.iter().enumerate() {
            let window_label = format!("focus-overlay-{}", index);

            // Check if overlay already exists for this monitor
            if let Some(window) = app_handle.get_webview_window(&window_label) {
                println!("Showing existing overlay window on monitor {}", index);
                window.show().map_err(|e| e.to_string())?;
                if index == 0 {
                    // Only focus the first overlay to avoid focus conflicts
                    window.set_focus().map_err(|e| e.to_string())?;
                }
                continue;
            }

            // Create new overlay window for this monitor
            println!(
                "Creating new overlay window on monitor {} at position ({}, {}), size {}x{}",
                index,
                monitor.position().x,
                monitor.position().y,
                monitor.size().width,
                monitor.size().height
            );

            use tauri::WebviewWindowBuilder;

            let window_result = WebviewWindowBuilder::new(
                &app_handle,
                &window_label,
                tauri::WebviewUrl::App(url.clone().into()),
            )
            .title(&format!("Focus Mode - App Blocked (Monitor {})", index + 1))
            .position(monitor.position().x as f64, monitor.position().y as f64)
            .inner_size(monitor.size().width as f64, monitor.size().height as f64)
            .always_on_top(true)
            .skip_taskbar(true)
            .closable(false)
            .minimizable(false)
            .maximizable(false)
            .resizable(false)
            .decorations(false)
            .visible(false) // Start hidden to prevent flickering
            .focused(index == 0) // Only focus the first overlay
            .transparent(true) // Make window transparent for overlay effect
            .build();

            match window_result {
                Ok(window) => {
                    println!(
                        "Successfully created overlay window on monitor {}, now showing it",
                        index
                    );
                    // Small delay to ensure window is fully initialized
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    // Show the window
                    window.show().map_err(|e| e.to_string())?;
                    if index == 0 {
                        // Only focus the first overlay
                        window.set_focus().map_err(|e| e.to_string())?;
                    }
                }
                Err(e) => {
                    println!(
                        "Failed to create overlay window on monitor {}: {}",
                        index, e
                    );
                    // Continue with other monitors even if one fails
                }
            }
        }
    } else {
        // Fall back to single fullscreen overlay (original behavior)
        println!("Using single overlay (fallback or single monitor)");

        if let Some(window) = app_handle.get_webview_window("focus-overlay") {
            // If window exists, just show it and set focus
            println!("Showing existing overlay window");
            window.show().map_err(|e| e.to_string())?;
            window.set_focus().map_err(|e| e.to_string())?;
        } else {
            // Create new overlay window using WebviewWindowBuilder
            println!("Creating new overlay window");
            use tauri::WebviewWindowBuilder;

            let window_result = WebviewWindowBuilder::new(
                &app_handle,
                "focus-overlay",
                tauri::WebviewUrl::App(url.into()),
            )
            .title("Focus Mode - App Blocked")
            .fullscreen(true) // Use fullscreen instead of manual sizing
            .always_on_top(true)
            .skip_taskbar(true)
            .closable(false)
            .minimizable(false)
            .maximizable(false)
            .resizable(false)
            .decorations(false)
            .visible(false) // Start hidden to prevent flickering
            .focused(true) // Focus during creation for better input handling
            .transparent(true) // Make window transparent for overlay effect
            .build();

            match window_result {
                Ok(window) => {
                    println!("Successfully created overlay window, now showing it");
                    // Small delay to ensure window is fully initialized
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                    // Show and focus only after everything is configured
                    window.show().map_err(|e| e.to_string())?;
                    window.set_focus().map_err(|e| e.to_string())?;
                    // Additional delay and re-focus to ensure proper focus
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    window.set_focus().map_err(|e| e.to_string())?;
                }
                Err(e) => {
                    println!("Failed to create overlay window: {}", e);
                    return Err(e.to_string());
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn hide_focus_overlay(app_handle: AppHandle) -> Result<(), String> {
    use tauri::Manager;

    // Hide and close all overlay windows
    let mut closed_count = 0;

    // Try to close multi-monitor overlay windows (focus-overlay-0, focus-overlay-1, etc.)
    for i in 0..10 {
        // Assume max 10 monitors (reasonable limit)
        let window_label = format!("focus-overlay-{}", i);
        if let Some(window) = app_handle.get_webview_window(&window_label) {
            println!("Hiding overlay window on monitor {}", i);
            if let Err(e) = window.hide() {
                println!(
                    "Warning: Failed to hide overlay window on monitor {}: {}",
                    i, e
                );
            }

            // Close the window entirely to prevent flickering on next show
            if let Err(e) = window.close() {
                println!(
                    "Warning: Failed to close overlay window on monitor {}: {}",
                    i, e
                );
            } else {
                closed_count += 1;
            }
        }
    }

    // Also try to close the single overlay window (fallback)
    if let Some(window) = app_handle.get_webview_window("focus-overlay") {
        println!("Hiding single overlay window");
        if let Err(e) = window.hide() {
            println!("Warning: Failed to hide single overlay window: {}", e);
        }

        // Close the window entirely to prevent flickering on next show
        if let Err(e) = window.close() {
            println!("Warning: Failed to close single overlay window: {}", e);
        } else {
            closed_count += 1;
        }
    }

    if closed_count > 0 {
        println!("Successfully closed {} overlay window(s)", closed_count);
    } else {
        println!("No overlay windows were found to close");
    }

    Ok(())
}

// Internal helper functions for website blocking
async fn start_website_blocking_internal(
    state: &State<'_, AppState>,
    _app_handle: &AppHandle,
) -> Result<(), String> {
    // Get URLs to block from database
    let url_mappings = state
        .db
        .get_url_mappings()
        .await
        .map_err(|e| e.to_string())?;

    println!("🔍 Total URL mappings found: {}", url_mappings.len());
    for mapping in &url_mappings {
        println!(
            "  📋 URL: {} -> Category: {}",
            mapping.url_pattern, mapping.category_id
        );
    }

    // Filter for distracting categories (social and entertainment)
    let mut urls_to_block: Vec<String> = url_mappings
        .into_iter()
        .filter(|mapping| mapping.category_id == "social" || mapping.category_id == "entertainment")
        .map(|mapping| mapping.url_pattern)
        .collect();

    // Add some common test domains for debugging if the list is empty
    if urls_to_block.is_empty() {
        println!("⚠️ No URLs found in database for social/entertainment categories. Adding test domains for debugging.");
        urls_to_block.extend(vec![
            "facebook.com".to_string(),
            "twitter.com".to_string(),
            "instagram.com".to_string(),
            "youtube.com".to_string(),
            "tiktok.com".to_string(),
            "reddit.com".to_string(),
            "linkedin.com".to_string(),
        ]);
    }

    println!(
        "🚫 URLs to block (social/entertainment): {:?}",
        urls_to_block
    );

    let app_state = state.inner();

    // Get the website blocker instance (should be initialized at startup)
    let blocker = {
        let website_blocker = app_state
            .website_blocker
            .lock()
            .map_err(|e| e.to_string())?;

        website_blocker.clone()
    }; // MutexGuard dropped here

    if let Some(blocker) = blocker {
        blocker.enable_website_blocking(urls_to_block).await?;
        Ok(())
    } else {
        Err("Website blocker not initialized at startup".to_string())
    }
}

async fn stop_website_blocking_internal(state: &State<'_, AppState>) -> Result<(), String> {
    let app_state = state.inner();

    // Get the website blocker instance
    let blocker = {
        let website_blocker = app_state
            .website_blocker
            .lock()
            .map_err(|e| e.to_string())?;

        website_blocker.clone()
    }; // MutexGuard dropped here

    if let Some(blocker) = blocker {
        blocker.disable_website_blocking().await?;
        Ok(())
    } else {
        Err("Website blocker not initialized at startup".to_string())
    }
}
