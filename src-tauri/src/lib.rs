mod database;
mod migrations;
mod models;
mod tracker;
mod tray;

use chrono::{NaiveDate, Utc};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tokio::time::{interval, Duration, Instant};
use uuid::Uuid;

use database::Database;
use models::{ActivityCategory, ActivityEntry, ActivitySummary, TimelineData, UserCategory};

use tracker::{ActivityTracker, CurrentActivity};
use tray::TrayManager;

// Application state
pub struct AppState {
    db: Arc<Database>,
    tracker: Arc<Mutex<ActivityTracker>>,
    is_tracking: Arc<Mutex<bool>>,
    pause_until: Arc<Mutex<Option<Instant>>>,
    is_paused_indefinitely: Arc<Mutex<bool>>,
}

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
    tray::TrayManager::update_menu(app_handle, is_tracking, pause_info).await
}

// Tauri commands
#[tauri::command]
async fn start_tracking(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
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
async fn stop_tracking(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
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
async fn get_tracking_status(state: State<'_, AppState>) -> Result<bool, String> {
    let is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
    Ok(*is_tracking)
}

#[tauri::command]
async fn get_current_activity(
    state: State<'_, AppState>,
) -> Result<Option<CurrentActivity>, String> {
    let mut tracker = state.tracker.lock().map_err(|e| e.to_string())?;
    Ok(tracker.get_current_activity())
}

#[tauri::command]
async fn test_permissions(state: State<'_, AppState>) -> Result<String, String> {
    let tracker = state.tracker.lock().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    {
        Ok(tracker.test_permissions())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok("Permission testing only available on macOS".to_string())
    }
}

#[tauri::command]
async fn get_permission_status(state: State<'_, AppState>) -> Result<bool, String> {
    let tracker = state.tracker.lock().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    {
        Ok(tracker.check_accessibility_permissions())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(true) // Assume permissions are granted on non-macOS systems
    }
}

#[tauri::command]
async fn update_user_activity(state: State<'_, AppState>) -> Result<(), String> {
    let mut tracker = state.tracker.lock().map_err(|e| e.to_string())?;
    tracker.update_last_input_time();
    Ok(())
}

#[tauri::command]
async fn get_activities_by_date(
    state: State<'_, AppState>,
    date: String,
) -> Result<Vec<ActivityEntry>, String> {
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;

    state
        .db
        .get_activities_by_date(parsed_date)
        .await
        .map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
async fn get_activities_by_date_range(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ActivityEntry>, String> {
    let parsed_start_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date format: {}", e))?;
    let parsed_end_date = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date format: {}", e))?;

    state
        .db
        .get_activities_by_date_range(parsed_start_date, parsed_end_date)
        .await
        .map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
async fn get_activity_summary(
    state: State<'_, AppState>,
    date: String,
) -> Result<ActivitySummary, String> {
    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))?;

    let summary = state
        .db
        .get_activity_summary(parsed_date)
        .await
        .map_err(|e| format!("Database error: {}", e))?;

    Ok(summary)
}

#[tauri::command]
async fn get_recent_activities(
    state: State<'_, AppState>,
    days: i64,
) -> Result<Vec<ActivitySummary>, String> {
    let mut summaries = Vec::new();
    let today = Utc::now().date_naive();

    for i in 0..days {
        let date = today - chrono::Duration::days(i);
        let summary = state
            .db
            .get_activity_summary(date)
            .await
            .map_err(|e| format!("Database error: {}", e))?;
        summaries.push(summary);
    }

    Ok(summaries)
}

#[tauri::command]
async fn get_timeline_data(
    state: State<'_, AppState>,
    minutes: Option<i64>,
) -> Result<TimelineData, String> {
    let timeline_minutes = minutes.unwrap_or(30); // Default to 30 minutes

    state
        .db
        .get_recent_timeline(timeline_minutes)
        .await
        .map_err(|e| format!("Database error: {}", e))
}

#[tauri::command]
async fn show_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_window(app_handle: AppHandle) -> Result<(), String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_tracking(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<bool, String> {
    let new_status = {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = !*is_tracking;
        *is_tracking
    };

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, new_status).await?;
    app_handle
        .emit("tracking-status-changed", new_status)
        .map_err(|e| e.to_string())?;

    Ok(new_status)
}

#[tauri::command]
async fn pause_tracking_for_duration(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    minutes: u64,
) -> Result<(), String> {
    // Stop tracking immediately
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = false;
    }

    // Set pause until time
    {
        let mut pause_until = state.pause_until.lock().map_err(|e| e.to_string())?;
        *pause_until = Some(Instant::now() + Duration::from_secs(minutes * 60));
    }

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, false).await?;
    app_handle
        .emit("tracking-status-changed", false)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn pause_tracking_until_tomorrow(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Stop tracking immediately
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = false;
    }

    // Calculate tomorrow at 9 AM
    let tomorrow = Utc::now()
        .date_naive()
        .succ_opt()
        .unwrap_or_else(|| Utc::now().date_naive())
        .and_hms_opt(9, 0, 0)
        .unwrap_or_else(|| Utc::now().naive_utc() + chrono::Duration::hours(24));

    let duration_until_tomorrow = (tomorrow - Utc::now().naive_utc()).num_seconds() as u64;

    // Set pause until time
    {
        let mut pause_until = state.pause_until.lock().map_err(|e| e.to_string())?;
        *pause_until = Some(Instant::now() + Duration::from_secs(duration_until_tomorrow));
    }

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, false).await?;
    app_handle
        .emit("tracking-status-changed", false)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn pause_tracking_indefinitely(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Stop tracking immediately
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = false;
    }

    // Set pause until to None (indefinite pause)
    {
        let mut pause_until = state.pause_until.lock().map_err(|e| e.to_string())?;
        *pause_until = None;
    }

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, false).await?;
    app_handle
        .emit("tracking-status-changed", false)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn resume_tracking_now(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    // Clear pause timer
    {
        let mut pause_until = state.pause_until.lock().map_err(|e| e.to_string())?;
        *pause_until = None;
    }

    // Start tracking
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = true;
    }

    // Update tray menu and emit event to frontend
    update_tray_with_state(&app_handle, true).await?;
    app_handle
        .emit("tracking-status-changed", true)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_pause_status(state: State<'_, AppState>) -> Result<Value, String> {
    let pause_until = state.pause_until.lock().map_err(|e| e.to_string())?;
    let is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;

    let remaining_seconds = if let Some(pause_time) = *pause_until {
        let now = Instant::now();
        if pause_time > now {
            (pause_time - now).as_secs()
        } else {
            0
        }
    } else {
        0
    };

    let result = serde_json::json!({
        "is_paused": !*is_tracking && (remaining_seconds > 0 || pause_until.is_none()),
        "remaining_seconds": remaining_seconds,
        "is_indefinite": pause_until.is_none() && !*is_tracking,
        "pause_until": if let Some(pause_time) = *pause_until {
            Some(pause_time.elapsed().as_secs())
        } else {
            None
        }
    });

    Ok(result)
}

#[tauri::command]
async fn get_categories(state: State<'_, AppState>) -> Result<Value, String> {
    // Load all categories from database (both built-in and user-defined)
    let categories = state
        .db
        .get_user_categories()
        .await
        .map_err(|e| format!("Failed to load categories: {}", e))?;

    // Convert to the expected JSON format
    let categories_json: Vec<serde_json::Value> = categories
        .into_iter()
        .map(|cat| {
            serde_json::json!({
                "id": cat.id,
                "name": cat.name,
                "color": cat.color,
                "parent_id": cat.parent_id,
                "created_at": cat.created_at,
                "updated_at": cat.updated_at
            })
        })
        .collect();

    let result = serde_json::json!({
        "categories": categories_json
    });

    Ok(result)
}

#[tauri::command]
async fn get_app_mappings(state: State<'_, AppState>) -> Result<Value, String> {
    println!("🔄 Loading app mappings from database...");

    // Load all mappings from database (both built-in and custom)
    let app_mappings = state
        .db
        .get_app_mappings()
        .await
        .map_err(|e| format!("Failed to load app mappings: {}", e))?;

    println!("🔍 Found {} app mappings from database", app_mappings.len());

    // Group mappings by category
    let mut category_mappings: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for mapping in app_mappings {
        category_mappings
            .entry(mapping.category_id)
            .or_insert_with(Vec::new)
            .push(mapping.app_pattern);
    }

    // Convert to the expected JSON format
    let mappings_json: Vec<serde_json::Value> = category_mappings
        .into_iter()
        .map(|(category, apps)| {
            serde_json::json!({
                "category": category,
                "apps": apps
            })
        })
        .collect();

    let result = serde_json::json!({
        "mappings": mappings_json
    });

    println!("📋 Final mappings count: {}", mappings_json.len());
    Ok(result)
}

#[tauri::command]
async fn add_category(
    name: String,
    color: String,
    parent_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let now = Utc::now();
    let category = UserCategory {
        id: Uuid::new_v4().to_string(),
        name,
        color,
        parent_id,
        created_at: now,
        updated_at: now,
    };

    state
        .db
        .add_user_category(&category)
        .await
        .map_err(|e| format!("Failed to add category: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn update_category(
    id: String,
    name: String,
    color: String,
    parent_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Create updated category with current timestamp
    let category = UserCategory {
        id,
        name,
        color,
        parent_id,
        created_at: Utc::now(), // This will be ignored in the update
        updated_at: Utc::now(),
    };

    state
        .db
        .update_user_category(&category)
        .await
        .map_err(|e| format!("Failed to update category: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn delete_category(id: String, state: State<'_, AppState>) -> Result<(), String> {
    state
        .db
        .delete_user_category(&id)
        .await
        .map_err(|e| format!("Failed to delete category: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn add_app_mapping(
    category_id: String,
    app_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let is_custom = true; // Default to true for manually added mappings
    state
        .db
        .add_simple_app_mapping(&category_id, &app_name, is_custom)
        .await
        .map_err(|e| format!("Failed to add app mapping: {}", e))
}

#[tauri::command]
async fn remove_app_mapping(
    category_id: String,
    app_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .db
        .remove_app_mapping(&category_id, &app_name)
        .await
        .map_err(|e| format!("Failed to remove app mapping: {}", e))
}

#[tauri::command]
async fn update_app_mapping(category: String, apps: Vec<String>) -> Result<(), String> {
    let mappings_path = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("data")
        .join("app-mappings.json");

    // Read existing mappings
    let mappings_content = std::fs::read_to_string(&mappings_path)
        .map_err(|e| format!("Failed to read app-mappings.json: {}", e))?;

    let mut mappings_data: Value = serde_json::from_str(&mappings_content)
        .map_err(|e| format!("Failed to parse app-mappings.json: {}", e))?;

    // Update the mapping
    if let Some(mappings_array) = mappings_data
        .get_mut("mappings")
        .and_then(|m| m.as_array_mut())
    {
        for mapping in mappings_array.iter_mut() {
            if let Some(mapping_category) = mapping.get("category").and_then(|c| c.as_str()) {
                if mapping_category == category {
                    *mapping = serde_json::json!({
                        "category": category,
                        "apps": apps
                    });
                    break;
                }
            }
        }
    }

    // Write back to file
    let updated_content = serde_json::to_string_pretty(&mappings_data)
        .map_err(|e| format!("Failed to serialize mappings: {}", e))?;

    std::fs::write(&mappings_path, updated_content)
        .map_err(|e| format!("Failed to write app-mappings.json: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn add_url_mapping(
    state: State<'_, AppState>,
    category_id: String,
    url_pattern: String,
) -> Result<(), String> {
    state
        .db
        .add_simple_url_mapping(&category_id, &url_pattern, true)
        .await
        .map_err(|e| format!("Failed to add URL mapping: {}", e))
}

#[tauri::command]
async fn remove_url_mapping(
    state: State<'_, AppState>,
    category_id: String,
    url_pattern: String,
) -> Result<(), String> {
    state
        .db
        .remove_url_mapping(&category_id, &url_pattern)
        .await
        .map_err(|e| format!("Failed to remove URL mapping: {}", e))
}

#[tauri::command]
async fn get_url_mappings(state: State<'_, AppState>) -> Result<Value, String> {
    println!("🔄 Loading URL mappings from database...");

    // Load all URL mappings from database (both built-in and custom)
    let url_mappings = state
        .db
        .get_url_mappings()
        .await
        .map_err(|e| format!("Failed to load URL mappings: {}", e))?;

    println!("🔍 Found {} URL mappings from database", url_mappings.len());

    // Group mappings by category
    let mut category_mappings: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for mapping in url_mappings {
        category_mappings
            .entry(mapping.category_id)
            .or_insert_with(Vec::new)
            .push(mapping.url_pattern);
    }

    // Convert to the expected JSON format
    let mappings_json: Vec<serde_json::Value> = category_mappings
        .into_iter()
        .map(|(category, urls)| {
            serde_json::json!({
                "category": category,
                "urls": urls
            })
        })
        .collect();

    let result = serde_json::json!({
        "mappings": mappings_json
    });

    println!("📋 Final URL mappings count: {}", mappings_json.len());
    Ok(result)
}

#[tauri::command]
async fn update_activity_category(
    activity_id: String,
    category: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    println!("Updating activity {} to category {}", activity_id, category);

    // Parse the category string into an ActivityCategory enum
    let activity_category = match category.as_str() {
        "development" => ActivityCategory::Development,
        "productive" => ActivityCategory::Productive,
        "communication" => ActivityCategory::Communication,
        "social" => ActivityCategory::Social,
        "entertainment" => ActivityCategory::Entertainment,
        _ => ActivityCategory::Unknown,
    };

    let result = state
        .db
        .update_activity_category(&activity_id, &activity_category)
        .await
        .map_err(|e| e.to_string());

    match &result {
        Ok(_) => println!("Successfully updated activity category"),
        Err(e) => println!("Failed to update activity category: {}", e),
    }

    result
}

async fn setup_database() -> Result<Database, Box<dyn std::error::Error>> {
    // Create data directory if it doesn't exist
    let app_data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("velosi-tracker");

    // Use std::fs to create directory synchronously since we need it immediately
    std::fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("velosi.db");
    let database_url = format!("sqlite:{}", db_path.display());

    println!("Setting up database at: {}", database_url);

    Database::new(&database_url).await.map_err(|e| {
        eprintln!("Database error: {:?}", e);
        e.into()
    })
}

// Helper function to categorize activity based on app name and URL with database mappings only
async fn categorize_activity(
    db: &Database,
    app_name: &str,
    _bundle_id: Option<&str>,
    url: Option<&str>,
) -> ActivityCategory {
    // First, try URL-based categorization if URL is available
    if let Some(url_str) = url {
        if let Ok(url_mappings) = db.get_url_mappings().await {
            for mapping in url_mappings {
                // Split pattern by "|" and check if any pattern matches
                let patterns: Vec<&str> = mapping.url_pattern.split('|').collect();
                let url_lower = url_str.to_lowercase();

                for pattern in patterns {
                    if url_lower.contains(&pattern.trim().to_lowercase()) {
                        // Convert category_id to ActivityCategory
                        return match mapping.category_id.to_lowercase().as_str() {
                            "development" => ActivityCategory::Development,
                            "communication" => ActivityCategory::Communication,
                            "social" => ActivityCategory::Social,
                            "entertainment" => ActivityCategory::Entertainment,
                            "productive" => ActivityCategory::Productive,
                            _ => ActivityCategory::Custom(mapping.category_id),
                        };
                    }
                }
            }
        }
    }

    // Try app-based categorization from database mappings
    if let Ok(app_mappings) = db.get_app_mappings().await {
        for mapping in app_mappings {
            // Split pattern by "|" and check if any pattern matches
            let patterns: Vec<&str> = mapping.app_pattern.split('|').collect();
            let app_lower = app_name.to_lowercase();

            for pattern in patterns {
                if app_lower.contains(&pattern.trim().to_lowercase()) {
                    // Convert category_id to ActivityCategory
                    return match mapping.category_id.to_lowercase().as_str() {
                        "development" => ActivityCategory::Development,
                        "communication" => ActivityCategory::Communication,
                        "social" => ActivityCategory::Social,
                        "entertainment" => ActivityCategory::Entertainment,
                        "productive" => ActivityCategory::Productive,
                        _ => ActivityCategory::Custom(mapping.category_id),
                    };
                }
            }
        }
    }

    // No database mappings found, return Unknown
    ActivityCategory::Unknown
}

async fn start_activity_tracking(app_handle: AppHandle) {
    let state: State<'_, AppState> = app_handle.state();
    let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds

    println!("Starting activity tracking loop...");

    loop {
        interval.tick().await;
        println!("🔄 Loop tick - checking tracking status...");

        // Check if pause timer has expired and auto-resume tracking
        {
            let mut pause_until = state.pause_until.lock().unwrap();
            if let Some(pause_time) = *pause_until {
                if Instant::now() >= pause_time {
                    // Pause timer expired, resume tracking
                    println!("⏰ Pause timer expired, resuming tracking...");
                    *pause_until = None;

                    let mut is_tracking = state.is_tracking.lock().unwrap();
                    *is_tracking = true;

                    // Update tray menu
                    let app_handle_clone = app_handle.clone();
                    tokio::spawn(async move {
                        if let Err(e) = update_tray_with_state(&app_handle_clone, true).await {
                            eprintln!("Failed to update tray menu: {}", e);
                        }
                        if let Err(e) = app_handle_clone.emit("tracking-status-changed", true) {
                            eprintln!("Failed to emit tracking status change: {}", e);
                        }
                    });
                }
            }
        }

        // Continue with normal activity tracking logic
        // Check if tracking is enabled
        let is_tracking = {
            let tracking_guard = state.is_tracking.lock().unwrap();
            *tracking_guard
        };
        println!("🔍 Is tracking enabled: {}", is_tracking);

        if !is_tracking {
            println!("Tracking is disabled, skipping...");
            continue;
        }

        // Check if user is active
        let should_track = {
            let mut tracker = state.tracker.lock().unwrap();
            tracker.should_track()
        };

        if !should_track {
            println!("User is inactive, skipping...");
            continue;
        }

        // Get current activity
        let current_activity = {
            let mut tracker = state.tracker.lock().unwrap();
            tracker.get_current_activity()
        };
        println!("Raw current_activity result: {:?}", current_activity);

        if let Some(current) = current_activity {
            println!(
                "Current activity: {} - {}",
                current.app_name, current.window_title
            );

            // Check if there's already an ongoing activity in the database
            match state.db.get_current_activity().await {
                Ok(Some(ongoing_activity)) => {
                    // There's an ongoing activity, check if it's the same
                    let is_same_activity = ongoing_activity.app_name == current.app_name
                        && ongoing_activity.window_title == current.window_title
                        && ongoing_activity.url == current.url;

                    if !is_same_activity {
                        println!("Activity changed! Ending previous activity and starting new one");

                        // End the current activity
                        let now = Utc::now();
                        if let Err(e) = state.db.end_current_activity(now).await {
                            eprintln!("Failed to end current activity: {}", e);
                        } else {
                            println!("Previous activity ended successfully");
                        }

                        // Start new activity
                        let category = categorize_activity(
                            &state.db,
                            &current.app_name,
                            current.app_bundle_id.as_deref(),
                            current.url.as_deref(),
                        )
                        .await;

                        let new_entry = ActivityEntry {
                            id: Uuid::new_v4(),
                            start_time: now,
                            end_time: None,
                            app_name: current.app_name.clone(),
                            app_bundle_id: current.app_bundle_id.clone(),
                            window_title: current.window_title.clone(),
                            url: current.url.clone(),
                            category,
                            segments: vec![], // TODO: Extract and store segments
                        };

                        if let Err(e) = state.db.start_activity(&new_entry).await {
                            eprintln!("Failed to start new activity: {}", e);
                        } else {
                            println!(
                                "New activity started: {} - {}",
                                new_entry.app_name, new_entry.window_title
                            );
                        }
                    } else {
                        println!("Same activity continuing, no action needed");
                    }
                }
                Ok(None) => {
                    // No ongoing activity, start a new one
                    println!("No ongoing activity, starting new one");

                    let category = categorize_activity(
                        &state.db,
                        &current.app_name,
                        current.app_bundle_id.as_deref(),
                        current.url.as_deref(),
                    )
                    .await;

                    let new_entry = ActivityEntry {
                        id: Uuid::new_v4(),
                        start_time: Utc::now(),
                        end_time: None,
                        app_name: current.app_name.clone(),
                        app_bundle_id: current.app_bundle_id.clone(),
                        window_title: current.window_title.clone(),
                        url: current.url.clone(),
                        category,
                        segments: vec![], // TODO: Extract and store segments
                    };

                    if let Err(e) = state.db.start_activity(&new_entry).await {
                        eprintln!("Failed to start new activity: {}", e);
                    } else {
                        println!(
                            "New activity started: {} - {}",
                            new_entry.app_name, new_entry.window_title
                        );
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get current activity from database: {}", e);
                }
            }
        } else {
            println!("No current activity detected, checking if we need to end ongoing activity");

            // No current activity detected, end any ongoing activity
            match state.db.get_current_activity().await {
                Ok(Some(_ongoing_activity)) => {
                    println!("Ending ongoing activity due to inactivity");
                    let now = Utc::now();
                    if let Err(e) = state.db.end_current_activity(now).await {
                        eprintln!("Failed to end activity due to inactivity: {}", e);
                    } else {
                        println!("Activity ended due to inactivity");
                    }
                }
                Ok(None) => {
                    println!("No ongoing activity to end");
                }
                Err(e) => {
                    eprintln!("Failed to check for ongoing activity: {}", e);
                }
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let rt = tokio::runtime::Runtime::new().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Setup database synchronously in a blocking context
            let db = rt
                .block_on(setup_database())
                .expect("Failed to setup database");

            // Initialize application state
            let state = AppState {
                db: Arc::new(db),
                tracker: Arc::new(Mutex::new(ActivityTracker::new())),
                is_tracking: Arc::new(Mutex::new(true)), // Start tracking by default
                pause_until: Arc::new(Mutex::new(None)),
                is_paused_indefinitely: Arc::new(Mutex::new(false)),
            };

            app.manage(state);

            // Create initial tray menu using the tray module
            let menu = tray::TrayManager::create_menu(app, true, None)?; // Start with tracking active, no pause info

            // Setup tray icon
            let _tray = TrayIconBuilder::with_id("main")
                .tooltip("Velosi Tracker - Tracking Active")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "toggle" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let app_handle = app_clone.clone();
                            let state: State<'_, AppState> = app_clone.state();
                            if let Err(e) = toggle_tracking(state, app_handle).await {
                                eprintln!("Failed to toggle tracking: {}", e);
                            }
                        });
                    }
                    "dashboard" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "pause_1min" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let app_handle = app_clone.clone();
                            let state: State<'_, AppState> = app_clone.state();
                            if let Err(e) = pause_tracking_for_duration(state, app_handle, 1).await
                            {
                                eprintln!("Failed to pause tracking: {}", e);
                            }
                        });
                    }
                    "pause_5min" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let app_handle = app_clone.clone();
                            let state: State<'_, AppState> = app_clone.state();
                            if let Err(e) = pause_tracking_for_duration(state, app_handle, 5).await
                            {
                                eprintln!("Failed to pause tracking: {}", e);
                            }
                        });
                    }
                    "pause_30min" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let app_handle = app_clone.clone();
                            let state: State<'_, AppState> = app_clone.state();
                            if let Err(e) = pause_tracking_for_duration(state, app_handle, 30).await
                            {
                                eprintln!("Failed to pause tracking: {}", e);
                            }
                        });
                    }
                    "pause_1hour" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let app_handle = app_clone.clone();
                            let state: State<'_, AppState> = app_clone.state();
                            if let Err(e) = pause_tracking_for_duration(state, app_handle, 60).await
                            {
                                eprintln!("Failed to pause tracking: {}", e);
                            }
                        });
                    }
                    "pause_until_tomorrow" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let app_handle = app_clone.clone();
                            let state: State<'_, AppState> = app_clone.state();
                            if let Err(e) = pause_tracking_until_tomorrow(state, app_handle).await {
                                eprintln!("Failed to pause tracking until tomorrow: {}", e);
                            }
                        });
                    }
                    "pause_indefinitely" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let app_handle = app_clone.clone();
                            let state: State<'_, AppState> = app_clone.state();
                            if let Err(e) = pause_tracking_indefinitely(state, app_handle).await {
                                eprintln!("Failed to pause tracking indefinitely: {}", e);
                            }
                        });
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // Handle window close event to hide instead of quit
            if let Some(window) = app.get_webview_window("main") {
                let app_handle_clone = app_handle.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent the window from closing
                        api.prevent_close();
                        // Hide the window instead
                        if let Some(window) = app_handle_clone.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                });
            }

            // Start background tracking outside the blocking context
            println!("🚀 About to spawn activity tracking task...");
            let handle_clone = app_handle.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(start_activity_tracking(handle_clone));
            });
            println!("✅ Activity tracking task spawned!");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_tracking,
            stop_tracking,
            get_tracking_status,
            get_current_activity,
            test_permissions,
            get_permission_status,
            update_user_activity,
            get_activities_by_date,
            get_activities_by_date_range,
            get_activity_summary,
            get_recent_activities,
            get_timeline_data,
            show_window,
            hide_window,
            toggle_tracking,
            get_categories,
            get_app_mappings,
            add_category,
            update_category,
            delete_category,
            add_app_mapping,
            remove_app_mapping,
            update_app_mapping,
            add_url_mapping,
            remove_url_mapping,
            get_url_mappings,
            update_activity_category,
            pause_tracking_for_duration,
            pause_tracking_until_tomorrow,
            pause_tracking_indefinitely,
            resume_tracking_now,
            get_pause_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
