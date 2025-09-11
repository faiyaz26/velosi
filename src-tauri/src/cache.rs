use crate::AppState;
use tauri::{AppHandle, Listener, Manager};

/// Cache manager for focus mode functionality
pub struct CacheManager {
    app_handle: AppHandle,
}

impl CacheManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    /// Check if an app is allowed using cache-first approach
    pub async fn is_app_allowed_cached(&self, app_name: &str) -> Result<bool, String> {
        let state: tauri::State<AppState> = self.app_handle.state();
        let now = chrono::Utc::now().timestamp();

        // Check cache first
        {
            let cache = state
                .focus_mode_allowed_apps_cache
                .lock()
                .map_err(|e| e.to_string())?;
            if let Some(&expires_at) = cache.get(app_name) {
                match expires_at {
                    Some(expiry) if expiry > now => {
                        println!(
                            "✅ App '{}' allowed from cache (expires: {})",
                            app_name, expiry
                        );
                        return Ok(true);
                    }
                    None => {
                        println!("✅ App '{}' allowed from cache (indefinite)", app_name);
                        return Ok(true);
                    }
                    Some(expiry) => {
                        println!("⏰ App '{}' cache expired (was: {})", app_name, expiry);
                        // Will fall through to refresh from DB
                    }
                }
            }
        }

        // Not in cache or expired, check database and update cache
        let is_allowed = state
            .db
            .is_focus_mode_app_allowed(app_name)
            .await
            .map_err(|e| e.to_string())?;

        // Update cache based on DB result
        if is_allowed {
            // Get the actual expiry from DB
            let allowed_apps = state
                .db
                .get_focus_mode_allowed_apps_with_expiry()
                .await
                .map_err(|e| e.to_string())?;

            let mut cache = state
                .focus_mode_allowed_apps_cache
                .lock()
                .map_err(|e| e.to_string())?;
            for (app, expiry) in allowed_apps {
                cache.insert(app, expiry);
            }
        }

        Ok(is_allowed)
    }

    /// Get app mappings with caching
    pub async fn get_app_mappings_cached(&self) -> Result<Vec<crate::models::AppMapping>, String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        // Check cache first
        {
            let cache = state.app_mappings_cache.lock().map_err(|e| e.to_string())?;
            if let Some(ref mappings) = *cache {
                println!("📋 Using cached app mappings ({} entries)", mappings.len());
                return Ok(mappings.clone());
            }
        }

        // Not cached, fetch from DB and cache
        println!("📋 Loading app mappings from database");
        let mappings = state
            .db
            .get_app_mappings()
            .await
            .map_err(|e| e.to_string())?;

        // Update cache
        {
            let mut cache = state.app_mappings_cache.lock().map_err(|e| e.to_string())?;
            *cache = Some(mappings.clone());
        }

        Ok(mappings)
    }

    /// Update allowed apps cache entry
    pub fn update_allowed_apps_cache(
        &self,
        app_name: &str,
        expires_at: Option<i64>,
    ) -> Result<(), String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        let mut cache = state
            .focus_mode_allowed_apps_cache
            .lock()
            .map_err(|e| e.to_string())?;

        cache.insert(app_name.to_string(), expires_at);
        println!(
            "✅ Updated cache for '{}' with expires_at: {:?}",
            app_name, expires_at
        );

        Ok(())
    }

    /// Remove app from allowed apps cache
    pub fn remove_from_allowed_apps_cache(&self, app_name: &str) -> Result<(), String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        let mut cache = state
            .focus_mode_allowed_apps_cache
            .lock()
            .map_err(|e| e.to_string())?;

        cache.remove(app_name);
        println!("❌ Removed '{}' from allowed apps cache", app_name);

        Ok(())
    }

    /// Clear app category cache for a specific app
    pub fn clear_app_category_cache(&self, app_name: &str) -> Result<(), String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        let mut cache = state.app_category_cache.lock().map_err(|e| e.to_string())?;
        cache.remove(app_name);
        println!("🗑️ Cleared category cache for '{}'", app_name);

        Ok(())
    }

    /// Clear all app mappings cache
    pub fn clear_app_mappings_cache(&self) -> Result<(), String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        let mut cache = state.app_mappings_cache.lock().map_err(|e| e.to_string())?;
        *cache = None;
        println!("🗑️ Cleared app mappings cache");

        Ok(())
    }

    /// Update focus mode enabled cache
    pub fn update_focus_mode_enabled_cache(&self, enabled: bool) -> Result<(), String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        let mut focus_enabled = state.focus_mode_enabled.lock().map_err(|e| e.to_string())?;
        *focus_enabled = enabled;
        println!("🔄 Updated focus mode enabled cache: {}", enabled);

        Ok(())
    }

    /// Update allowed categories cache
    pub fn update_allowed_categories_cache(&self, categories: Vec<String>) -> Result<(), String> {
        let state: tauri::State<AppState> = self.app_handle.state();

        let mut allowed_categories = state
            .focus_mode_allowed_categories
            .lock()
            .map_err(|e| e.to_string())?;
        *allowed_categories = categories.clone();
        println!("🔄 Updated allowed categories cache: {:?}", categories);

        Ok(())
    }
}

/// Event listener for cache invalidation (synchronous version)
pub fn setup_cache_listeners_sync(app_handle: AppHandle) {
    let app_handle_clone = app_handle.clone();

    app_handle.listen("focus-cache-invalidate", move |event| {
        let app_handle = app_handle_clone.clone();
        let payload = event.payload().to_string();

        // Use a thread to handle the async cache invalidation
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                if let Err(e) = handle_cache_invalidation(app_handle, &payload).await {
                    eprintln!("Error handling cache invalidation: {}", e);
                }
            });
        });
    });
}

/// Handle cache invalidation events
async fn handle_cache_invalidation(app_handle: AppHandle, payload: &str) -> Result<(), String> {
    let cache_manager = CacheManager::new(app_handle);

    // Parse the event payload
    let event_data: serde_json::Value = serde_json::from_str(payload)
        .map_err(|e| format!("Failed to parse cache invalidation event: {}", e))?;

    let event_type = event_data["type"].as_str().unwrap_or("");

    match event_type {
        "focus_mode_enabled_changed" => {
            println!("🔄 Cache invalidation: focus mode enabled changed");
            if let Some(enabled) = event_data["enabled"].as_bool() {
                cache_manager.update_focus_mode_enabled_cache(enabled)?;
            }
        }
        "allowed_categories_changed" => {
            println!("🔄 Cache invalidation: allowed categories changed");
            if let Some(categories_val) = event_data["categories"].as_array() {
                let categories: Vec<String> = categories_val
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                cache_manager.update_allowed_categories_cache(categories)?;
            }

            // Clear app mappings cache since category changes affect app blocking
            cache_manager.clear_app_mappings_cache()?;
        }
        "allowed_apps_changed" => {
            println!("🔄 Cache invalidation: allowed apps changed");
            println!("📄 Event data: {}", event_data);

            if let Some(app_name) = event_data["app_name"].as_str() {
                if event_data["removed"].as_bool().unwrap_or(false) {
                    // Remove from cache
                    cache_manager.remove_from_allowed_apps_cache(app_name)?;
                } else {
                    // Update cache
                    let expires_at = event_data["expires_at"].as_i64();
                    cache_manager.update_allowed_apps_cache(app_name, expires_at)?;
                }

                // Also clear app category cache for this specific app
                cache_manager.clear_app_category_cache(app_name)?;
            }
        }
        _ => {
            println!("⚠️ Unknown cache invalidation event type: {}", event_type);
        }
    }

    Ok(())
}
