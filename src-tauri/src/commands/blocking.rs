use tauri::{AppHandle, Emitter, State};

use crate::AppState;

#[tauri::command]
pub async fn get_app_blocking_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .db
        .get_app_blocking_enabled()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_app_blocking_enabled(
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), String> {
    state
        .db
        .set_app_blocking_enabled(enabled)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_website_blocking_enabled(state: State<'_, AppState>) -> Result<bool, String> {
    state
        .db
        .get_website_blocking_enabled()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_website_blocking_enabled(
    state: State<'_, AppState>,
    app_handle: AppHandle,
    enabled: bool,
) -> Result<(), String> {
    // Save the preference to database
    state
        .db
        .set_website_blocking_enabled(enabled)
        .await
        .map_err(|e| e.to_string())?;

    // If enabling, start website blocking; if disabling, stop it
    if enabled {
        if let Err(e) = start_website_blocking_internal(&state, &app_handle).await {
            println!("⚠️ Warning: Failed to start website blocking: {}", e);
            // Don't fail the entire operation if website blocking fails to start
        }
    } else {
        if let Err(e) = stop_website_blocking_internal(&state).await {
            println!("⚠️ Warning: Failed to stop website blocking: {}", e);
            // Don't fail the entire operation if website blocking fails to stop
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn start_website_blocker(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    start_website_blocking_internal(&state, &app_handle).await?;
    Ok("Website blocking started".to_string())
}

#[tauri::command]
pub async fn stop_website_blocker(state: State<'_, AppState>) -> Result<String, String> {
    stop_website_blocking_internal(&state).await?;
    Ok("Website blocking stopped".to_string())
}

#[tauri::command]
pub async fn get_website_blocker_status(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let app_state = state.inner();

    // Get the website blocker instance
    let blocker = {
        let website_blocker = app_state
            .website_blocker
            .lock()
            .map_err(|e| e.to_string())?;

        website_blocker.clone()
    }; // MutexGuard dropped here

    let is_active = if let Some(ref blocker) = blocker {
        blocker.is_blocking().await
    } else {
        false
    };

    // Check if system proxy is enabled
    let system_proxy_enabled = if let Some(ref blocker) = blocker {
        match blocker.is_system_proxy_enabled().await {
            Ok(enabled) => enabled,
            Err(e) => {
                println!("⚠️ Failed to check system proxy status: {}", e);
                false
            }
        }
    } else {
        false
    };

    // Get proxy info if available
    let (proxy_address, proxy_port) = if let Some(blocker) = &blocker {
        let (addr, port) = blocker.get_proxy_info().await;
        (Some(addr), Some(port))
    } else {
        (None, None)
    };

    Ok(serde_json::json!({
        "running": is_active,
        "system_proxy_enabled": system_proxy_enabled,
        "method": "local_proxy",
        "platform": std::env::consts::OS,
        "proxy_address": proxy_address,
        "proxy_port": proxy_port
    }))
}

#[tauri::command]
pub async fn check_website_blocking_permissions() -> Result<serde_json::Value, String> {
    let blocker = crate::local_proxy_blocker::LocalProxyBlocker::new();

    match blocker.check_proxy_permissions().await {
        Ok(()) => Ok(serde_json::json!({
            "has_permission": true,
            "message": "Local proxy website blocking is available"
        })),
        Err(e) => Ok(serde_json::json!({
            "has_permission": false,
            "message": e,
            "platform": std::env::consts::OS
        })),
    }
}

#[tauri::command]
pub async fn get_proxy_setup_info(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let app_state = state.inner();

    // Get the website blocker instance
    let blocker = {
        let website_blocker = app_state
            .website_blocker
            .lock()
            .map_err(|e| e.to_string())?;

        website_blocker.clone()
    }; // MutexGuard dropped here

    if let Some(blocker) = blocker {
        let (address, port) = blocker.get_proxy_info().await;
        let blocked_domains = blocker.get_blocked_domains().await;

        Ok(serde_json::json!({
            "proxy_address": address,
            "proxy_port": port,
            "blocked_domains": blocked_domains,
            "setup_instructions": {
                "macos": [
                    "1. Go to System Preferences > Network",
                    "2. Select your active network connection",
                    "3. Click 'Advanced...' > 'Proxies'",
                    "4. Check 'Web Proxy (HTTP)' and 'Secure Web Proxy (HTTPS)'",
                    format!("5. Enter {}:{}", address, port)
                ],
                "windows": [
                    "1. Go to Settings > Network & Internet > Proxy",
                    "2. Enable 'Use a proxy server'",
                    format!("3. Enter {}:{}", address, port)
                ]
            }
        }))
    } else {
        Err("Website blocker not initialized".to_string())
    }
}

#[tauri::command]
pub async fn initialize_proxy_server(
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    let app_state = state.inner();

    // Check if already initialized
    {
        let website_blocker = app_state
            .website_blocker
            .lock()
            .map_err(|e| e.to_string())?;

        if website_blocker.is_some() {
            return Ok(serde_json::json!({
                "success": true,
                "message": "Proxy server already initialized"
            }));
        }
    }

    // Initialize and start the proxy server
    println!("🚀 Initializing proxy server...");
    let proxy_blocker = crate::local_proxy_blocker::LocalProxyBlocker::with_app_handle(app_handle)
        .with_database(app_state.db.clone());

    // Start the proxy server
    if let Err(e) = proxy_blocker.start_proxy_server().await {
        return Err(format!("Failed to start proxy server: {}", e));
    }

    let (addr, port) = proxy_blocker.get_proxy_info().await;
    println!("✅ Proxy server started at {}:{}", addr, port);

    // Store the initialized blocker in the state
    {
        let mut website_blocker = app_state
            .website_blocker
            .lock()
            .map_err(|e| e.to_string())?;
        *website_blocker = Some(proxy_blocker);
    }

    Ok(serde_json::json!({
        "success": true,
        "message": "Proxy server initialized successfully",
        "proxy_address": addr,
        "proxy_port": port
    }))
}

#[tauri::command]
pub async fn get_proxy_port(state: State<'_, AppState>) -> Result<u16, String> {
    let db = &state.db;
    db.get_proxy_port()
        .await
        .map_err(|e| format!("Failed to get proxy port: {}", e))
}

#[tauri::command]
pub async fn set_proxy_port(
    state: State<'_, AppState>,
    port: u16,
    app_handle: AppHandle,
) -> Result<(), String> {
    let db = &state.db;
    db.set_proxy_port(port)
        .await
        .map_err(|e| format!("Failed to set proxy port: {}", e))?;

    // Emit event to notify frontend of proxy port change
    app_handle
        .emit("proxy-port-changed", port)
        .map_err(|e| format!("Failed to emit event: {}", e))?;

    Ok(())
}

// Internal helper functions for website blocking
async fn start_website_blocking_internal(
    state: &State<'_, AppState>,
    _app_handle: &AppHandle,
) -> Result<(), String> {
    // Get URLs to block from database
    let url_mappings = state
        .db
        .get_url_mappings()
        .await
        .map_err(|e| e.to_string())?;

    println!("🔍 Total URL mappings found: {}", url_mappings.len());
    for mapping in &url_mappings {
        println!(
            "  📋 URL: {} -> Category: {}",
            mapping.url_pattern, mapping.category_id
        );
    }

    // Filter for distracting categories (social and entertainment)
    let mut urls_to_block: Vec<String> = url_mappings
        .into_iter()
        .filter(|mapping| mapping.category_id == "social" || mapping.category_id == "entertainment")
        .map(|mapping| mapping.url_pattern)
        .collect();

    // Add some common test domains for debugging if the list is empty
    if urls_to_block.is_empty() {
        println!("⚠️ No URLs found in database for social/entertainment categories. Adding test domains for debugging.");
        urls_to_block.extend(vec![
            "facebook.com".to_string(),
            "twitter.com".to_string(),
            "instagram.com".to_string(),
            "youtube.com".to_string(),
            "tiktok.com".to_string(),
            "reddit.com".to_string(),
            "linkedin.com".to_string(),
        ]);
    }

    println!(
        "🚫 URLs to block (social/entertainment): {:?}",
        urls_to_block
    );

    let app_state = state.inner();

    // Get the website blocker instance (should be initialized at startup)
    let blocker = {
        let website_blocker = app_state
            .website_blocker
            .lock()
            .map_err(|e| e.to_string())?;

        website_blocker.clone()
    }; // MutexGuard dropped here

    if let Some(blocker) = blocker {
        blocker.enable_website_blocking(urls_to_block).await?;
        Ok(())
    } else {
        Err("Website blocker not initialized at startup".to_string())
    }
}

async fn stop_website_blocking_internal(state: &State<'_, AppState>) -> Result<(), String> {
    let app_state = state.inner();

    // Get the website blocker instance
    let blocker = {
        let website_blocker = app_state
            .website_blocker
            .lock()
            .map_err(|e| e.to_string())?;

        website_blocker.clone()
    }; // MutexGuard dropped here

    if let Some(blocker) = blocker {
        blocker.disable_website_blocking().await?;
        Ok(())
    } else {
        Err("Website blocker not initialized at startup".to_string())
    }
}