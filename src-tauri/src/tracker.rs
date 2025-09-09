use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use url::Url;

#[cfg(target_os = "macos")]
use std::process::Command;

#[cfg(target_os = "windows")]
use std::ffi::CStr;
#[cfg(target_os = "windows")]
use std::ptr;
#[cfg(target_os = "windows")]
use winapi::um::handleapi::CloseHandle;
#[cfg(target_os = "windows")]
use winapi::um::processthreadsapi::OpenProcess;
#[cfg(target_os = "windows")]
use winapi::um::psapi::GetModuleFileNameExA;
#[cfg(target_os = "windows")]
use winapi::um::winnt::PROCESS_QUERY_INFORMATION;
#[cfg(target_os = "windows")]
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextA, GetWindowThreadProcessId};

use crate::models::SegmentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentActivity {
    pub app_name: String,
    pub app_bundle_id: Option<String>,
    pub window_title: String,
    pub url: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub segment_info: Option<SegmentInfo>, // New: detailed segment information
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentInfo {
    pub segment_type: SegmentType,
    pub title: String,             // Tab title, file name, etc.
    pub url: Option<String>,       // URL for browser tabs
    pub file_path: Option<String>, // File path for editors
    pub metadata: Option<String>,  // Additional metadata
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    pub last_input_time: SystemTime,
    pub is_active: bool,
}

pub struct ActivityTracker {
    user_activity: UserActivity,
    inactive_threshold: Duration,
}

impl ActivityTracker {
    pub fn new() -> Self {
        Self {
            user_activity: UserActivity {
                last_input_time: SystemTime::now(),
                is_active: true,
            },
            inactive_threshold: Duration::from_secs(30 * 60), // 30 minutes
        }
    }

    pub fn get_current_activity(&mut self) -> Option<CurrentActivity> {
        println!("Getting current activity...");
        if !self.should_track() {
            println!("Should not track, returning None");
            return None;
        }

        // Use platform-specific implementation to get real activity
        #[cfg(target_os = "macos")]
        {
            println!("Getting real macOS activity");
            self.get_current_activity_macos()
        }

        #[cfg(target_os = "windows")]
        {
            println!("Getting real Windows activity");
            self.get_current_activity_windows()
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            println!("Platform not supported for activity tracking");
            None
        }
    }

    #[cfg(target_os = "macos")]
    pub fn check_accessibility_permissions(&self) -> bool {
        // Simple test to see if we have accessibility permissions
        let test_script = r#"
            tell application "System Events"
                try
                    set frontApp to first application process whose frontmost is true
                    set appName to name of frontApp
                    return "success:" & appName
                on error errMsg
                    return "error:" & errMsg
                end try
            end tell
        "#;

        match Command::new("osascript")
            .arg("-e")
            .arg(test_script)
            .output()
        {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                println!("Accessibility test result: {}", result);
                result.starts_with("success:")
            }
            Err(e) => {
                println!("Failed to run accessibility test: {}", e);
                false
            }
        }
    }

    #[cfg(target_os = "macos")]
    pub fn test_permissions(&self) -> String {
        if self.check_accessibility_permissions() {
            "✅ Accessibility permissions are working correctly!".to_string()
        } else {
            "❌ Accessibility permissions needed. Please:\n\
             1. Open System Preferences/Settings\n\
             2. Go to Privacy & Security → Accessibility\n\
             3. Add your terminal/IDE to the list\n\
             4. Restart the application"
                .to_string()
        }
    }

    #[cfg(target_os = "macos")]
    fn get_current_activity_macos(&self) -> Option<CurrentActivity> {
        // Check accessibility permissions first
        if !self.check_accessibility_permissions() {
            println!("⚠️  Accessibility permissions not granted!");
            println!("Please enable accessibility access for this application:");
            println!("1. Open System Preferences/Settings");
            println!("2. Go to Security & Privacy → Privacy → Accessibility");
            println!("3. Add Terminal or VS Code to the list");
            println!("4. Restart the application");

            // Return a fallback activity entry
            return Some(CurrentActivity {
                app_name: "Permission Required".to_string(),
                app_bundle_id: None,
                window_title: "Accessibility permission needed - check console for instructions"
                    .to_string(),
                url: None,
                timestamp: Utc::now(),
                segment_info: None,
            });
        }
        // Use AppleScript to get the frontmost application
        let app_script = r#"
            tell application "System Events"
                set frontApp to first application process whose frontmost is true
                set appName to name of frontApp
                try
                    set bundleID to bundle identifier of frontApp
                on error
                    set bundleID to ""
                end try
                return appName & "|" & bundleID
            end tell
        "#;

        let output = Command::new("osascript")
            .arg("-e")
            .arg(app_script)
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().split('|').collect();

        if parts.is_empty() {
            return None;
        }

        let raw_app_name = parts[0].to_string();
        let bundle_id = if parts.len() > 1 && !parts[1].is_empty() {
            Some(parts[1].to_string())
        } else {
            None
        };

        // Improve app name recognition based on bundle ID
        let app_name = self.get_better_app_name(&raw_app_name, &bundle_id);

        let window_title = self
            .get_active_window_title()
            .unwrap_or_else(|| "Unknown Window".to_string());

        // Enhanced browser detection
        let url = if self.is_browser_app(&app_name, &bundle_id) {
            self.get_browser_url(&app_name, &bundle_id)
        } else {
            None
        };

        // Extract segment information
        let segment_info = self.extract_segment_info(&app_name, &bundle_id, &window_title, &url);

        Some(CurrentActivity {
            app_name,
            app_bundle_id: bundle_id,
            window_title,
            url,
            timestamp: Utc::now(),
            segment_info,
        })
    }

    #[cfg(target_os = "macos")]
    fn get_active_window_title(&self) -> Option<String> {
        // Enhanced AppleScript to get window title with better error handling
        let script = r#"
            tell application "System Events"
                set frontApp to first application process whose frontmost is true
                set appName to name of frontApp
                try
                    -- Try to get the window title
                    if (count of windows of frontApp) > 0 then
                        set windowTitle to name of first window of frontApp
                        if windowTitle is not missing value and windowTitle is not "" then
                            return windowTitle
                        end if
                    end if
                    
                    -- Fallback: try to get document name for some apps
                    try
                        set docName to name of document 1 of frontApp
                        if docName is not missing value and docName is not "" then
                            return docName
                        end if
                    on error
                        -- Ignore document name errors
                    end try
                    
                    -- If no window title, return the app name as fallback
                    return appName & " - No Window Title"
                    
                on error errMsg
                    return "Error: " & errMsg
                end try
            end tell
        "#;

        println!("Getting window title...");

        match Command::new("osascript").arg("-e").arg(script).output() {
            Ok(output) => {
                if output.status.success() {
                    let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("Raw window title: '{}'", title);

                    if !title.is_empty() && title != "missing value" && !title.starts_with("Error:")
                    {
                        println!("Successfully got window title: {}", title);
                        Some(title)
                    } else {
                        println!("Window title was empty or error: {}", title);
                        None
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!(
                        "AppleScript failed with status: {}, stderr: {}",
                        output.status, stderr
                    );
                    None
                }
            }
            Err(e) => {
                println!("Failed to execute AppleScript: {}", e);
                None
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn get_browser_url(&self, app_name: &str, bundle_id: &Option<String>) -> Option<String> {
        let script = if app_name.contains("Chrome")
            || bundle_id
                .as_ref()
                .map_or(false, |id| id.contains("com.google.Chrome"))
        {
            r#"
                tell application "Google Chrome"
                    try
                        if (count of windows) > 0 then
                            set currentTab to active tab of first window
                            set tabURL to URL of currentTab
                            set tabTitle to title of currentTab
                            if tabURL is not missing value and tabURL is not "" then
                                return tabURL
                            else
                                return "Chrome: " & tabTitle
                            end if
                        else
                            return "Chrome: No active tab"
                        end if
                    on error errMsg
                        return "Chrome: Error getting tab info - " & errMsg
                    end try
                end tell
            "#
        } else if app_name.contains("Safari")
            || bundle_id
                .as_ref()
                .map_or(false, |id| id.contains("com.apple.Safari"))
        {
            r#"
                tell application "Safari"
                    try
                        if (count of windows) > 0 then
                            set currentTab to current tab of first window
                            set tabURL to URL of currentTab
                            set tabName to name of currentTab
                            if tabURL is not missing value and tabURL is not "" then
                                return tabURL
                            else
                                return "Safari: " & tabName
                            end if
                        else
                            return "Safari: No active tab"
                        end if
                    on error errMsg
                        return "Safari: Error getting tab info - " & errMsg
                    end try
                end tell
            "#
        } else if app_name.contains("Firefox")
            || bundle_id
                .as_ref()
                .map_or(false, |id| id.contains("org.mozilla.firefox"))
        {
            // Firefox doesn't have good AppleScript support, so we'll use window title
            return Some("Firefox: Check window title for tab info".to_string());
        } else {
            return None;
        };

        println!("Getting browser URL for: {}", app_name);

        match Command::new("osascript").arg("-e").arg(script).output() {
            Ok(output) => {
                if output.status.success() {
                    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    println!("Browser URL result: '{}'", result);

                    if !result.is_empty() && result != "missing value" {
                        Some(result)
                    } else {
                        println!("Browser URL was empty or missing value");
                        None
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    println!(
                        "Browser AppleScript failed with status: {}, stderr: {}",
                        output.status, stderr
                    );
                    None
                }
            }
            Err(e) => {
                println!("Failed to execute browser AppleScript: {}", e);
                None
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn get_better_app_name(&self, raw_name: &str, bundle_id: &Option<String>) -> String {
        // First check bundle IDs for definitive identification
        if let Some(bundle) = bundle_id {
            match bundle.as_str() {
                "com.microsoft.VSCode" => return "Visual Studio Code".to_string(),
                "com.google.Chrome" => return "Google Chrome".to_string(),
                "com.apple.Safari" => return "Safari".to_string(),
                "org.mozilla.firefox" => return "Firefox".to_string(),
                "com.spotify.client" => return "Spotify".to_string(),
                "com.apple.finder" => return "Finder".to_string(),
                "com.apple.Terminal" => return "Terminal".to_string(),
                "com.figma.Desktop" => return "Figma".to_string(),
                "com.notion.id" => return "Notion".to_string(),
                "com.slack.Slack" => return "Slack".to_string(),
                "us.zoom.xos" => return "Zoom".to_string(),
                "com.discord.Discord" => return "Discord".to_string(),
                _ => {}
            }
        }

        // For Electron apps, try to identify from window title
        if raw_name == "Electron" {
            // Get window title to help identify the specific Electron app
            if let Some(window_title) = self.get_active_window_title() {
                println!("Electron app with window title: '{}'", window_title);

                // Check for specific patterns in window titles
                let title_lower = window_title.to_lowercase();

                if title_lower.contains("visual studio code")
                    || title_lower.contains("vs code")
                    || window_title.contains(".ts")
                    || window_title.contains(".js")
                    || window_title.contains(".jsx")
                    || window_title.contains(".tsx")
                    || window_title.contains(".py")
                    || window_title.contains(".rs")
                    || window_title.contains(".json")
                    || window_title.contains("src/")
                    || window_title.contains("velosi-tracker")
                {
                    return "Visual Studio Code".to_string();
                }

                if title_lower.contains("notion") {
                    return "Notion".to_string();
                }

                if title_lower.contains("discord") {
                    return "Discord".to_string();
                }

                if title_lower.contains("slack") {
                    return "Slack".to_string();
                }

                if title_lower.contains("figma") {
                    return "Figma".to_string();
                }

                if title_lower.contains("spotify") {
                    return "Spotify".to_string();
                }

                // If we can't identify it, return a more descriptive name
                return format!("Electron App ({})", window_title);
            } else {
                return "Electron App".to_string();
            }
        }

        // Return the raw name for other apps
        raw_name.to_string()
    }

    #[cfg(target_os = "macos")]
    fn is_browser_app(&self, app_name: &str, bundle_id: &Option<String>) -> bool {
        // Check by app name
        if app_name.contains("Chrome")
            || app_name.contains("Safari")
            || app_name.contains("Firefox")
            || app_name.contains("Edge")
        {
            return true;
        }

        // Check by bundle ID
        if let Some(bundle) = bundle_id {
            matches!(
                bundle.as_str(),
                "com.google.Chrome"
                    | "com.apple.Safari"
                    | "org.mozilla.firefox"
                    | "com.microsoft.edgemac"
            )
        } else {
            false
        }
    }

    #[cfg(target_os = "macos")]
    fn extract_segment_info(
        &self,
        app_name: &str,
        bundle_id: &Option<String>,
        window_title: &str,
        url: &Option<String>,
    ) -> Option<SegmentInfo> {
        // Extract detailed segment information based on app type

        // Browser tabs
        if self.is_browser_app(app_name, bundle_id) {
            if let Some(url_str) = url {
                // Extract domain and page title from URL
                let title = if url_str.starts_with("http") {
                    // Try to extract a meaningful title from the URL
                    if let Ok(url_parsed) = Url::parse(url_str) {
                        url_parsed.domain().unwrap_or("Unknown Site").to_string()
                    } else {
                        url_str.clone()
                    }
                } else {
                    url_str.clone()
                };

                return Some(SegmentInfo {
                    segment_type: SegmentType::BrowserTab,
                    title,
                    url: Some(url_str.clone()),
                    file_path: None,
                    metadata: None,
                });
            }
        }

        // Code editors (VS Code, etc.)
        if app_name.contains("Visual Studio Code")
            || bundle_id
                .as_ref()
                .map_or(false, |id| id.contains("com.microsoft.VSCode"))
        {
            // Extract file path from window title
            // VS Code window titles often look like: "filename.ext - folder - Visual Studio Code"
            if let Some(file_info) = self.extract_vscode_file_info(window_title) {
                return Some(SegmentInfo {
                    segment_type: SegmentType::EditorFile,
                    title: file_info.clone(),
                    url: None,
                    file_path: Some(file_info),
                    metadata: None,
                });
            }
        }

        // Terminal sessions
        if app_name.contains("Terminal") || app_name.contains("iTerm") {
            return Some(SegmentInfo {
                segment_type: SegmentType::TerminalSession,
                title: window_title.to_string(),
                url: None,
                file_path: None,
                metadata: None,
            });
        }

        // Generic app window
        Some(SegmentInfo {
            segment_type: SegmentType::AppWindow,
            title: window_title.to_string(),
            url: None,
            file_path: None,
            metadata: None,
        })
    }

    #[cfg(target_os = "macos")]
    fn extract_vscode_file_info(&self, window_title: &str) -> Option<String> {
        // Extract file path/name from VS Code window title
        // Patterns: "file.ext - folder" or "● file.ext - folder" (● indicates unsaved)

        if window_title.contains(" - ") {
            let parts: Vec<&str> = window_title.split(" - ").collect();
            if !parts.is_empty() {
                let mut file_part = parts[0];
                // Remove unsaved indicator
                if file_part.starts_with('●') {
                    file_part = file_part.trim_start_matches('●').trim();
                }
                return Some(file_part.to_string());
            }
        }

        // Fallback: return the full window title
        Some(window_title.to_string())
    }

    // Windows implementation
    #[cfg(target_os = "windows")]
    fn get_current_activity_windows(&self) -> Option<CurrentActivity> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }

            // Get window title
            let mut window_title = [0u8; 512];
            let title_len = GetWindowTextA(hwnd, window_title.as_mut_ptr() as *mut i8, 512);
            let window_title_str = if title_len > 0 {
                CStr::from_ptr(window_title.as_ptr() as *const i8)
                    .to_string_lossy()
                    .to_string()
            } else {
                "Unknown Window".to_string()
            };

            // Get process ID and executable name
            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, &mut process_id);

            let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION, 0, process_id);
            if process_handle.is_null() {
                return Some(CurrentActivity {
                    app_name: "Unknown Application".to_string(),
                    app_bundle_id: Some(process_id.to_string()),
                    window_title: window_title_str,
                    url: None,
                    timestamp: Utc::now(),
                    segment_info: self.extract_segment_info_windows(
                        &"Unknown Application".to_string(),
                        &window_title_str,
                    ),
                });
            }

            let mut exe_path = [0u8; 512];
            let path_len = GetModuleFileNameExA(
                process_handle,
                ptr::null_mut(),
                exe_path.as_mut_ptr() as *mut i8,
                512,
            );
            CloseHandle(process_handle);

            let app_name = if path_len > 0 {
                let full_path = CStr::from_ptr(exe_path.as_ptr() as *const i8)
                    .to_string_lossy()
                    .to_string();

                // Extract just the executable name from the full path
                std::path::Path::new(&full_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            } else {
                "Unknown Application".to_string()
            };

            // Improve app name recognition
            let better_app_name = self.get_better_app_name_windows(&app_name);

            // Check if it's a browser and get URL if possible
            let url = if self.is_browser_app_windows(&better_app_name) {
                self.get_browser_url_windows(&better_app_name, &window_title_str)
            } else {
                None
            };

            // Extract segment information
            let segment_info =
                self.extract_segment_info_windows(&better_app_name, &window_title_str);

            Some(CurrentActivity {
                app_name: better_app_name,
                app_bundle_id: Some(process_id.to_string()),
                window_title: window_title_str,
                url,
                timestamp: Utc::now(),
                segment_info,
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn get_better_app_name_windows(&self, raw_name: &str) -> String {
        match raw_name.to_lowercase().as_str() {
            "chrome" => "Google Chrome".to_string(),
            "firefox" => "Mozilla Firefox".to_string(),
            "msedge" => "Microsoft Edge".to_string(),
            "code" => "Visual Studio Code".to_string(),
            "notepad++" => "Notepad++".to_string(),
            "explorer" => "File Explorer".to_string(),
            "cmd" => "Command Prompt".to_string(),
            "powershell" => "PowerShell".to_string(),
            "discord" => "Discord".to_string(),
            "slack" => "Slack".to_string(),
            "teams" => "Microsoft Teams".to_string(),
            "zoom" => "Zoom".to_string(),
            "spotify" => "Spotify".to_string(),
            "vlc" => "VLC Media Player".to_string(),
            _ => raw_name.to_string(),
        }
    }

    #[cfg(target_os = "windows")]
    fn is_browser_app_windows(&self, app_name: &str) -> bool {
        let app_lower = app_name.to_lowercase();
        app_lower.contains("chrome")
            || app_lower.contains("firefox")
            || app_lower.contains("edge")
            || app_lower.contains("safari")
            || app_lower.contains("opera")
            || app_lower.contains("brave")
    }

    #[cfg(target_os = "windows")]
    fn get_browser_url_windows(&self, app_name: &str, window_title: &str) -> Option<String> {
        // For Windows, we'll extract URL from window title if possible
        // This is a simplified approach - more sophisticated URL extraction would require browser-specific APIs

        if window_title.contains(" - ") {
            let parts: Vec<&str> = window_title.split(" - ").collect();
            if parts.len() >= 2 {
                let potential_url = parts[parts.len() - 2]; // Usually the URL is before the browser name
                if potential_url.starts_with("http") {
                    return Some(potential_url.to_string());
                }
            }
        }

        // Try to extract from common browser title patterns
        if app_name.to_lowercase().contains("chrome") && window_title.contains("Google Chrome") {
            // Chrome usually has format: "Page Title - Google Chrome"
            if let Some(title_part) = window_title.strip_suffix(" - Google Chrome") {
                return Some(format!("Chrome Tab: {}", title_part));
            }
        }

        if app_name.to_lowercase().contains("firefox") && window_title.contains("Mozilla Firefox") {
            if let Some(title_part) = window_title.strip_suffix(" - Mozilla Firefox") {
                return Some(format!("Firefox Tab: {}", title_part));
            }
        }

        if app_name.to_lowercase().contains("edge") && window_title.contains("Microsoft Edge") {
            if let Some(title_part) = window_title.strip_suffix(" - Microsoft Edge") {
                return Some(format!("Edge Tab: {}", title_part));
            }
        }

        Some(format!("{}: {}", app_name, window_title))
    }

    #[cfg(target_os = "windows")]
    fn extract_segment_info_windows(
        &self,
        app_name: &str,
        window_title: &str,
    ) -> Option<SegmentInfo> {
        let app_lower = app_name.to_lowercase();

        if self.is_browser_app_windows(app_name) {
            Some(SegmentInfo {
                segment_type: SegmentType::BrowserTab,
                title: window_title.to_string(),
                url: self.get_browser_url_windows(app_name, window_title),
                file_path: None,
                metadata: Some(format!("Browser: {}", app_name)),
            })
        } else if app_lower.contains("code")
            || app_lower.contains("notepad")
            || window_title.contains(".")
        {
            // Likely a code editor or file editor
            Some(SegmentInfo {
                segment_type: SegmentType::EditorFile,
                title: window_title.to_string(),
                url: None,
                file_path: self.extract_file_path_from_title_windows(window_title),
                metadata: Some(format!("Editor: {}", app_name)),
            })
        } else {
            Some(SegmentInfo {
                segment_type: SegmentType::AppWindow,
                title: window_title.to_string(),
                url: None,
                file_path: None,
                metadata: Some(format!("Application: {}", app_name)),
            })
        }
    }

    #[cfg(target_os = "windows")]
    fn extract_file_path_from_title_windows(&self, window_title: &str) -> Option<String> {
        // Try to extract file path from various editor title patterns

        // VS Code pattern: "filename.ext - path - Visual Studio Code"
        if window_title.contains(" - Visual Studio Code") {
            if let Some(title_part) = window_title.strip_suffix(" - Visual Studio Code") {
                if title_part.contains(" - ") {
                    let parts: Vec<&str> = title_part.split(" - ").collect();
                    if parts.len() >= 2 {
                        return Some(format!("{}/{}", parts[1], parts[0]));
                    }
                }
                return Some(title_part.to_string());
            }
        }

        // Notepad++ pattern: "filename.ext - Notepad++"
        if window_title.contains(" - Notepad++") {
            if let Some(filename) = window_title.strip_suffix(" - Notepad++") {
                return Some(filename.to_string());
            }
        }

        // Generic file pattern: look for file extensions
        if window_title.contains('.') {
            let potential_file = window_title.split(' ').find(|part| {
                part.contains('.')
                    && part.len() > 3
                    && (part.ends_with(".txt")
                        || part.ends_with(".rs")
                        || part.ends_with(".js")
                        || part.ends_with(".ts")
                        || part.ends_with(".py")
                        || part.ends_with(".cpp")
                        || part.ends_with(".c")
                        || part.ends_with(".h")
                        || part.ends_with(".java"))
            });

            if let Some(file) = potential_file {
                return Some(file.to_string());
            }
        }

        None
    }

    // Fallback implementations for non-macOS, non-Windows platforms
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_active_window_title(&self) -> Option<String> {
        None
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_browser_url(&self, _app_name: &str, _bundle_id: &Option<String>) -> Option<String> {
        None
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn get_better_app_name(&self, raw_name: &str, _bundle_id: &Option<String>) -> String {
        raw_name.to_string()
    }

    #[cfg(not(target_os = "macos"))]
    fn is_browser_app(&self, _app_name: &str, _bundle_id: &Option<String>) -> bool {
        false
    }

    #[cfg(not(target_os = "macos"))]
    fn extract_segment_info(
        &self,
        _app_name: &str,
        _bundle_id: &Option<String>,
        window_title: &str,
        _url: &Option<String>,
    ) -> Option<SegmentInfo> {
        Some(SegmentInfo {
            segment_type: SegmentType::AppWindow,
            title: window_title.to_string(),
            url: None,
            file_path: None,
            metadata: None,
        })
    }

    pub fn check_user_activity(&mut self) -> bool {
        // For now, we'll be more permissive and assume user is active if we're checking
        // In a real implementation, you'd want to check for actual user input
        self.user_activity.last_input_time = SystemTime::now();
        self.user_activity.is_active = true;

        // Only return false if we haven't updated for the threshold time
        if let Ok(elapsed) = self.user_activity.last_input_time.elapsed() {
            if elapsed > self.inactive_threshold {
                self.user_activity.is_active = false;
                return false;
            }
        }
        true
    }

    pub fn update_last_input_time(&mut self) {
        self.user_activity.last_input_time = SystemTime::now();
        self.user_activity.is_active = true;
    }

    pub fn should_track(&mut self) -> bool {
        let result = self.check_user_activity();
        println!("Should track result: {}", result);
        result
    }
}
