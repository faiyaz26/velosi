mod commands;
mod database;
mod migrations;
mod models;
mod tracker;
mod tray;

use chrono::Utc;
use std::sync::{Arc, Mutex};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};
use tokio::time::{interval, Duration, Instant};
use uuid::Uuid;

use database::Database;

// Helper function to handle pause operations
async fn handle_pause_operation(
    app_handle: AppHandle,
    duration_seconds: Option<u64>,
    operation_name: &str,
) {
    let state: State<'_, AppState> = app_handle.state();
    if let Err(e) = commands::pause_tracking(state, app_handle.clone(), duration_seconds).await {
        eprintln!("Failed to {}: {}", operation_name, e);
    }
}
use models::{ActivityCategory, ActivityEntry};
use tracker::{ActivityTracker, CurrentActivity};

// Application state
pub struct AppState {
    db: Arc<Database>,
    tracker: Arc<Mutex<ActivityTracker>>,
    is_tracking: Arc<Mutex<bool>>,
    pause_until: Arc<Mutex<Option<Instant>>>,
    is_paused_indefinitely: Arc<Mutex<bool>>,
    current_activity: Arc<Mutex<Option<CurrentActivity>>>,
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
                        let state: State<'_, AppState> = app_handle_clone.state();
                        if let Err(e) =
                            commands::resume_tracking(state, app_handle_clone.clone()).await
                        {
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
                current_activity: Arc::new(Mutex::new(None)),
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
                            if let Err(e) = commands::toggle_tracking(state, app_handle).await {
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
                            handle_pause_operation(
                                app_clone,
                                Some(60),
                                "pause tracking for 1 minute",
                            )
                            .await;
                        });
                    }
                    "pause_5min" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_pause_operation(
                                app_clone,
                                Some(300),
                                "pause tracking for 5 minutes",
                            )
                            .await;
                        });
                    }
                    "pause_30min" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_pause_operation(
                                app_clone,
                                Some(1800),
                                "pause tracking for 30 minutes",
                            )
                            .await;
                        });
                    }
                    "pause_1hour" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_pause_operation(
                                app_clone,
                                Some(3600),
                                "pause tracking for 1 hour",
                            )
                            .await;
                        });
                    }
                    "pause_until_tomorrow" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_pause_operation(
                                app_clone,
                                None,
                                "pause tracking until tomorrow",
                            )
                            .await;
                        });
                    }
                    "pause_indefinitely" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            handle_pause_operation(app_clone, None, "pause tracking indefinitely")
                                .await;
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
            commands::start_tracking,
            commands::stop_tracking,
            commands::get_tracking_status,
            commands::pause_tracking,
            commands::pause_tracking_for_duration,
            commands::resume_tracking,
            commands::toggle_tracking,
            commands::get_pause_status,
            commands::get_current_activity,
            commands::set_current_activity,
            commands::get_activities_by_date,
            commands::get_activities_by_date_range,
            commands::get_activity_summary,
            commands::get_timeline_data,
            commands::get_categories,
            commands::load_categories,
            commands::add_category,
            commands::update_category,
            commands::delete_category,
            commands::get_app_mappings,
            commands::get_url_mappings,
            commands::add_app_mapping,
            commands::update_app_mapping,
            commands::delete_app_mapping,
            commands::show_main_window,
            commands::hide_main_window,
            commands::hide_window,
            commands::close_main_window,
            commands::pause_tracking_until_tomorrow,
            commands::pause_tracking_indefinitely,
            commands::resume_tracking_now,
            commands::update_activity_category,
            commands::get_permission_status,
            commands::remove_app_mapping,
            commands::add_url_mapping,
            commands::remove_url_mapping
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
