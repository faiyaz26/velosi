#[cfg(test)]
mod tracker_tests {
    use crate::models::*;
    use crate::tracker::*;
    use chrono::Utc;

    #[test]
    fn test_current_activity_creation() {
        let activity = CurrentActivity {
            app_name: "Test App".to_string(),
            app_bundle_id: Some("com.test.app".to_string()),
            window_title: "Test Window".to_string(),
            url: Some("https://example.com".to_string()),
            timestamp: Utc::now(),
            segment_info: None,
        };

        assert_eq!(activity.app_name, "Test App");
        assert_eq!(activity.app_bundle_id, Some("com.test.app".to_string()));
        assert_eq!(activity.window_title, "Test Window");
        assert_eq!(activity.url, Some("https://example.com".to_string()));
        assert!(activity.segment_info.is_none());
    }

    #[test]
    fn test_current_activity_with_segment_info() {
        let segment_info = SegmentInfo {
            segment_type: SegmentType::BrowserTab,
            title: "GitHub - Repository".to_string(),
            url: Some("https://github.com/user/repo".to_string()),
            file_path: None,
            metadata: Some(r#"{"tab_id": 123}"#.to_string()),
        };

        let activity = CurrentActivity {
            app_name: "Google Chrome".to_string(),
            app_bundle_id: Some("com.google.Chrome".to_string()),
            window_title: "GitHub - Repository - Google Chrome".to_string(),
            url: Some("https://github.com/user/repo".to_string()),
            timestamp: Utc::now(),
            segment_info: Some(segment_info.clone()),
        };

        assert!(activity.segment_info.is_some());
        let segment = activity.segment_info.unwrap();
        assert_eq!(segment.segment_type, SegmentType::BrowserTab);
        assert_eq!(segment.title, "GitHub - Repository");
        assert_eq!(
            segment.url,
            Some("https://github.com/user/repo".to_string())
        );
        assert!(segment.file_path.is_none());
        assert!(segment.metadata.is_some());
    }

    #[test]
    fn test_segment_types() {
        let browser_segment = SegmentInfo {
            segment_type: SegmentType::BrowserTab,
            title: "Example Page".to_string(),
            url: Some("https://example.com".to_string()),
            file_path: None,
            metadata: None,
        };

        let editor_segment = SegmentInfo {
            segment_type: SegmentType::EditorFile,
            title: "main.rs".to_string(),
            url: None,
            file_path: Some("/path/to/project/src/main.rs".to_string()),
            metadata: Some(r#"{"language": "rust", "line": 42}"#.to_string()),
        };

        let terminal_segment = SegmentInfo {
            segment_type: SegmentType::TerminalSession,
            title: "/home/user/project".to_string(),
            url: None,
            file_path: Some("/home/user/project".to_string()),
            metadata: Some(r#"{"shell": "zsh", "command": "cargo test"}"#.to_string()),
        };

        let document_segment = SegmentInfo {
            segment_type: SegmentType::DocumentFile,
            title: "Report.docx".to_string(),
            url: None,
            file_path: Some("/path/to/Report.docx".to_string()),
            metadata: None,
        };

        let app_window_segment = SegmentInfo {
            segment_type: SegmentType::AppWindow,
            title: "Settings".to_string(),
            url: None,
            file_path: None,
            metadata: Some(r#"{"window_type": "preferences"}"#.to_string()),
        };

        let unknown_segment = SegmentInfo {
            segment_type: SegmentType::Unknown,
            title: "Unknown Window".to_string(),
            url: None,
            file_path: None,
            metadata: None,
        };

        // Test that all segment types can be created and have expected properties
        assert_eq!(browser_segment.segment_type, SegmentType::BrowserTab);
        assert!(browser_segment.url.is_some());
        assert!(browser_segment.file_path.is_none());

        assert_eq!(editor_segment.segment_type, SegmentType::EditorFile);
        assert!(editor_segment.file_path.is_some());
        assert!(editor_segment.url.is_none());

        assert_eq!(terminal_segment.segment_type, SegmentType::TerminalSession);
        assert!(terminal_segment.file_path.is_some());

        assert_eq!(document_segment.segment_type, SegmentType::DocumentFile);
        assert!(document_segment.file_path.is_some());

        assert_eq!(app_window_segment.segment_type, SegmentType::AppWindow);
        assert!(app_window_segment.metadata.is_some());

        assert_eq!(unknown_segment.segment_type, SegmentType::Unknown);
    }

    #[test]
    fn test_user_activity_creation() {
        use std::time::SystemTime;

        let user_activity = UserActivity {
            last_input_time: SystemTime::now(),
            is_active: true,
        };

        assert!(user_activity.is_active);
        // Can't easily test the exact time, but we can verify it was set
        assert!(user_activity.last_input_time <= SystemTime::now());
    }

    #[test]
    fn test_activity_tracker_creation() {
        let _tracker = ActivityTracker::new();
        // Test that tracker can be created without panicking
        // The actual implementation details depend on the ActivityTracker struct
        // which wasn't fully visible in the truncated file
    }

    #[test]
    fn test_segment_info_serialization() {
        let segment_info = SegmentInfo {
            segment_type: SegmentType::BrowserTab,
            title: "Test Page".to_string(),
            url: Some("https://test.com".to_string()),
            file_path: None,
            metadata: Some(r#"{"test": true}"#.to_string()),
        };

        // Test that segment info can be serialized and deserialized
        let serialized = serde_json::to_string(&segment_info).unwrap();
        let deserialized: SegmentInfo = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.segment_type, SegmentType::BrowserTab);
        assert_eq!(deserialized.title, "Test Page");
        assert_eq!(deserialized.url, Some("https://test.com".to_string()));
        assert!(deserialized.file_path.is_none());
        assert_eq!(deserialized.metadata, Some(r#"{"test": true}"#.to_string()));
    }

    #[test]
    fn test_current_activity_serialization() {
        let activity = CurrentActivity {
            app_name: "Test App".to_string(),
            app_bundle_id: Some("com.test.app".to_string()),
            window_title: "Test Window".to_string(),
            url: Some("https://example.com".to_string()),
            timestamp: Utc::now(),
            segment_info: Some(SegmentInfo {
                segment_type: SegmentType::BrowserTab,
                title: "Test Tab".to_string(),
                url: Some("https://example.com".to_string()),
                file_path: None,
                metadata: None,
            }),
        };

        // Test that current activity can be serialized and deserialized
        let serialized = serde_json::to_string(&activity).unwrap();
        let deserialized: CurrentActivity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.app_name, "Test App");
        assert_eq!(deserialized.app_bundle_id, Some("com.test.app".to_string()));
        assert_eq!(deserialized.window_title, "Test Window");
        assert_eq!(deserialized.url, Some("https://example.com".to_string()));
        assert!(deserialized.segment_info.is_some());

        let segment = deserialized.segment_info.unwrap();
        assert_eq!(segment.segment_type, SegmentType::BrowserTab);
        assert_eq!(segment.title, "Test Tab");
    }

    #[test]
    fn test_segment_type_equality() {
        assert_eq!(SegmentType::BrowserTab, SegmentType::BrowserTab);
        assert_eq!(SegmentType::EditorFile, SegmentType::EditorFile);
        assert_eq!(SegmentType::DocumentFile, SegmentType::DocumentFile);
        assert_eq!(SegmentType::TerminalSession, SegmentType::TerminalSession);
        assert_eq!(SegmentType::AppWindow, SegmentType::AppWindow);
        assert_eq!(SegmentType::Unknown, SegmentType::Unknown);

        assert_ne!(SegmentType::BrowserTab, SegmentType::EditorFile);
        assert_ne!(SegmentType::EditorFile, SegmentType::Unknown);
    }

    #[test]
    fn test_current_activity_clone() {
        let original = CurrentActivity {
            app_name: "Test App".to_string(),
            app_bundle_id: Some("com.test.app".to_string()),
            window_title: "Test Window".to_string(),
            url: Some("https://example.com".to_string()),
            timestamp: Utc::now(),
            segment_info: None,
        };

        let cloned = original.clone();

        assert_eq!(original.app_name, cloned.app_name);
        assert_eq!(original.app_bundle_id, cloned.app_bundle_id);
        assert_eq!(original.window_title, cloned.window_title);
        assert_eq!(original.url, cloned.url);
        assert_eq!(original.timestamp, cloned.timestamp);
        assert_eq!(
            original.segment_info.is_none(),
            cloned.segment_info.is_none()
        );
    }

    #[test]
    fn test_segment_info_with_different_metadata() {
        let json_metadata = r#"{"type": "json", "valid": true}"#;
        let simple_metadata = "simple string metadata";
        let empty_metadata = "";

        let segment1 = SegmentInfo {
            segment_type: SegmentType::EditorFile,
            title: "test.json".to_string(),
            url: None,
            file_path: Some("/path/to/test.json".to_string()),
            metadata: Some(json_metadata.to_string()),
        };

        let segment2 = SegmentInfo {
            segment_type: SegmentType::AppWindow,
            title: "App Window".to_string(),
            url: None,
            file_path: None,
            metadata: Some(simple_metadata.to_string()),
        };

        let segment3 = SegmentInfo {
            segment_type: SegmentType::Unknown,
            title: "Unknown".to_string(),
            url: None,
            file_path: None,
            metadata: Some(empty_metadata.to_string()),
        };

        let segment4 = SegmentInfo {
            segment_type: SegmentType::BrowserTab,
            title: "No Metadata".to_string(),
            url: Some("https://example.com".to_string()),
            file_path: None,
            metadata: None,
        };

        assert_eq!(segment1.metadata, Some(json_metadata.to_string()));
        assert_eq!(segment2.metadata, Some(simple_metadata.to_string()));
        assert_eq!(segment3.metadata, Some(empty_metadata.to_string()));
        assert!(segment4.metadata.is_none());
    }

    #[test]
    fn test_current_activity_with_minimal_data() {
        let minimal_activity = CurrentActivity {
            app_name: "App".to_string(),
            app_bundle_id: None,
            window_title: "".to_string(),
            url: None,
            timestamp: Utc::now(),
            segment_info: None,
        };

        assert_eq!(minimal_activity.app_name, "App");
        assert!(minimal_activity.app_bundle_id.is_none());
        assert_eq!(minimal_activity.window_title, "");
        assert!(minimal_activity.url.is_none());
        assert!(minimal_activity.segment_info.is_none());
    }

    #[test]
    fn test_current_activity_with_maximal_data() {
        let maximal_activity = CurrentActivity {
            app_name: "Complex Application Name".to_string(),
            app_bundle_id: Some("com.company.complex.application.name".to_string()),
            window_title: "Very Long Window Title With Special Characters !@#$%^&*()".to_string(),
            url: Some("https://very-long-domain-name.example.com/path/to/resource?param1=value1&param2=value2#fragment".to_string()),
            timestamp: Utc::now(),
            segment_info: Some(SegmentInfo {
                segment_type: SegmentType::BrowserTab,
                title: "Complex Tab Title With Unicode 🚀 Characters".to_string(),
                url: Some("https://unicode-test.example.com/🚀/test".to_string()),
                file_path: None,
                metadata: Some(r#"{"complex": {"nested": {"data": [1, 2, 3]}, "unicode": "🚀"}, "timestamp": 1234567890}"#.to_string()),
            }),
        };

        // Test that complex data is handled correctly
        assert!(maximal_activity.app_name.len() > 10);
        assert!(maximal_activity.app_bundle_id.is_some());
        assert!(maximal_activity.window_title.contains("Special Characters"));
        assert!(maximal_activity.url.is_some());
        assert!(maximal_activity.segment_info.is_some());

        let segment = maximal_activity.segment_info.unwrap();
        assert!(segment.title.contains("🚀"));
        assert!(segment.url.unwrap().contains("🚀"));
        assert!(segment.metadata.unwrap().contains("unicode"));
    }
}
