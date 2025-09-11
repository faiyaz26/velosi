use chrono::{NaiveDate, Utc};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::models::{ActivityEntry, ActivitySummary, TimelineData, UserCategory};
use crate::tracker::CurrentActivity;
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

// Helper function to update tray menu with pause info
async fn update_tray_with_state(app_handle: &AppHandle, is_tracking: bool) -> Result<(), String> {
    let state: State<'_, AppState> = app_handle.state();
    let pause_info = get_pause_info(&state, is_tracking).await;
    TrayManager::update_menu(app_handle, is_tracking, pause_info).await
}

// Tauri commands
#[tauri::command]
pub async fn start_tracking(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = true;
    } // MutexGuard is dropped here

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, true).await?;
    app_handle
        .emit("tracking-status-changed", true)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn stop_tracking(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = false;
    } // MutexGuard is dropped here

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, false).await?;
    app_handle
        .emit("tracking-status-changed", false)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_tracking_status(state: State<'_, AppState>) -> Result<bool, String> {
    let is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
    Ok(*is_tracking)
}

#[tauri::command]
pub async fn pause_tracking(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    duration_seconds: Option<u64>,
) -> Result<(), String> {
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = false;
    }

    {
        let mut pause_until = state.pause_until.lock().map_err(|e| e.to_string())?;
        if let Some(seconds) = duration_seconds {
            *pause_until = Some(Instant::now() + Duration::from_secs(seconds));
        } else {
            *pause_until = None; // Indefinite pause
        }
    }

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, false).await?;
    app_handle
        .emit("tracking-status-changed", false)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn pause_tracking_for_duration(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    duration_seconds: u64,
) -> Result<(), String> {
    pause_tracking(state, app_handle, Some(duration_seconds)).await
}

#[tauri::command]
pub async fn resume_tracking(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = true;
    }

    {
        let mut pause_until = state.pause_until.lock().map_err(|e| e.to_string())?;
        *pause_until = None;
    }

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, true).await?;
    app_handle
        .emit("tracking-status-changed", true)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_tracking(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let current_status = {
        let is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking
    };

    if current_status {
        stop_tracking(state, app_handle).await
    } else {
        start_tracking(state, app_handle).await
    }
}

#[tauri::command]
pub async fn get_activities_by_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<ActivityEntry>, String> {
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;

    state
        .db
        .get_activities_by_date(parsed_date)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_activity_summary(
    state: State<'_, AppState>,
    date: String,
) -> Result<ActivitySummary, String> {
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;

    state
        .db
        .get_activity_summary(parsed_date)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_timeline_data(
    state: State<'_, AppState>,
    date: String,
) -> Result<TimelineData, String> {
    // For now, we'll get recent timeline for the last 24 hours
    // TODO: Use the date parameter to filter timeline data
    let _parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;

    state
        .db
        .get_recent_timeline(1440) // 24 hours in minutes
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<UserCategory>, String> {
    state
        .db
        .get_user_categories()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_category(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<UserCategory, String> {
    let now = Utc::now();
    let category = UserCategory {
        id: Uuid::new_v4().to_string(),
        name,
        color,
        parent_id: None,
        created_at: now,
        updated_at: now,
    };

    state
        .db
        .add_user_category(&category)
        .await
        .map_err(|e| e.to_string())?;

    Ok(category)
}

#[tauri::command]
pub async fn update_category(
    state: State<'_, AppState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), String> {
    let category = UserCategory {
        id: id.clone(),
        name,
        color,
        parent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state
        .db
        .update_user_category(&category)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_category(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state
        .db
        .delete_user_category(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_mappings(state: State<'_, AppState>) -> Result<Value, String> {
    let mappings = state
        .db
        .get_app_mappings()
        .await
        .map_err(|e| e.to_string())?;

    // Get all categories to map category_id to category name
    let categories = state
        .db
        .get_user_categories()
        .await
        .map_err(|e| e.to_string())?;

    let category_map: std::collections::HashMap<String, String> = categories
        .into_iter()
        .map(|cat| (cat.id, cat.name))
        .collect();

    // Group mappings by category_id
    use std::collections::HashMap;
    let mut category_mappings: HashMap<String, Vec<String>> = HashMap::new();

    for mapping in mappings {
        category_mappings
            .entry(mapping.category_id)
            .or_insert_with(Vec::new)
            .push(mapping.app_pattern);
    }

    // Convert to the expected JSON format with category names
    let mappings_json: Vec<serde_json::Value> = category_mappings
        .into_iter()
        .map(|(category_id, apps)| {
            let category_name = category_map
                .get(&category_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            serde_json::json!({
                "category": category_name,
                "apps": apps
            })
        })
        .collect();

    Ok(serde_json::json!({
        "mappings": mappings_json
    }))
}

#[tauri::command]
pub async fn add_app_mapping(
    state: State<'_, AppState>,
    app_name: String,
    category_id: String,
) -> Result<(), String> {
    state
        .db
        .add_simple_app_mapping(&category_id, &app_name, true)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_app_mapping(
    state: State<'_, AppState>,
    app_name: String,
    category_id: String,
) -> Result<(), String> {
    // For update, we need to remove the old mapping and add a new one
    // since the app_name might have changed
    state
        .db
        .remove_app_mapping(&category_id, &app_name)
        .await
        .map_err(|e| e.to_string())?;

    state
        .db
        .add_simple_app_mapping(&category_id, &app_name, true)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_app_mapping(
    state: State<'_, AppState>,
    app_name: String,
) -> Result<(), String> {
    // We need to find the mapping first to get the category_id
    let mappings = state
        .db
        .get_app_mappings()
        .await
        .map_err(|e| e.to_string())?;

    for mapping in mappings {
        if mapping.app_pattern == app_name {
            return state
                .db
                .remove_app_mapping(&mapping.category_id, &app_name)
                .await
                .map_err(|e| e.to_string());
        }
    }

    Err("App mapping not found".to_string())
}

#[tauri::command]
pub async fn show_main_window(app_handle: AppHandle) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn hide_main_window(app_handle: AppHandle) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    window.hide().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn hide_window(app_handle: AppHandle) -> Result<(), String> {
    hide_main_window(app_handle).await
}

#[tauri::command]
pub async fn close_main_window(app_handle: AppHandle) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("Main window not found")?;

    window.close().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_pause_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let is_tracking = {
        let is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking
    };

    if let Some((remaining_seconds, is_indefinite)) = get_pause_info(&state, is_tracking).await {
        Ok(serde_json::json!({
            "is_paused": true,
            "remaining_seconds": remaining_seconds,
            "is_indefinite": is_indefinite
        }))
    } else {
        Ok(serde_json::json!({
            "is_paused": false,
            "remaining_seconds": 0,
            "is_indefinite": false
        }))
    }
}

#[tauri::command]
pub async fn get_current_activity(
    state: State<'_, AppState>,
) -> Result<Option<CurrentActivity>, String> {
    let current_activity = state.current_activity.lock().map_err(|e| e.to_string())?;
    Ok(current_activity.clone())
}

#[tauri::command]
pub async fn set_current_activity(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    activity: Option<CurrentActivity>,
) -> Result<(), String> {
    {
        let mut current_activity = state.current_activity.lock().map_err(|e| e.to_string())?;
        *current_activity = activity.clone();
    }

    // Emit event to frontend
    app_handle
        .emit("current-activity-changed", activity)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_activities_by_date_range(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ActivityEntry>, String> {
    let start_parsed = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date format: {}", e))?;
    let end_parsed = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date format: {}", e))?;

    state
        .db
        .get_activities_by_date_range(start_parsed, end_parsed)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_categories(state: State<'_, AppState>) -> Result<Vec<UserCategory>, String> {
    state
        .db
        .get_user_categories()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_url_mappings(state: State<'_, AppState>) -> Result<Value, String> {
    let mappings = state
        .db
        .get_url_mappings()
        .await
        .map_err(|e| e.to_string())?;

    // Get all categories to map category_id to category name
    let categories = state
        .db
        .get_user_categories()
        .await
        .map_err(|e| e.to_string())?;

    let category_map: std::collections::HashMap<String, String> = categories
        .into_iter()
        .map(|cat| (cat.id, cat.name))
        .collect();

    // Group mappings by category_id
    use std::collections::HashMap;
    let mut category_mappings: HashMap<String, Vec<String>> = HashMap::new();

    for mapping in mappings {
        category_mappings
            .entry(mapping.category_id)
            .or_insert_with(Vec::new)
            .push(mapping.url_pattern);
    }

    // Convert to the expected JSON format with category names
    let mappings_json: Vec<serde_json::Value> = category_mappings
        .into_iter()
        .map(|(category_id, urls)| {
            let category_name = category_map
                .get(&category_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            serde_json::json!({
                "category": category_name,
                "urls": urls
            })
        })
        .collect();

    Ok(serde_json::json!({
        "mappings": mappings_json
    }))
}

#[tauri::command]
pub async fn pause_tracking_until_tomorrow(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Calculate seconds until tomorrow at midnight
    let now = chrono::Utc::now();
    let tomorrow = (now + chrono::Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let seconds_until_tomorrow = (tomorrow - now).num_seconds() as u64;

    pause_tracking(state, app_handle, Some(seconds_until_tomorrow)).await
}

#[tauri::command]
pub async fn pause_tracking_indefinitely(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    pause_tracking(state, app_handle, None).await
}

#[tauri::command]
pub async fn resume_tracking_now(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    resume_tracking(state, app_handle).await
}

#[tauri::command]
pub async fn update_activity_category(
    state: State<'_, AppState>,
    activity_id: String,
    category_id: String,
) -> Result<(), String> {
    // Convert category_id to ActivityCategory
    let category = match category_id.as_str() {
        "productive" => crate::models::ActivityCategory::Productive,
        "social" => crate::models::ActivityCategory::Social,
        "entertainment" => crate::models::ActivityCategory::Entertainment,
        "development" => crate::models::ActivityCategory::Development,
        "communication" => crate::models::ActivityCategory::Communication,
        "unknown" => crate::models::ActivityCategory::Unknown,
        custom_id => crate::models::ActivityCategory::Custom(custom_id.to_string()),
    };

    state
        .db
        .update_activity_category(&activity_id, &category)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_permission_status() -> Result<bool, String> {
    // This is a placeholder - implement based on your permission system
    // For now, return true as if permissions are granted
    Ok(true)
}

#[tauri::command]
pub async fn remove_app_mapping(
    state: State<'_, AppState>,
    app_name: String,
) -> Result<(), String> {
    delete_app_mapping(state, app_name).await
}

#[tauri::command]
pub async fn add_url_mapping(
    state: State<'_, AppState>,
    url_pattern: String,
    category_id: String,
) -> Result<(), String> {
    let mapping = crate::models::UrlMapping {
        id: uuid::Uuid::new_v4(),
        url_pattern,
        category_id,
        is_custom: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state
        .db
        .add_url_mapping(&mapping)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_url_mapping(
    state: State<'_, AppState>,
    url_pattern: String,
    category_id: String,
) -> Result<(), String> {
    state
        .db
        .remove_url_mapping(&category_id, &url_pattern)
        .await
        .map_err(|e| e.to_string())
}

// =============================================================================
// FOCUS MODE COMMANDS
// =============================================================================

#[tauri::command]
pub async fn enable_focus_mode(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Persist to database
    state
        .db
        .set_focus_mode_enabled(true)
        .await
        .map_err(|e| e.to_string())?;

    // Update in-memory state
    {
        let mut focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
        *focus_enabled = true;
    }

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

    Ok(())
}

#[tauri::command]
pub async fn disable_focus_mode(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Persist to database
    state
        .db
        .set_focus_mode_enabled(false)
        .await
        .map_err(|e| e.to_string())?;

    // Update in-memory state
    {
        let mut focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
        *focus_enabled = false;
    }

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

    Ok(())
}

#[tauri::command]
pub async fn get_focus_mode_status(state: State<'_, AppState>) -> Result<bool, String> {
    // Get from database (authoritative source)
    state
        .db
        .get_focus_mode_enabled()
        .await
        .map_err(|e| e.to_string())
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

    println!("Creating overlay with URL: {}", url);

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

    Ok(())
}

#[tauri::command]
pub async fn hide_focus_overlay(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("focus-overlay") {
        println!("Hiding overlay window");
        window.hide().map_err(|e| e.to_string())?;

        // Close the window entirely to prevent flickering on next show
        // The window will be recreated next time it's needed
        if let Err(e) = window.close() {
            println!("Warning: Failed to close overlay window: {}", e);
        }
    }
    Ok(())
}
