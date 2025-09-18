use chrono::Utc;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{Duration, Instant};

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

// Helper function to update tray menu with pause info and focus mode status
async fn update_tray_with_state(app_handle: &AppHandle, is_tracking: bool) -> Result<(), String> {
    let state: State<'_, AppState> = app_handle.state();
    let pause_info = get_pause_info(&state, is_tracking).await;
    let focus_mode_enabled = {
        let focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
        *focus_enabled
    };
    TrayManager::update_menu(
        app_handle,
        is_tracking,
        pause_info,
        Some(focus_mode_enabled),
    )
    .await
}

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

    // Ensure any ongoing activity is ended so active time stops accruing,
    // and clear the in-memory current activity so frontend stops showing it as active.
    {
        let now = Utc::now();
        if let Err(e) = state.db.end_current_activity(now).await {
            eprintln!("Failed to end current activity when pausing: {}", e);
        }

        if let Ok(mut current_activity) = state.current_activity.lock() {
            *current_activity = None;
            if let Err(e) = app_handle.emit("current-activity-changed", json!(null)) {
                eprintln!(
                    "Failed to emit current-activity-changed after clearing: {}",
                    e
                );
            }
        } else {
            eprintln!("Failed to acquire lock to clear current_activity when pausing");
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
