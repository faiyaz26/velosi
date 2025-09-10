use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::AppHandle;

pub struct TrayManager;

impl TrayManager {
    pub fn create_menu<T>(
        app: &T,
        is_tracking: bool,
        pause_info: Option<(u64, bool)>, // (remaining_seconds, is_indefinite)
    ) -> Result<tauri::menu::Menu<tauri::Wry>, String>
    where
        T: tauri::Manager<tauri::Wry>,
    {
        let status_text = if is_tracking {
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
        let toggle_text = if is_tracking {
            "Pause Tracking"
        } else {
            "Resume Tracking"
        };

        // Create menu items
        let status_item = MenuItem::with_id(app, "status", status_text, false, None::<&str>)
            .map_err(|e| e.to_string())?;
        let toggle_item = MenuItem::with_id(app, "toggle", toggle_text, true, None::<&str>)
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
                &[&status_item, &pause_submenu, &dashboard_item, &quit_item],
            )
            .map_err(|e| e.to_string())?
        } else {
            Menu::with_items(
                app,
                &[&status_item, &toggle_item, &dashboard_item, &quit_item],
            )
            .map_err(|e| e.to_string())?
        };

        Ok(menu)
    }

    pub async fn update_menu(
        app_handle: &AppHandle,
        is_tracking: bool,
        pause_info: Option<(u64, bool)>,
    ) -> Result<(), String> {
        // Use the passed pause_info instead of accessing state directly
        let menu = Self::create_menu(app_handle, is_tracking, pause_info)?;

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
            "Pause Options",
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
