use chrono::Utc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::time::{interval, Duration, Instant};
use uuid::Uuid;

use crate::commands;
use crate::database::Database;
use crate::focus_mode::FocusMode;
use crate::models::{ActivityCategory, ActivityEntry};
use crate::AppState;

/// Helper to handle pause operations initiated from the tray/menu.
/// Delegates to the commands::pause_tracking implementation and logs failures.
pub async fn handle_pause_operation(
    app_handle: AppHandle,
    duration_seconds: Option<u64>,
    operation_name: &str,
) {
    let state: State<'_, AppState> = app_handle.state();
    if let Err(e) = commands::pause_tracking(state, app_handle.clone(), duration_seconds).await {
        eprintln!("Failed to {}: {}", operation_name, e);
    }
}

/// Helper function to categorize activity based on app name and URL with database mappings only
pub async fn categorize_activity(
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

/// Main activity tracking loop that runs continuously in the background
pub async fn start_activity_tracking(app_handle: AppHandle) {
    let state: State<'_, AppState> = app_handle.state();

    println!("Starting activity tracking loop...");

    loop {
        // Dynamic interval based on focus mode status
        let check_interval = {
            let focus_enabled = state.focus_mode_enabled.lock().unwrap();
            if *focus_enabled {
                Duration::from_millis(500) // Much more frequent when focus mode is active (0.5 seconds)
            } else {
                Duration::from_secs(5) // Normal interval
            }
        };

        let mut interval = interval(check_interval);
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

            // Check if this app was recently hidden - if so, skip it and let the system settle
            {
                let now = tokio::time::Instant::now();
                let mut hidden_apps = state.recently_hidden_apps.lock().unwrap();

                // Clean up old entries (older than 3 seconds)
                hidden_apps
                    .retain(|_, &mut hidden_time| now.duration_since(hidden_time).as_secs() < 3);

                // If this app was recently hidden (within 2 seconds), skip it
                if let Some(&hidden_time) = hidden_apps.get(&current.app_name) {
                    if now.duration_since(hidden_time).as_secs() < 2 {
                        println!(
                            "App '{}' was recently hidden, skipping to let system settle",
                            current.app_name
                        );
                        continue;
                    }
                }
            }

            // Check focus mode before proceeding with activity tracking
            let focus_mode = FocusMode::new(app_handle.clone());
            match focus_mode
                .check_and_block_app(&current.app_name, current.app_bundle_id.as_deref())
                .await
            {
                Ok(is_allowed) => {
                    if !is_allowed {
                        println!("App '{}' is blocked by focus mode", current.app_name);
                        // Give the system a moment to update the frontmost app after hiding
                        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                        continue; // Skip tracking this blocked app and recheck immediately
                    }
                }
                Err(e) => {
                    eprintln!("Error checking focus mode: {}", e);
                }
            }

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
