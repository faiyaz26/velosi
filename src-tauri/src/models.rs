use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>, // NULL means activity is still ongoing
    pub app_name: String,
    pub app_bundle_id: Option<String>,
    pub window_title: String,
    pub url: Option<String>, // For browsers
    pub category: ActivityCategory,
    pub segments: Vec<ActivitySegment>, // New: granular activity segments
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySegment {
    pub id: Uuid,
    pub activity_id: Uuid, // Reference to parent activity
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub segment_type: SegmentType,
    pub title: String,             // Tab title, file name, etc.
    pub url: Option<String>,       // URL for browser tabs
    pub file_path: Option<String>, // File path for editors
    pub metadata: Option<String>,  // JSON string for additional data
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SegmentType {
    BrowserTab,      // Individual browser tab
    EditorFile,      // File in code editor
    DocumentFile,    // Document in word processor
    TerminalSession, // Terminal session/directory
    AppWindow,       // Generic app window
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActivityCategory {
    Productive,
    Social,
    Entertainment,
    Development,
    Communication,
    Custom(String), // For user-defined categories
    Unknown,
}

// User-defined categories stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCategory {
    pub id: String,
    pub name: String,
    pub color: String,
    pub parent_id: Option<String>, // For subcategories
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// App to category mappings stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMapping {
    pub id: Uuid,
    pub app_pattern: String, // App name pattern (can include wildcards)
    pub category_id: String, // References either built-in or user category
    pub is_custom: bool,     // true if user override, false if default
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlMapping {
    pub id: Uuid,
    pub url_pattern: String, // URL pattern (domain, subdomain, or full URL)
    pub category_id: String, // References either built-in or user category
    pub is_custom: bool,     // true if user override, false if default
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub date: String,
    pub total_active_time: i64,
    pub categories: Vec<CategorySummary>,
    pub top_apps: Vec<AppSummary>,
    pub detailed_activities: Vec<DetailedActivity>, // New: includes segments
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineData {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub activities: Vec<TimelineActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineActivity {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: i64,
    pub app_name: String,
    pub app_bundle_id: Option<String>,
    pub window_title: String,
    pub url: Option<String>,
    pub category: ActivityCategory,
    pub segments: Vec<TimelineSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineSegment {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: i64,
    pub segment_type: SegmentType,
    pub title: String,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedActivity {
    pub app_name: String,
    pub duration_seconds: i64,
    pub segments: Vec<SegmentSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentSummary {
    pub segment_type: SegmentType,
    pub title: String,
    pub duration_seconds: i64,
    pub url: Option<String>,
    pub file_path: Option<String>,
    pub times_accessed: i32, // How many times this segment was accessed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySummary {
    pub category: ActivityCategory,
    pub duration_seconds: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSummary {
    pub app_name: String,
    pub duration_seconds: i64,
    pub percentage: f64,
}

impl ActivityCategory {
    #[allow(dead_code)]
    pub fn from_app_name(app_name: &str, _bundle_id: Option<&str>) -> Self {
        Self::from_app_name_and_url(app_name, _bundle_id, None)
    }

    #[allow(dead_code)]
    pub fn from_app_name_and_url(
        app_name: &str,
        _bundle_id: Option<&str>,
        url: Option<&str>,
    ) -> Self {
        // First, try URL-based categorization if URL is available
        if let Some(url_str) = url {
            if let Some(category) = Self::from_url(url_str) {
                return category;
            }
        }

        // Fall back to app-based categorization
        match app_name.to_lowercase().as_str() {
            name if name.contains("xcode")
                || name.contains("vscode")
                || name.contains("intellij")
                || name.contains("terminal")
                || name.contains("iterm") =>
            {
                Self::Development
            }
            name if name.contains("chrome")
                || name.contains("safari")
                || name.contains("firefox")
                || name.contains("edge") =>
            {
                // For browsers, if no URL-specific category was found, default to Productive
                Self::Productive
            }
            name if name.contains("slack")
                || name.contains("discord")
                || name.contains("zoom")
                || name.contains("teams")
                || name.contains("mail") =>
            {
                Self::Communication
            }
            name if name.contains("twitter")
                || name.contains("facebook")
                || name.contains("instagram")
                || name.contains("linkedin") =>
            {
                Self::Social
            }
            name if name.contains("youtube")
                || name.contains("netflix")
                || name.contains("spotify")
                || name.contains("music")
                || name.contains("vlc") =>
            {
                Self::Entertainment
            }
            _ => Self::Unknown,
        }
    }

    #[allow(dead_code)]
    fn from_url(url: &str) -> Option<Self> {
        let url_lower = url.to_lowercase();

        // Check for common URL patterns and categorize accordingly
        if url_lower.contains("github.com")
            || url_lower.contains("gitlab.com")
            || url_lower.contains("stackoverflow.com")
            || url_lower.contains("docs.")
            || url_lower.contains("developer.")
        {
            Some(Self::Development)
        } else if url_lower.contains("gmail.com")
            || url_lower.contains("outlook.com")
            || url_lower.contains("mail.")
            || url_lower.contains("slack.com")
            || url_lower.contains("discord.com")
            || url_lower.contains("teams.microsoft.com")
        {
            Some(Self::Communication)
        } else if url_lower.contains("twitter.com")
            || url_lower.contains("facebook.com")
            || url_lower.contains("instagram.com")
            || url_lower.contains("linkedin.com")
            || url_lower.contains("reddit.com")
            || url_lower.contains("tiktok.com")
        {
            Some(Self::Social)
        } else if url_lower.contains("youtube.com")
            || url_lower.contains("netflix.com")
            || url_lower.contains("spotify.com")
            || url_lower.contains("twitch.tv")
            || url_lower.contains("hulu.com")
            || url_lower.contains("disney.com")
        {
            Some(Self::Entertainment)
        } else if url_lower.contains("amazon.com")
            || url_lower.contains("ebay.com")
            || url_lower.contains("shopping")
            || url_lower.contains("store")
        {
            Some(Self::Productive) // Shopping can be considered productive in some contexts
        } else {
            None // No specific category found, will fall back to app-based categorization
        }
    }
}
