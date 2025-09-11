use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::tray::MouseButton;
use tauri::tray::MouseButtonState;
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;
use tauri::{Manager, State};

use crate::AppState;
use crate::{activity, commands};

pub struct TrayManager;

impl TrayManager {
    pub fn create_menu<T>(
        app: &T,
        is_tracking: bool,
        pause_info: Option<(u64, bool)>, // (remaining_seconds, is_indefinite)
        is_focus_mode_enabled: Option<bool>, // Focus mode status
    ) -> Result<tauri::menu::Menu<tauri::Wry>, String>
    where
        T: tauri::Manager<tauri::Wry>,
    {
        let tracking_status_text = if is_tracking {
            "🟢 Tracking Active".to_string()
        } else if let Some((_, is_indefinite)) = pause_info {
            if is_indefinite {
                "🔴 Paused Indefinitely".to_string()
            } else {
                "🔴 Paused (Timed)".to_string()
            }
        } else {
            "🔴 Tracking Paused".to_string()
        };

        let focus_status_text = if let Some(focus_enabled) = is_focus_mode_enabled {
            if focus_enabled {
                "🎯 Focus Mode: ON".to_string()
            } else {
                "🎯 Focus Mode: OFF".to_string()
            }
        } else {
            "🎯 Focus Mode: Unknown".to_string()
        };

        let tracking_toggle_text = if is_tracking {
            "Pause Tracking"
        } else {
            "Resume Tracking"
        };

        let focus_toggle_text = if is_focus_mode_enabled.unwrap_or(false) {
            "Disable Focus Mode"
        } else {
            "Enable Focus Mode"
        };

        // Create menu items
        let tracking_status_item = MenuItem::with_id(
            app,
            "tracking_status",
            tracking_status_text,
            false,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        let focus_status_item =
            MenuItem::with_id(app, "focus_status", focus_status_text, false, None::<&str>)
                .map_err(|e| e.to_string())?;
        let tracking_toggle_item =
            MenuItem::with_id(app, "toggle", tracking_toggle_text, true, None::<&str>)
                .map_err(|e| e.to_string())?;
        let focus_toggle_item =
            MenuItem::with_id(app, "focus_toggle", focus_toggle_text, true, None::<&str>)
                .map_err(|e| e.to_string())?;

        let dashboard_item = MenuItem::with_id(app, "dashboard", "Dashboard", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
            .map_err(|e| e.to_string())?;

        // Create pause submenu items only when tracking is active
        let menu = if is_tracking {
            let pause_submenu = Self::create_pause_submenu(app)?;

            Menu::with_items(
                app,
                &[
                    &tracking_status_item,
                    &focus_status_item,
                    &pause_submenu,
                    &focus_toggle_item,
                    &dashboard_item,
                    &quit_item,
                ],
            )
            .map_err(|e| e.to_string())?
        } else {
            Menu::with_items(
                app,
                &[
                    &tracking_status_item,
                    &focus_status_item,
                    &tracking_toggle_item,
                    &focus_toggle_item,
                    &dashboard_item,
                    &quit_item,
                ],
            )
            .map_err(|e| e.to_string())?
        };

        Ok(menu)
    }

    /// Create the tray icon, wire menu and icon events, and build the tray.
    pub fn create_tray(
        app: &AppHandle,
        is_tracking: bool,
        pause_info: Option<(u64, bool)>,
        is_focus_mode_enabled: Option<bool>,
    ) -> Result<(), String> {
        // Create the menu first
        let menu = Self::create_menu(app, is_tracking, pause_info, is_focus_mode_enabled)?;

        // Build tray icon and wire event handlers
        let _tray = TrayIconBuilder::with_id("main")
            .tooltip("Velosi - Tracking Active")
            .icon(
                app.default_window_icon()
                    .ok_or("missing default icon")?
                    .clone(),
            )
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
                "focus_toggle" => {
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let app_handle = app_clone.clone();
                        let state: State<'_, AppState> = app_clone.state();

                        // Get current focus mode status
                        let current_status = {
                            let focus_enabled = state.focus_mode_enabled.lock().unwrap();
                            *focus_enabled
                        };

                        // Toggle focus mode
                        if current_status {
                            if let Err(e) =
                                commands::disable_focus_mode(state.clone(), app_handle.clone())
                                    .await
                            {
                                eprintln!("Failed to disable focus mode: {}", e);
                            }
                        } else {
                            if let Err(e) =
                                commands::enable_focus_mode(state.clone(), app_handle.clone()).await
                            {
                                eprintln!("Failed to enable focus mode: {}", e);
                            }
                        }

                        // Update tray menu to reflect new status
                        let is_tracking = {
                            let tracking = state.is_tracking.lock().unwrap();
                            *tracking
                        };
                        let new_focus_status = {
                            let focus_enabled = state.focus_mode_enabled.lock().unwrap();
                            *focus_enabled
                        };
                        if let Err(e) = Self::update_menu(
                            &app_handle,
                            is_tracking,
                            None,
                            Some(new_focus_status),
                        )
                        .await
                        {
                            eprintln!("Failed to update tray menu: {}", e);
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
                        activity::handle_pause_operation(
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
                        activity::handle_pause_operation(
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
                        activity::handle_pause_operation(
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
                        activity::handle_pause_operation(
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
                        activity::handle_pause_operation(
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
                        activity::handle_pause_operation(
                            app_clone,
                            None,
                            "pause tracking indefinitely",
                        )
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
            .build(app)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn update_menu(
        app_handle: &AppHandle,
        is_tracking: bool,
        pause_info: Option<(u64, bool)>,
        is_focus_mode_enabled: Option<bool>,
    ) -> Result<(), String> {
        // Use the passed pause_info instead of accessing state directly
        let menu = Self::create_menu(app_handle, is_tracking, pause_info, is_focus_mode_enabled)?;

        // Update the tray menu
        if let Some(tray) = app_handle.tray_by_id("main") {
            tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;

            // Update tooltip to reflect current status
            let tooltip = if is_tracking {
                "Velosi - Tracking Active".to_string()
            } else if let Some((_, is_indefinite)) = pause_info {
                if is_indefinite {
                    "Velosi - Paused Indefinitely".to_string()
                } else {
                    "Velosi - Paused (Timed)".to_string()
                }
            } else {
                "Velosi - Tracking Paused".to_string()
            };
            tray.set_tooltip(Some(&tooltip))
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }
    fn create_pause_submenu<T>(app: &T) -> Result<tauri::menu::Submenu<tauri::Wry>, String>
    where
        T: tauri::Manager<tauri::Wry>,
    {
        // Create pause submenu items
        let pause_1min =
            MenuItem::with_id(app, "pause_1min", "Pause for 1 minute", true, None::<&str>)
                .map_err(|e| e.to_string())?;
        let pause_5min =
            MenuItem::with_id(app, "pause_5min", "Pause for 5 minutes", true, None::<&str>)
                .map_err(|e| e.to_string())?;
        let pause_30min = MenuItem::with_id(
            app,
            "pause_30min",
            "Pause for 30 minutes",
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        let pause_1hour =
            MenuItem::with_id(app, "pause_1hour", "Pause for 1 hour", true, None::<&str>)
                .map_err(|e| e.to_string())?;
        let pause_until_tomorrow = MenuItem::with_id(
            app,
            "pause_until_tomorrow",
            "Pause until tomorrow",
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        let pause_indefinitely = MenuItem::with_id(
            app,
            "pause_indefinitely",
            "Pause indefinitely",
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;

        // Create pause submenu
        let pause_submenu = Submenu::with_id_and_items(
            app,
            "pause_menu",
            "Pause tracking options",
            true,
            &[
                &pause_1min,
                &pause_5min,
                &pause_30min,
                &pause_1hour,
                &pause_until_tomorrow,
                &pause_indefinitely,
            ],
        )
        .map_err(|e| e.to_string())?;

        Ok(pause_submenu)
    }
}
