use chrono::Utc;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

use crate::models::{PomodoroSession, PomodoroSessionType, PomodoroSettings, PomodoroSummary};
use crate::AppState;

#[tauri::command]
pub async fn save_pomodoro_session(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    session: PomodoroSession,
) -> Result<String, String> {
    let session_id = session.id.to_string();

    state
        .db
        .save_pomodoro_session(&session)
        .await
        .map_err(|e| format!("Failed to save pomodoro session: {}", e))?;

    // Emit event to frontend
    app_handle
        .emit("pomodoro-session-saved", &session)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(session_id)
}

#[tauri::command]
pub async fn get_pomodoro_settings(state: State<'_, AppState>) -> Result<PomodoroSettings, String> {
    state
        .db
        .get_pomodoro_settings()
        .await
        .map_err(|e| format!("Failed to get pomodoro settings: {}", e))
}

#[tauri::command]
pub async fn update_pomodoro_settings(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    settings: PomodoroSettings,
) -> Result<(), String> {
    state
        .db
        .update_pomodoro_settings(&settings)
        .await
        .map_err(|e| format!("Failed to update pomodoro settings: {}", e))?;

    // Emit event to frontend
    app_handle
        .emit("pomodoro-settings-updated", &settings)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn get_pomodoro_sessions(
    state: State<'_, AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
    session_type: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<PomodoroSession>, String> {
    let session_type_enum = match session_type.as_ref() {
        Some(s) => match s.as_str() {
            "work" => Some(PomodoroSessionType::Work),
            "break" => Some(PomodoroSessionType::Break),
            _ => return Err("Invalid session type".to_string()),
        },
        None => None,
    };

    state
        .db
        .get_pomodoro_sessions(start_date, end_date, session_type_enum, limit)
        .await
        .map_err(|e| format!("Failed to get pomodoro sessions: {}", e))
}

#[tauri::command]
pub async fn get_pomodoro_summary(
    state: State<'_, AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<PomodoroSummary, String> {
    state
        .db
        .get_pomodoro_summary(start_date, end_date)
        .await
        .map_err(|e| format!("Failed to get pomodoro summary: {}", e))
}

#[tauri::command]
pub async fn delete_pomodoro_session(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    session_id: String,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id).map_err(|e| format!("Invalid session ID: {}", e))?;

    state
        .db
        .delete_pomodoro_session(uuid)
        .await
        .map_err(|e| format!("Failed to delete pomodoro session: {}", e))?;

    // Emit event to frontend
    app_handle
        .emit("pomodoro-session-deleted", session_id)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn start_pomodoro_session(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    session_type: String,
    duration_minutes: i32,
    work_description: Option<String>,
    enable_focus_mode: bool,
    enable_app_tracking: bool,
) -> Result<PomodoroSession, String> {
    let session_type_enum = match session_type.as_str() {
        "work" => PomodoroSessionType::Work,
        "break" => PomodoroSessionType::Break,
        _ => return Err("Invalid session type".to_string()),
    };

    let session = PomodoroSession {
        id: Uuid::new_v4(),
        session_type: session_type_enum,
        start_time: Utc::now(),
        end_time: None,
        duration_minutes,
        actual_duration_seconds: None,
        work_description,
        completed: false,
        focus_mode_enabled: enable_focus_mode,
        app_tracking_enabled: enable_app_tracking,
    };

    // Save the session to database
    state
        .db
        .save_pomodoro_session(&session)
        .await
        .map_err(|e| format!("Failed to save pomodoro session: {}", e))?;

    // Enable focus mode if requested
    if enable_focus_mode {
        if let Err(e) =
            crate::commands::focus_mode::enable_focus_mode(state.clone(), app_handle.clone()).await
        {
            println!("Warning: Failed to enable focus mode for pomodoro: {}", e);
        }
    }

    // Enable or disable app tracking according to the session setting
    if enable_app_tracking {
        if let Err(e) =
            crate::commands::tracking::start_tracking(state.clone(), app_handle.clone()).await
        {
            println!("Warning: Failed to start app tracking for pomodoro: {}", e);
        }
    } else {
        if let Err(e) =
            crate::commands::tracking::stop_tracking(state.clone(), app_handle.clone()).await
        {
            println!("Warning: Failed to stop app tracking for pomodoro: {}", e);
        }
    }

    // Emit event to frontend
    app_handle
        .emit("pomodoro-session-started", &session)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(session)
}

#[tauri::command]
pub async fn complete_pomodoro_session(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    session_id: String,
    completed: bool,
) -> Result<PomodoroSession, String> {
    let uuid = Uuid::parse_str(&session_id).map_err(|e| format!("Invalid session ID: {}", e))?;

    let mut session = state
        .db
        .get_pomodoro_session_by_id(uuid)
        .await
        .map_err(|e| format!("Failed to get pomodoro session: {}", e))?
        .ok_or("Session not found")?;

    let now = Utc::now();
    session.end_time = Some(now);
    session.completed = completed;

    // Calculate actual duration
    if let Some(start_time) = Some(session.start_time) {
        session.actual_duration_seconds = Some((now - start_time).num_seconds() as i32);
    }

    // Update the session in database
    state
        .db
        .update_pomodoro_session(&session)
        .await
        .map_err(|e| format!("Failed to update pomodoro session: {}", e))?;

    // Disable focus mode if it was enabled for this session
    if session.focus_mode_enabled {
        if let Err(e) =
            crate::commands::focus_mode::disable_focus_mode(state.clone(), app_handle.clone()).await
        {
            println!(
                "Warning: Failed to disable focus mode after pomodoro: {}",
                e
            );
        }
    }

    // If this session had app tracking enabled, stop tracking when session completes
    if session.app_tracking_enabled {
        if let Err(e) =
            crate::commands::tracking::stop_tracking(state.clone(), app_handle.clone()).await
        {
            println!("Warning: Failed to stop app tracking after pomodoro: {}", e);
        }
    }

    // Emit event to frontend
    app_handle
        .emit("pomodoro-session-completed", &session)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(session)
}

#[tauri::command]
pub async fn test_notification_permissions(app_handle: AppHandle) -> Result<String, String> {
    println!("🧪 Testing notification permissions...");

    use tauri_plugin_notification::NotificationExt;

    // Try to check if notifications are supported/allowed
    match app_handle
        .notification()
        .builder()
        .title("Test Notification")
        .body("This is a test notification from Velosi")
        .show()
    {
        Ok(_) => {
            println!("✅ Test notification sent successfully");
            Ok("Notification permissions working".to_string())
        }
        Err(e) => {
            println!("❌ Test notification failed: {}", e);
            Err(format!("Notification test failed: {}", e))
        }
    }
}

#[tauri::command]
pub async fn send_pomodoro_notification(
    app_handle: AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    println!("🔔 Attempting to send notification: {} - {}", title, body);

    // Use the notification plugin for system-wide notifications
    use tauri_plugin_notification::NotificationExt;

    // Try to ensure we're sending the notification from the main thread
    let result = tokio::task::block_in_place(|| {
        app_handle
            .notification()
            .builder()
            .title(title.clone())
            .body(body.clone())
            .icon("Velosi.png")
            .show()
    });

    match result {
        Ok(_) => {
            println!("✅ Notification sent successfully");
            Ok(())
        }
        Err(e) => {
            println!("❌ Failed to send notification: {}", e);
            // Also try the simple approach as fallback
            println!("🔄 Trying fallback notification method...");
            match app_handle
                .notification()
                .builder()
                .title(title)
                .body(body)
                .show()
            {
                Ok(_) => {
                    println!("✅ Fallback notification sent successfully");
                    Ok(())
                }
                Err(e2) => {
                    println!("❌ Fallback notification also failed: {}", e2);
                    Err(format!(
                        "Failed to send notification: {} (fallback: {})",
                        e, e2
                    ))
                }
            }
        }
    }
}
