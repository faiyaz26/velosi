use chrono::{NaiveDate, Utc};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::models::{ActivityEntry, ActivitySummary, TimelineData, UrlMapping, UserCategory};
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
