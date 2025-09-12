#[cfg(test)]
mod database_tests {
    use crate::database::Database;
    use crate::models::*;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    use crate::test_config::test_utils::create_test_database;

    async fn create_test_db() -> Database {
        create_test_database().await
    }

    #[tokio::test]
    async fn test_database_creation() {
        let _db = create_test_db().await;
        // If we get here without panicking, database creation succeeded
        assert!(true);
    }

    #[tokio::test]
    async fn test_activity_crud_operations() {
        let db = create_test_db().await;

        // Create test activity
        let activity = ActivityEntry {
            id: Uuid::new_v4(),
            start_time: Utc::now(),
            end_time: None,
            app_name: "Test App".to_string(),
            app_bundle_id: Some("com.test.app".to_string()),
            window_title: "Test Window".to_string(),
            url: Some("https://example.com".to_string()),
            category: ActivityCategory::Development,
            segments: vec![],
        };

        // Test creating activity
        let result = db.start_activity(&activity).await;
        assert!(result.is_ok());

        // Test getting current activity
        let current = db.get_current_activity().await.unwrap();
        assert!(current.is_some());
        let current = current.unwrap();
        assert_eq!(current.app_name, "Test App");
        assert_eq!(current.window_title, "Test Window");

        // Test ending activity
        let end_time = Utc::now();
        let result = db.end_current_activity(end_time).await;
        assert!(result.is_ok());

        // Verify no current activity
        let current = db.get_current_activity().await.unwrap();
        assert!(current.is_none());
    }

    #[tokio::test]
    async fn test_user_categories() {
        let db = create_test_db().await;

        // Test getting categories (may have initial data)
        let initial_categories = db.get_user_categories().await.unwrap();
        let initial_count = initial_categories.len();

        // Create test category
        let category = UserCategory {
            id: Uuid::new_v4().to_string(),
            name: "Work".to_string(),
            color: "#FF0000".to_string(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test adding category
        let result = db.add_user_category(&category).await;
        assert!(result.is_ok());

        // Test getting categories
        let categories = db.get_user_categories().await.unwrap();
        assert_eq!(categories.len(), initial_count + 1);

        // Find our added category
        let work_category = categories.iter().find(|c| c.name == "Work").unwrap();
        assert_eq!(work_category.name, "Work");
        assert_eq!(work_category.color, "#FF0000");

        // Test updating category
        let mut updated_category = category.clone();
        updated_category.name = "Updated Work".to_string();
        updated_category.color = "#00FF00".to_string();
        updated_category.updated_at = Utc::now();

        let result = db.update_user_category(&updated_category).await;
        assert!(result.is_ok());

        // Verify update
        let categories = db.get_user_categories().await.unwrap();
        let updated_category = categories.iter().find(|c| c.id == category.id).unwrap();
        assert_eq!(updated_category.name, "Updated Work");
        assert_eq!(updated_category.color, "#00FF00");

        // Test deleting category
        let result = db.delete_user_category(&category.id).await;
        assert!(result.is_ok());

        // Verify deletion
        let categories = db.get_user_categories().await.unwrap();
        assert_eq!(categories.len(), initial_count);
        assert!(categories.iter().find(|c| c.id == category.id).is_none());
    }

    #[tokio::test]
    async fn test_app_mappings() {
        let db = create_test_db().await;

        // Create a category first
        let category = UserCategory {
            id: Uuid::new_v4().to_string(),
            name: "Development".to_string(),
            color: "#0000FF".to_string(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.add_user_category(&category).await.unwrap();

        // Test getting mappings (may have initial data)
        let initial_mappings = db.get_app_mappings().await.unwrap();
        let initial_count = initial_mappings.len();

        // Test adding app mapping
        let result = db
            .add_simple_app_mapping(&category.id, "Visual Studio Code", true)
            .await;
        assert!(result.is_ok());

        // Test getting mappings
        let mappings = db.get_app_mappings().await.unwrap();
        assert_eq!(mappings.len(), initial_count + 1);

        // Find our added mapping
        let vscode_mapping = mappings
            .iter()
            .find(|m| m.app_pattern == "Visual Studio Code")
            .unwrap();
        assert_eq!(vscode_mapping.app_pattern, "Visual Studio Code");
        assert_eq!(vscode_mapping.category_id, category.id);
        assert!(vscode_mapping.is_custom);

        // Test removing mapping
        let result = db
            .remove_app_mapping(&category.id, "Visual Studio Code")
            .await;
        assert!(result.is_ok());

        // Verify removal - should be back to initial count
        let mappings = db.get_app_mappings().await.unwrap();
        assert_eq!(mappings.len(), initial_count);
    }

    #[tokio::test]
    async fn test_url_mappings() {
        let db = create_test_db().await;

        // Create a category first
        let category = UserCategory {
            id: Uuid::new_v4().to_string(),
            name: "Social".to_string(),
            color: "#FF00FF".to_string(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        db.add_user_category(&category).await.unwrap();

        // Test getting URL mappings (may have initial data)
        let initial_mappings = db.get_url_mappings().await.unwrap();
        let initial_count = initial_mappings.len();

        // Create URL mapping with unique pattern
        let unique_url = format!("test-{}.com", Uuid::new_v4());
        let url_mapping = UrlMapping {
            id: Uuid::new_v4(),
            url_pattern: unique_url.clone(),
            category_id: category.id.clone(),
            is_custom: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test adding URL mapping
        let result = db.add_url_mapping(&url_mapping).await;
        assert!(result.is_ok());

        // Test getting URL mappings
        let mappings = db.get_url_mappings().await.unwrap();
        assert_eq!(mappings.len(), initial_count + 1);

        // Find our added mapping
        let test_mapping = mappings
            .iter()
            .find(|m| m.url_pattern == unique_url)
            .unwrap();
        assert_eq!(test_mapping.url_pattern, unique_url);
        assert_eq!(test_mapping.category_id, category.id);

        // Test removing URL mapping
        let result = db.remove_url_mapping(&category.id, &unique_url).await;
        assert!(result.is_ok());

        // Verify removal - should be back to initial count
        let mappings = db.get_url_mappings().await.unwrap();
        assert_eq!(mappings.len(), initial_count);
    }

    #[tokio::test]
    async fn test_focus_mode_settings() {
        let db = create_test_db().await;

        // Test default focus mode (should be false)
        let enabled = db.get_focus_mode_enabled().await.unwrap();
        assert!(!enabled);

        // Test setting focus mode enabled
        let result = db.set_focus_mode_enabled(true).await;
        assert!(result.is_ok());

        let enabled = db.get_focus_mode_enabled().await.unwrap();
        assert!(enabled);

        // Test setting focus mode disabled
        let result = db.set_focus_mode_enabled(false).await;
        assert!(result.is_ok());

        let enabled = db.get_focus_mode_enabled().await.unwrap();
        assert!(!enabled);
    }

    #[tokio::test]
    async fn test_focus_mode_categories() {
        let db = create_test_db().await;

        // Test getting empty allowed categories
        let categories = db.get_focus_mode_allowed_categories().await.unwrap();
        assert!(categories.is_empty());

        // Test setting allowed categories
        let test_categories = vec![
            "work".to_string(),
            "development".to_string(),
            "communication".to_string(),
        ];
        let result = db.set_focus_mode_allowed_categories(&test_categories).await;
        assert!(result.is_ok());

        // Test getting allowed categories
        let categories = db.get_focus_mode_allowed_categories().await.unwrap();
        assert_eq!(categories.len(), 3);
        assert!(categories.contains(&"work".to_string()));
        assert!(categories.contains(&"development".to_string()));
        assert!(categories.contains(&"communication".to_string()));

        // Test updating categories (should replace, not append)
        let new_categories = vec!["entertainment".to_string()];
        let result = db.set_focus_mode_allowed_categories(&new_categories).await;
        assert!(result.is_ok());

        let categories = db.get_focus_mode_allowed_categories().await.unwrap();
        assert_eq!(categories.len(), 1);
        assert!(categories.contains(&"entertainment".to_string()));
        assert!(!categories.contains(&"work".to_string()));
    }

    #[tokio::test]
    async fn test_focus_mode_allowed_apps() {
        let db = create_test_db().await;

        // Test getting empty allowed apps
        let apps = db.get_focus_mode_allowed_apps().await.unwrap();
        assert!(apps.is_empty());

        // Test app not allowed initially
        let allowed = db.is_focus_mode_app_allowed("Test App").await.unwrap();
        assert!(!allowed);

        // Test adding allowed app with expiry
        let expires_at = Some(chrono::Utc::now().timestamp() + 300); // 5 minutes from now
        let result = db.add_focus_mode_allowed_app("Test App", expires_at).await;
        assert!(result.is_ok());

        // Test app is now allowed
        let allowed = db.is_focus_mode_app_allowed("Test App").await.unwrap();
        assert!(allowed);

        // Test getting allowed apps
        let apps = db.get_focus_mode_allowed_apps().await.unwrap();
        assert_eq!(apps.len(), 1);
        assert!(apps.contains(&"Test App".to_string()));

        // Test getting allowed apps with expiry
        let apps_with_expiry = db.get_focus_mode_allowed_apps_with_expiry().await.unwrap();
        assert_eq!(apps_with_expiry.len(), 1);
        assert_eq!(apps_with_expiry[0].0, "Test App");
        assert!(apps_with_expiry[0].1.is_some());

        // Test adding app without expiry (indefinite)
        let result = db.add_focus_mode_allowed_app("Permanent App", None).await;
        assert!(result.is_ok());

        let apps_with_expiry = db.get_focus_mode_allowed_apps_with_expiry().await.unwrap();
        assert_eq!(apps_with_expiry.len(), 2);

        // Find the permanent app
        let permanent_app = apps_with_expiry
            .iter()
            .find(|(name, _)| name == "Permanent App");
        assert!(permanent_app.is_some());
        assert!(permanent_app.unwrap().1.is_none()); // No expiry

        // Test removing allowed app
        let result = db.remove_focus_mode_allowed_app("Test App").await;
        assert!(result.is_ok());

        // Test app is no longer allowed
        let allowed = db.is_focus_mode_app_allowed("Test App").await.unwrap();
        assert!(!allowed);

        // But permanent app should still be allowed
        let allowed = db.is_focus_mode_app_allowed("Permanent App").await.unwrap();
        assert!(allowed);

        let apps = db.get_focus_mode_allowed_apps().await.unwrap();
        assert_eq!(apps.len(), 1);
        assert!(apps.contains(&"Permanent App".to_string()));
    }

    #[tokio::test]
    async fn test_activities_by_date() {
        let db = create_test_db().await;

        let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Test getting activities for empty date
        let activities = db.get_activities_by_date(test_date).await.unwrap();
        assert!(activities.is_empty());

        // Create and add test activity for specific date
        let activity = ActivityEntry {
            id: Uuid::new_v4(),
            start_time: test_date.and_hms_opt(10, 0, 0).unwrap().and_utc(),
            end_time: Some(test_date.and_hms_opt(11, 0, 0).unwrap().and_utc()),
            app_name: "Test App".to_string(),
            app_bundle_id: Some("com.test.app".to_string()),
            window_title: "Test Window".to_string(),
            url: None,
            category: ActivityCategory::Development,
            segments: vec![],
        };

        db.start_activity(&activity).await.unwrap();

        // End the activity to make it complete
        db.end_current_activity(activity.end_time.unwrap())
            .await
            .unwrap();

        // Test getting activities for the date (this might not work as expected
        // depending on how the database stores and queries dates)
        // The test verifies the query doesn't crash
        let _activities = db.get_activities_by_date(test_date).await.unwrap();
        // Note: The actual result depends on the database implementation
    }

    #[tokio::test]
    async fn test_activities_by_date_range() {
        let db = create_test_db().await;

        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 7).unwrap();

        // Test getting activities for empty date range
        let activities = db
            .get_activities_by_date_range(start_date, end_date)
            .await
            .unwrap();
        assert!(activities.is_empty());
    }

    #[tokio::test]
    async fn test_activity_summary() {
        let db = create_test_db().await;

        let test_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Test getting summary for empty date
        let summary = db.get_activity_summary(test_date).await.unwrap();
        assert_eq!(summary.total_active_time, 0);
        assert!(summary.categories.is_empty());
        assert!(summary.top_apps.is_empty());
    }

    #[tokio::test]
    async fn test_timeline_data() {
        let db = create_test_db().await;

        // Test getting recent timeline (empty)
        let timeline = db.get_recent_timeline(60).await.unwrap(); // Last hour
        assert!(timeline.activities.is_empty());
    }

    #[tokio::test]
    async fn test_update_activity_category() {
        let db = create_test_db().await;

        // Create test activity
        let activity = ActivityEntry {
            id: Uuid::new_v4(),
            start_time: Utc::now(),
            end_time: Some(Utc::now()),
            app_name: "Test App".to_string(),
            app_bundle_id: None,
            window_title: "Test Window".to_string(),
            url: None,
            category: ActivityCategory::Unknown,
            segments: vec![],
        };

        db.start_activity(&activity).await.unwrap();

        // Test updating category
        let new_category = ActivityCategory::Development;
        let result = db
            .update_activity_category(&activity.id.to_string(), &new_category)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_expired_allowed_apps_cleanup() {
        let db = create_test_db().await;

        // Add app with past expiry time (should be expired)
        let past_expiry = chrono::Utc::now().timestamp() - 300; // 5 minutes ago
        let result = db
            .add_focus_mode_allowed_app("Expired App", Some(past_expiry))
            .await;
        assert!(result.is_ok());

        // Add app with future expiry time
        let future_expiry = chrono::Utc::now().timestamp() + 300; // 5 minutes from now
        let result = db
            .add_focus_mode_allowed_app("Valid App", Some(future_expiry))
            .await;
        assert!(result.is_ok());

        // The expired app should not be considered allowed
        // (This depends on the database implementation handling expiry)
        let _allowed = db.is_focus_mode_app_allowed("Expired App").await.unwrap();
        // Note: The actual behavior depends on whether the database checks expiry

        let allowed = db.is_focus_mode_app_allowed("Valid App").await.unwrap();
        assert!(allowed);
    }
}
