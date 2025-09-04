mod database;
mod models;
mod tracker;

use chrono::{NaiveDate, Utc};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};
use tokio::time::{interval, Duration};
use uuid::Uuid;

use database::Database;
use models::{ActivityCategory, ActivityEntry, ActivitySummary, TimelineData};
use tracker::{ActivityTracker, CurrentActivity};

// Application state
pub struct AppState {
    db: Arc<Database>,
    tracker: Arc<Mutex<ActivityTracker>>,
    is_tracking: Arc<Mutex<bool>>,
}

// Tauri commands
#[tauri::command]
async fn start_tracking(state: State<'_, AppState>) -> Result<(), String> {
    let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
    *is_tracking = true;
    Ok(())
}

#[tauri::command]
async fn stop_tracking(state: State<'_, AppState>) -> Result<(), String> {
    let mut is_tracking = state.is_tracking.lock().map_err(|e| e.to_string())?;
    *is_tracking = false;
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

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

async fn setup_database() -> Result<Database, Box<dyn std::error::Error>> {
    // Create data directory if it doesn't exist
    let app_data_dir = dirs::data_dir()
        .ok_or("Failed to get data directory")?
        .join("velosi-tracker");

    // Use std::fs to create directory synchronously since we need it immediately
    std::fs::create_dir_all(&app_data_dir)?;

    let db_path = app_data_dir.join("activities.db");
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
            greet,
            start_tracking,
            stop_tracking,
            get_tracking_status,
            get_current_activity,
            test_permissions,
            update_user_activity,
            get_activities_by_date,
            get_activity_summary,
            get_recent_activities,
            get_timeline_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
