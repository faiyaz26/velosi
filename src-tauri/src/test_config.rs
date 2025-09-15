// Test configuration and utilities for Velosi Tracker tests

#[cfg(test)]
pub mod test_utils {
    use crate::database::Database;
    use crate::models::*;
    use crate::AppState;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tauri::test::{mock_app, MockRuntime};
    use tauri::Manager;
    use uuid::Uuid;

    /// Creates a test database with in-memory SQLite
    pub async fn create_test_database() -> Database {
        Database::new("sqlite::memory:")
            .await
            .expect("Failed to create test database")
    }

    /// Creates a test app state with the given database
    pub fn create_test_app_state(db: Database) -> AppState {
        AppState {
            db: Arc::new(db),
            tracker: Arc::new(Mutex::new(crate::tracker::ActivityTracker::new())),
            is_tracking: Arc::new(Mutex::new(true)),
            pause_until: Arc::new(Mutex::new(None)),
            current_activity: Arc::new(Mutex::new(None)),
            focus_mode_enabled: Arc::new(Mutex::new(false)),
            focus_mode_allowed_categories: Arc::new(Mutex::new(Vec::new())),
            recently_blocked_apps: Arc::new(Mutex::new(HashMap::new())),
            app_category_cache: Arc::new(Mutex::new(HashMap::new())),
            focus_mode_allowed_apps_cache: Arc::new(Mutex::new(HashMap::new())),
            app_mappings_cache: Arc::new(Mutex::new(None)),
            recently_hidden_apps: Arc::new(Mutex::new(HashMap::new())),
            website_blocker: Arc::new(Mutex::new(None)),
        }
    }

    /// Creates a mock Tauri app with test state
    pub async fn create_mock_app_with_state() -> (tauri::App<MockRuntime>, AppState) {
        let db = create_test_database().await;
        let state = create_test_app_state(db);
        let app = mock_app();
        app.manage(state.clone());
        (app, state)
    }

    /// Creates a sample user category for testing
    pub fn create_sample_category(name: &str, color: &str) -> UserCategory {
        UserCategory {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            color: color.to_string(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Creates a sample activity entry for testing
    pub fn create_sample_activity(app_name: &str, window_title: &str) -> ActivityEntry {
        ActivityEntry {
            id: Uuid::new_v4(),
            start_time: Utc::now(),
            end_time: None,
            app_name: app_name.to_string(),
            app_bundle_id: Some(format!(
                "com.test.{}",
                app_name.to_lowercase().replace(" ", "")
            )),
            window_title: window_title.to_string(),
            url: None,
            category: ActivityCategory::Unknown,
            segments: vec![],
        }
    }

    /// Creates a sample app mapping for testing
    pub fn create_sample_app_mapping(category_id: &str, app_pattern: &str) -> AppMapping {
        AppMapping {
            id: Uuid::new_v4(),
            app_pattern: app_pattern.to_string(),
            category_id: category_id.to_string(),
            is_custom: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Creates a sample URL mapping for testing
    pub fn create_sample_url_mapping(category_id: &str, url_pattern: &str) -> UrlMapping {
        UrlMapping {
            id: Uuid::new_v4(),
            url_pattern: url_pattern.to_string(),
            category_id: category_id.to_string(),
            is_custom: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Test data constants
    pub mod test_data {
        pub const SAMPLE_CATEGORIES: &[(&str, &str)] = &[
            ("Work", "#FF0000"),
            ("Development", "#00FF00"),
            ("Entertainment", "#0000FF"),
            ("Communication", "#FFFF00"),
            ("Social", "#FF00FF"),
        ];

        pub const SAMPLE_APPS: &[(&str, &str)] = &[
            ("Visual Studio Code", "Code Editor"),
            ("Google Chrome", "Web Browser"),
            ("Slack", "Communication"),
            ("Spotify", "Music Player"),
            ("Terminal", "Command Line"),
        ];

        pub const SAMPLE_URLS: &[&str] = &[
            "github.com",
            "stackoverflow.com",
            "google.com",
            "youtube.com",
            "twitter.com",
        ];
    }

    /// Assertion helpers for tests
    pub mod assertions {
        use crate::models::*;

        /// Assert that two activities are equivalent (ignoring timestamps)
        pub fn assert_activities_equivalent(actual: &ActivityEntry, expected: &ActivityEntry) {
            assert_eq!(actual.app_name, expected.app_name);
            assert_eq!(actual.app_bundle_id, expected.app_bundle_id);
            assert_eq!(actual.window_title, expected.window_title);
            assert_eq!(actual.url, expected.url);
            assert_eq!(actual.category, expected.category);
        }

        /// Assert that a category has expected properties
        pub fn assert_category_properties(category: &UserCategory, name: &str, color: &str) {
            assert_eq!(category.name, name);
            assert_eq!(category.color, color);
            assert!(!category.id.is_empty());
        }

        /// Assert that an app mapping has expected properties
        pub fn assert_app_mapping_properties(
            mapping: &AppMapping,
            app_pattern: &str,
            category_id: &str,
        ) {
            assert_eq!(mapping.app_pattern, app_pattern);
            assert_eq!(mapping.category_id, category_id);
        }

        /// Assert that a URL mapping has expected properties
        pub fn assert_url_mapping_properties(
            mapping: &UrlMapping,
            url_pattern: &str,
            category_id: &str,
        ) {
            assert_eq!(mapping.url_pattern, url_pattern);
            assert_eq!(mapping.category_id, category_id);
        }
    }

    /// Performance testing utilities
    pub mod performance {
        use std::time::Instant;

        /// Measure execution time of a function
        pub async fn measure_async<F, Fut, T>(f: F) -> (T, std::time::Duration)
        where
            F: FnOnce() -> Fut,
            Fut: std::future::Future<Output = T>,
        {
            let start = Instant::now();
            let result = f().await;
            let duration = start.elapsed();
            (result, duration)
        }

        /// Assert that an operation completes within a time limit
        pub fn assert_performance<T>(
            result: (T, std::time::Duration),
            max_duration: std::time::Duration,
            operation_name: &str,
        ) -> T {
            let (value, duration) = result;
            assert!(
                duration <= max_duration,
                "{} took {:?}, expected <= {:?}",
                operation_name,
                duration,
                max_duration
            );
            value
        }
    }

    /// Database seeding utilities for integration tests
    pub mod seeding {
        use super::*;
        use crate::database::Database;

        /// Seed the database with sample categories
        pub async fn seed_categories(db: &Database) -> Vec<UserCategory> {
            let mut categories = Vec::new();

            for (name, color) in test_data::SAMPLE_CATEGORIES {
                let category = create_sample_category(name, color);
                db.add_user_category(&category).await.unwrap();
                categories.push(category);
            }

            categories
        }

        /// Seed the database with sample app mappings
        pub async fn seed_app_mappings(
            db: &Database,
            categories: &[UserCategory],
        ) -> Vec<AppMapping> {
            let mut mappings = Vec::new();

            for (i, (app_name, _)) in test_data::SAMPLE_APPS.iter().enumerate() {
                if let Some(category) = categories.get(i % categories.len()) {
                    let mapping = create_sample_app_mapping(&category.id, app_name);
                    db.add_simple_app_mapping(&category.id, app_name, true)
                        .await
                        .unwrap();
                    mappings.push(mapping);
                }
            }

            mappings
        }

        /// Seed the database with sample URL mappings
        pub async fn seed_url_mappings(
            db: &Database,
            categories: &[UserCategory],
        ) -> Vec<UrlMapping> {
            let mut mappings = Vec::new();

            for (i, url) in test_data::SAMPLE_URLS.iter().enumerate() {
                if let Some(category) = categories.get(i % categories.len()) {
                    let mapping = create_sample_url_mapping(&category.id, url);
                    db.add_url_mapping(&mapping).await.unwrap();
                    mappings.push(mapping);
                }
            }

            mappings
        }

        /// Seed the database with sample activities
        pub async fn seed_activities(db: &Database) -> Vec<ActivityEntry> {
            let mut activities = Vec::new();

            for (app_name, window_title) in test_data::SAMPLE_APPS {
                let activity = create_sample_activity(app_name, window_title);
                db.start_activity(&activity).await.unwrap();
                activities.push(activity);
            }

            activities
        }

        /// Seed the database with complete test data
        pub async fn seed_complete_test_data(db: &Database) -> TestDataSet {
            let categories = seed_categories(db).await;
            let app_mappings = seed_app_mappings(db, &categories).await;
            let url_mappings = seed_url_mappings(db, &categories).await;
            let activities = seed_activities(db).await;

            TestDataSet {
                categories,
                app_mappings,
                url_mappings,
                activities,
            }
        }
    }

    /// Complete test data set
    pub struct TestDataSet {
        pub categories: Vec<UserCategory>,
        pub app_mappings: Vec<AppMapping>,
        pub url_mappings: Vec<UrlMapping>,
        pub activities: Vec<ActivityEntry>,
    }
}
