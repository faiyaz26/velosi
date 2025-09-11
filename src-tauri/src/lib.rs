mod activity;
mod commands;
mod database;
mod focus_mode;
mod migrations;
mod models;
mod tracker;
mod tray;

use std::sync::{Arc, Mutex};
use tauri::{Manager, WindowEvent};
use tokio::time::Instant;

use database::Database;

use tracker::{ActivityTracker, CurrentActivity};

// Application state
pub struct AppState {
    db: Arc<Database>,
    tracker: Arc<Mutex<ActivityTracker>>,
    is_tracking: Arc<Mutex<bool>>,
    pause_until: Arc<Mutex<Option<Instant>>>,
    // is_paused_indefinitely: Arc<Mutex<bool>>, // removed - not used
    current_activity: Arc<Mutex<Option<CurrentActivity>>>,
    // Focus mode state
    focus_mode_enabled: Arc<Mutex<bool>>,
    focus_mode_allowed_categories: Arc<Mutex<Vec<String>>>,
    recently_blocked_apps: Arc<Mutex<std::collections::HashMap<String, Instant>>>,
    temporarily_allowed_apps: Arc<Mutex<std::collections::HashMap<String, Instant>>>,
    // App category cache for faster lookups (app_name -> category_id)
    app_category_cache: Arc<Mutex<std::collections::HashMap<String, String>>>,
    // Recently hidden apps to avoid tracking them immediately after hiding
    recently_hidden_apps: Arc<Mutex<std::collections::HashMap<String, Instant>>>,
}

// =============================================================================
// DATABASE SETUP
// =============================================================================

// Initialize the SQLite database and create necessary tables
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

// =============================================================================
// MAIN APPLICATION SETUP
// =============================================================================

// Application entry point
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

            // Load focus mode preferences from database
            let db_arc = Arc::new(db);
            let focus_enabled = rt
                .block_on(db_arc.get_focus_mode_enabled())
                .unwrap_or(false);
            let allowed_categories = rt
                .block_on(db_arc.get_focus_mode_allowed_categories())
                .unwrap_or_default();

            // Initialize application state
            let state = AppState {
                db: db_arc,
                tracker: Arc::new(Mutex::new(ActivityTracker::new())),
                is_tracking: Arc::new(Mutex::new(true)), // Start tracking by default
                pause_until: Arc::new(Mutex::new(None)),
                current_activity: Arc::new(Mutex::new(None)),
                // Focus mode state (loaded from database)
                focus_mode_enabled: Arc::new(Mutex::new(focus_enabled)),
                focus_mode_allowed_categories: Arc::new(Mutex::new(allowed_categories)),
                recently_blocked_apps: Arc::new(Mutex::new(std::collections::HashMap::new())),
                temporarily_allowed_apps: Arc::new(Mutex::new(std::collections::HashMap::new())),
                app_category_cache: Arc::new(Mutex::new(std::collections::HashMap::new())),
                recently_hidden_apps: Arc::new(Mutex::new(std::collections::HashMap::new())),
            };

            app.manage(state);

            // Setup the tray icon and handlers via the tray module
            tray::TrayManager::create_tray(&app_handle, true, None)?;

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
                rt.block_on(activity::start_activity_tracking(handle_clone));
            });
            println!("✅ Activity tracking task spawned!");

            Ok(())
        })
        // Register Tauri commands that can be called from the frontend
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
            commands::remove_url_mapping,
            // Focus mode commands
            commands::enable_focus_mode,
            commands::disable_focus_mode,
            commands::get_focus_mode_status,
            commands::set_focus_mode_categories,
            commands::get_focus_mode_categories,
            commands::check_app_focus_allowed,
            commands::temporarily_allow_app,
            commands::get_focus_mode_allowed_apps,
            commands::remove_focus_mode_allowed_app,
            commands::show_focus_overlay,
            commands::hide_focus_overlay
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
