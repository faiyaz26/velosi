use chrono::NaiveDate;
use tauri::{AppHandle, Emitter, State};

use crate::models::{ActivityEntry, ActivitySummary, TimelineData};
use crate::tracker::CurrentActivity;
use crate::AppState;

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
