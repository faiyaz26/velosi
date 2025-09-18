#[tauri::command]
pub async fn get_permission_status() -> Result<bool, String> {
    // This is a placeholder - implement based on your permission system
    // For now, return true as if permissions are granted
    Ok(true)
}

#[tauri::command]
pub async fn check_apple_events_permissions() -> Result<bool, String> {
    use std::process::Command;

    // Test both System Events and browser access permissions

    // First test System Events (general Apple Events permission)
    let system_events_script = r#"
        tell application "System Events"
            try
                set frontApp to first application process whose frontmost is true
                set appName to name of frontApp
                return "system_events_ok"
            on error errMsg
                return "system_events_error:" & errMsg
            end try
        end tell
    "#;

    let system_events_result = match Command::new("osascript")
        .arg("-e")
        .arg(system_events_script)
        .output()
    {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("System Events permission test: '{}'", result);
            output.status.success() && result == "system_events_ok"
        }
        Err(e) => {
            println!("Failed to run System Events test: {}", e);
            false
        }
    };

    if !system_events_result {
        println!("❌ System Events permission not granted");
        return Ok(false);
    }

    // Test browser-specific permissions (Chrome as example)
    let chrome_test_script = r#"
        tell application "Google Chrome"
            try
                if (count of windows) > 0 then
                    set currentTab to active tab of first window
                    return "chrome_access_ok"
                else
                    return "chrome_no_windows"
                end if
            on error errMsg
                if errMsg contains "not allowed" or errMsg contains "not authorized" then
                    return "chrome_permission_denied"
                else
                    return "chrome_error:" & errMsg
                end if
            end try
        end tell
    "#;

    let chrome_result = match Command::new("osascript")
        .arg("-e")
        .arg(chrome_test_script)
        .output()
    {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("Chrome permission test: '{}'", result);

            // Accept both successful access and "no windows" as permission granted
            output.status.success()
                && (result == "chrome_access_ok" || result == "chrome_no_windows")
        }
        Err(e) => {
            println!("Failed to run Chrome test: {}", e);
            false
        }
    };

    if !chrome_result {
        println!("❌ Chrome/Browser access permission not granted");
    }

    // Return true only if both System Events and browser access work
    let has_permissions = system_events_result && chrome_result;

    println!(
        "🔍 Permission check result: System Events: {}, Chrome: {}, Overall: {}",
        system_events_result, chrome_result, has_permissions
    );

    Ok(has_permissions)
}

#[tauri::command]
pub async fn trigger_apple_events_permission_request() -> Result<bool, String> {
    use std::process::Command;

    println!("🔄 Triggering Apple Events permission requests...");

    // First, trigger System Events permission
    let system_events_script = r#"
        tell application "System Events"
            try
                set frontApp to first application process whose frontmost is true
                set appName to name of frontApp
                return "system_events_requested"
            on error errMsg
                return "system_events_error:" & errMsg
            end try
        end tell
    "#;

    println!("📋 Requesting System Events permission...");
    match Command::new("osascript")
        .arg("-e")
        .arg(system_events_script)
        .output()
    {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("System Events request result: '{}'", result);
        }
        Err(e) => {
            println!("Failed to trigger System Events permission: {}", e);
        }
    }

    // Give the system a moment to process the permission
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    // Now trigger browser permission (Chrome as example)
    let chrome_permission_script = r#"
        tell application "Google Chrome"
            try
                -- Just check if we can access Chrome without actually doing anything
                set windowCount to count of windows
                return "chrome_permission_requested"
            on error errMsg
                return "chrome_error:" & errMsg
            end try
        end tell
    "#;

    println!("🌐 Requesting Chrome access permission...");
    match Command::new("osascript")
        .arg("-e")
        .arg(chrome_permission_script)
        .output()
    {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("Chrome permission request result: '{}'", result);
        }
        Err(e) => {
            println!("Failed to trigger Chrome permission: {}", e);
        }
    }

    // Give additional time for user to respond to dialogs
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;

    // Now check if we have all permissions
    let has_permissions = check_apple_events_permissions().await?;

    if has_permissions {
        println!("✅ All permissions granted successfully!");
    } else {
        println!("❌ Some permissions still missing. User may need to check System Settings.");
    }

    Ok(has_permissions)
}

#[tauri::command]
pub async fn test_chrome_access() -> Result<String, String> {
    use std::process::Command;

    println!("🧪 Testing Chrome access...");

    let chrome_test_script = r#"
        tell application "Google Chrome"
            try
                set windowCount to count of windows
                if windowCount > 0 then
                    set currentTab to active tab of first window
                    set tabURL to URL of currentTab
                    set tabTitle to title of currentTab
                    return "SUCCESS: " & tabTitle & " - " & tabURL
                else
                    return "SUCCESS: Chrome is running but no windows open"
                end if
            on error errMsg
                return "ERROR: " & errMsg
            end try
        end tell
    "#;

    match Command::new("osascript")
        .arg("-e")
        .arg(chrome_test_script)
        .output()
    {
        Ok(output) => {
            let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            println!("Chrome test result: '{}'", result);
            if !stderr.is_empty() {
                println!("Chrome test stderr: '{}'", stderr);
            }

            Ok(format!(
                "Status: {}, Output: '{}', Stderr: '{}'",
                output.status, result, stderr
            ))
        }
        Err(e) => {
            println!("Failed to run Chrome test: {}", e);
            Err(format!("Failed to execute test: {}", e))
        }
    }
}

#[tauri::command]
pub async fn open_automation_settings() -> Result<(), String> {
    use std::process::Command;

    // Open System Settings to the Privacy & Security > Automation section
    match Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Automation")
        .output()
    {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("Failed to open automation settings: {}", e);
            // Fallback: try to open the general Privacy & Security settings
            match Command::new("open")
                .arg("x-apple.systempreferences:com.apple.preference.security")
                .output()
            {
                Ok(_) => Ok(()),
                Err(e2) => Err(format!("Failed to open system settings: {}", e2)),
            }
        }
    }
}

#[tauri::command]
pub async fn reset_apple_events_permissions() -> Result<(), String> {
    use std::process::Command;

    // Get the app's bundle identifier - use the correct one from tauri.conf.json
    let bundle_id =
        std::env::var("TAURI_BUNDLE_IDENTIFIER").unwrap_or_else(|_| "com.velosi.app".to_string());

    // Use tccutil to reset automation permissions for our app
    match Command::new("tccutil")
        .arg("reset")
        .arg("AppleEvents")
        .arg(&bundle_id)
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                println!("Apple Events permissions reset successfully");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("Failed to reset permissions: {}", stderr))
            }
        }
        Err(e) => {
            println!("Failed to run tccutil: {}", e);
            Err(format!("Failed to reset permissions: {}. You may need to do this manually in System Settings.", e))
        }
    }
}