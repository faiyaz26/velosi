mod database;
mod migrations;
mod models;
mod tracker;

use chrono::{NaiveDate, Utc};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tokio::time::{interval, Duration};
use uuid::Uuid;

use database::Database;
use models::{ActivityCategory, ActivityEntry, ActivitySummary, TimelineData, UserCategory};

use tracker::{ActivityTracker, CurrentActivity};

// Application state
pub struct AppState {
    db: Arc<Database>,
    tracker: Arc<Mutex<ActivityTracker>>,
    is_tracking: Arc<Mutex<bool>>,
}

// Tauri commands
#[tauri::command]
async fn start_tracking(state: State<'_, AppState>, app_handle: AppHandle) -> Result<(), String> {
    {
        let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
        *is_tracking = true;
    } // MutexGuard is dropped here

    // Update tray menu and emit event to frontend
    update_tray_menu(&app_handle, true).await?;
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
    update_tray_menu(&app_handle, false).await?;
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
    update_tray_menu(&app_handle, new_status).await?;
    app_handle
        .emit("tracking-status-changed", new_status)
        .map_err(|e| e.to_string())?;

    Ok(new_status)
}

#[tauri::command]
async fn load_categories(app_handle: tauri::AppHandle) -> Result<Value, String> {
    // Try multiple path resolution strategies
    let possible_paths = vec![
        // Development: relative to current directory
        std::env::current_dir()
            .ok()
            .map(|dir| dir.join("data").join("categories.json")),
        // Development: relative to project root
        std::env::current_dir()
            .ok()
            .map(|dir| dir.parent().map(|p| p.join("data").join("categories.json")))
            .flatten(),
        // Try the app's executable directory
        app_handle
            .path()
            .app_local_data_dir()
            .ok()
            .map(|dir| dir.join("data").join("categories.json")),
    ];

    let mut categories_path = None;
    for path_option in possible_paths {
        if let Some(path) = path_option {
            if path.exists() {
                categories_path = Some(path);
                break;
            }
        }
    }

    let categories_path =
        categories_path.ok_or("Could not find categories.json in any expected location")?;

    let categories_content = std::fs::read_to_string(&categories_path).map_err(|e| {
        format!(
            "Failed to read categories.json from {:?}: {}",
            categories_path, e
        )
    })?;

    let categories: Value = serde_json::from_str(&categories_content)
        .map_err(|e| format!("Failed to parse categories.json: {}", e))?;

    Ok(categories)
}

#[tauri::command]
async fn load_app_mappings(app_handle: tauri::AppHandle) -> Result<Value, String> {
    // Try multiple path resolution strategies
    let possible_paths = vec![
        // Development: relative to current directory
        std::env::current_dir()
            .ok()
            .map(|dir| dir.join("data").join("app-mappings.json")),
        // Development: relative to project root
        std::env::current_dir()
            .ok()
            .map(|dir| {
                dir.parent()
                    .map(|p| p.join("data").join("app-mappings.json"))
            })
            .flatten(),
        // Try the app's executable directory
        app_handle
            .path()
            .app_local_data_dir()
            .ok()
            .map(|dir| dir.join("data").join("app-mappings.json")),
    ];

    let mut mappings_path = None;
    for path_option in possible_paths {
        if let Some(path) = path_option {
            if path.exists() {
                mappings_path = Some(path);
                break;
            }
        }
    }

    let mappings_path =
        mappings_path.ok_or("Could not find app-mappings.json in any expected location")?;

    let mappings_content = std::fs::read_to_string(&mappings_path).map_err(|e| {
        format!(
            "Failed to read app-mappings.json from {:?}: {}",
            mappings_path, e
        )
    })?;

    let mappings: Value = serde_json::from_str(&mappings_content)
        .map_err(|e| format!("Failed to parse app-mappings.json: {}", e))?;

    Ok(mappings)
}

#[tauri::command]
async fn get_categories(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    // Load built-in categories from JSON
    let builtin_categories = load_categories(app_handle).await?;

    // Load user-defined categories from database
    let user_categories = state
        .db
        .get_user_categories()
        .await
        .map_err(|e| format!("Failed to load user categories: {}", e))?;

    // Merge built-in and user categories
    let mut result = builtin_categories;
    if let Some(categories_array) = result.get_mut("categories").and_then(|v| v.as_array_mut()) {
        for user_cat in user_categories {
            let user_cat_json = serde_json::json!({
                "id": user_cat.id,
                "name": user_cat.name,
                "color": user_cat.color,
                "parent_id": user_cat.parent_id
            });
            categories_array.push(user_cat_json);
        }
    }

    Ok(result)
}

#[tauri::command]
async fn get_app_mappings(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    println!("🔄 Loading app mappings...");

    // Load built-in mappings from JSON
    let builtin_mappings = match load_app_mappings(app_handle).await {
        Ok(mappings) => {
            println!("✅ Successfully loaded built-in mappings");
            mappings
        }
        Err(e) => {
            println!(
                "⚠️  No built-in mappings found: {}, using empty structure",
                e
            );
            serde_json::json!({ "mappings": [] })
        }
    };

    // Load custom mappings from database
    let custom_mappings = state
        .db
        .get_app_mappings()
        .await
        .map_err(|e| format!("Failed to load custom app mappings: {}", e))?;

    println!(
        "🔍 Found {} custom mappings from database",
        custom_mappings.len()
    );

    // Start with built-in mappings
    let mut result = builtin_mappings;

    // Apply custom mappings (they override built-in ones)
    if let Some(mappings_array) = result.get_mut("mappings").and_then(|v| v.as_array_mut()) {
        for custom_mapping in custom_mappings {
            if custom_mapping.is_custom {
                // Find existing mapping for this category and replace it
                let mut found = false;
                for mapping in mappings_array.iter_mut() {
                    if let Some(category) = mapping.get("category").and_then(|v| v.as_str()) {
                        if category == custom_mapping.category_id {
                            // Replace with custom mapping
                            *mapping = serde_json::json!({
                                "category": custom_mapping.category_id,
                                "apps": vec![custom_mapping.app_pattern.clone()]
                            });
                            found = true;
                            break;
                        }
                    }
                }

                // If not found, add as new mapping
                if !found {
                    mappings_array.push(serde_json::json!({
                        "category": custom_mapping.category_id,
                        "apps": vec![custom_mapping.app_pattern]
                    }));
                }
            }
        }
    }

    // Debug: print final result
    if let Some(mappings_array) = result.get("mappings").and_then(|v| v.as_array()) {
        println!("📋 Final mappings count: {}", mappings_array.len());
        for (i, mapping) in mappings_array.iter().take(3).enumerate() {
            println!("📋 Sample mapping {}: {:?}", i, mapping);
        }
    }

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
    category_id: i32,
    app_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let is_custom = true; // Default to true for manually added mappings
    state
        .db
        .add_simple_app_mapping(&category_id.to_string(), &app_name, is_custom)
        .await
        .map_err(|e| format!("Failed to add app mapping: {}", e))
}

#[tauri::command]
async fn remove_app_mapping(
    category_id: String,
    app_pattern: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state
        .db
        .remove_app_mapping(&category_id, &app_pattern)
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
async fn get_url_mappings(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<Value, String> {
    println!("🔄 Loading URL mappings...");

    // Load built-in URL mappings from JSON (if they exist)
    let builtin_mappings = match load_url_mappings(app_handle).await {
        Ok(mappings) => {
            println!("✅ Successfully loaded built-in URL mappings");
            mappings
        }
        Err(e) => {
            println!(
                "⚠️  No built-in URL mappings found: {}, using empty structure",
                e
            );
            serde_json::json!({ "mappings": [] })
        }
    };

    // Load custom URL mappings from database
    let custom_mappings = state
        .db
        .get_url_mappings()
        .await
        .map_err(|e| format!("Failed to load custom URL mappings: {}", e))?;

    println!(
        "🔍 Found {} custom URL mappings from database",
        custom_mappings.len()
    );

    // Start with built-in mappings
    let mut result = builtin_mappings;

    // Apply custom mappings (they override built-in ones)
    if let Some(mappings_array) = result.get_mut("mappings").and_then(|v| v.as_array_mut()) {
        for custom_mapping in custom_mappings {
            if custom_mapping.is_custom {
                // Find existing mapping for this category and replace it
                let mut found = false;
                for mapping in mappings_array.iter_mut() {
                    if let Some(category) = mapping.get("category").and_then(|v| v.as_str()) {
                        if category == custom_mapping.category_id {
                            // Replace with custom mapping
                            *mapping = serde_json::json!({
                                "category": custom_mapping.category_id,
                                "urls": vec![custom_mapping.url_pattern.clone()]
                            });
                            found = true;
                            break;
                        }
                    }
                }

                // If not found, add as new mapping
                if !found {
                    mappings_array.push(serde_json::json!({
                        "category": custom_mapping.category_id,
                        "urls": vec![custom_mapping.url_pattern]
                    }));
                }
            }
        }
    }

    // Debug: print final result
    if let Some(mappings_array) = result.get("mappings").and_then(|v| v.as_array()) {
        println!("📋 Final URL mappings count: {}", mappings_array.len());
        for (i, mapping) in mappings_array.iter().take(3).enumerate() {
            println!("📋 Sample URL mapping {}: {:?}", i, mapping);
        }
    }

    Ok(result)
}

async fn load_url_mappings(app_handle: tauri::AppHandle) -> Result<Value, String> {
    // Try multiple path resolution strategies
    let possible_paths = vec![
        // Development: relative to current directory
        std::env::current_dir()
            .ok()
            .map(|dir| dir.join("data").join("url-mappings.json")),
        // Development: relative to project root
        std::env::current_dir()
            .ok()
            .map(|dir| {
                dir.parent()
                    .map(|p| p.join("data").join("url-mappings.json"))
            })
            .flatten(),
        // Try the app's executable directory
        app_handle
            .path()
            .app_local_data_dir()
            .ok()
            .map(|dir| dir.join("data").join("url-mappings.json")),
    ];

    let mut mappings_path = None;
    for path_option in possible_paths {
        if let Some(path) = path_option {
            if path.exists() {
                mappings_path = Some(path);
                break;
            }
        }
    }

    let path = mappings_path.ok_or("Could not find url-mappings.json in any expected location")?;

    let mappings_content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read url-mappings.json: {}", e))?;

    let mappings: Value = serde_json::from_str(&mappings_content)
        .map_err(|e| format!("Failed to parse url-mappings.json: {}", e))?;

    Ok(mappings)
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

async fn update_tray_menu(app_handle: &AppHandle, is_tracking: bool) -> Result<(), String> {
    use tauri::menu::{Menu, MenuItem};

    let status_text = if is_tracking {
        "🟢 Tracking Active"
    } else {
        "🔴 Tracking Paused"
    };

    let toggle_text = if is_tracking {
        "Pause Tracking"
    } else {
        "Start Tracking"
    };

    // Create menu items
    let status_item = MenuItem::with_id(app_handle, "status", status_text, false, None::<&str>)
        .map_err(|e| e.to_string())?;
    let toggle_item = MenuItem::with_id(app_handle, "toggle", toggle_text, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    let dashboard_item =
        MenuItem::with_id(app_handle, "dashboard", "Dashboard", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    let quit_item = MenuItem::with_id(app_handle, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        app_handle,
        &[&status_item, &toggle_item, &dashboard_item, &quit_item],
    )
    .map_err(|e| e.to_string())?;

    // Update the tray menu
    if let Some(tray) = app_handle.tray_by_id("main") {
        tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;

        // Update tooltip to reflect current status
        let tooltip = if is_tracking {
            "Velosi Tracker - Tracking Active"
        } else {
            "Velosi Tracker - Tracking Paused"
        };
        tray.set_tooltip(Some(tooltip)).map_err(|e| e.to_string())?;
    }

    Ok(())
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

async fn start_activity_tracking(app_handle: AppHandle) {
    let state: State<'_, AppState> = app_handle.state();
    let mut interval = interval(Duration::from_secs(5)); // Check every 5 seconds

    println!("Starting activity tracking loop...");

    loop {
        interval.tick().await;
        println!("🔄 Loop tick - checking tracking status...");

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
                        let new_entry = ActivityEntry {
                            id: Uuid::new_v4(),
                            start_time: now,
                            end_time: None,
                            app_name: current.app_name.clone(),
                            app_bundle_id: current.app_bundle_id.clone(),
                            window_title: current.window_title.clone(),
                            url: current.url.clone(),
                            category: ActivityCategory::from_app_name(
                                &current.app_name,
                                current.app_bundle_id.as_deref(),
                            ),
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

                    let new_entry = ActivityEntry {
                        id: Uuid::new_v4(),
                        start_time: Utc::now(),
                        end_time: None,
                        app_name: current.app_name.clone(),
                        app_bundle_id: current.app_bundle_id.clone(),
                        window_title: current.window_title.clone(),
                        url: current.url.clone(),
                        category: ActivityCategory::from_app_name(
                            &current.app_name,
                            current.app_bundle_id.as_deref(),
                        ),
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
            };

            app.manage(state);

            // Create initial tray menu
            let status_item =
                MenuItem::with_id(app, "status", "🟢 Tracking Active", false, None::<&str>)?;
            let toggle_item =
                MenuItem::with_id(app, "toggle", "Pause Tracking", true, None::<&str>)?;
            let dashboard_item =
                MenuItem::with_id(app, "dashboard", "Dashboard", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(
                app,
                &[&status_item, &toggle_item, &dashboard_item, &quit_item],
            )?;

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
            load_categories,
            load_app_mappings,
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
            update_activity_category
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
