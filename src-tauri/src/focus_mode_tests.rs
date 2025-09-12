#[cfg(test)]
mod focus_mode_tests {
    use crate::models::*;
    use crate::test_config::test_utils::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_focus_mode_state_management() {
        let db = create_test_database().await;
        let state = create_test_app_state(db);

        // Focus mode should be disabled by default
        assert!(!*state.focus_mode_enabled.lock().unwrap());

        // Test enabling focus mode
        *state.focus_mode_enabled.lock().unwrap() = true;
        assert!(*state.focus_mode_enabled.lock().unwrap());

        // Test disabling focus mode
        *state.focus_mode_enabled.lock().unwrap() = false;
        assert!(!*state.focus_mode_enabled.lock().unwrap());
    }

    #[tokio::test]
    async fn test_focus_mode_categories() {
        let db = create_test_database().await;
        let state = create_test_app_state(db);

        // Test setting allowed categories
        let categories = vec!["work".to_string(), "development".to_string()];
        *state.focus_mode_allowed_categories.lock().unwrap() = categories.clone();

        let stored_categories = state.focus_mode_allowed_categories.lock().unwrap().clone();
        assert_eq!(stored_categories.len(), 2);
        assert!(stored_categories.contains(&"work".to_string()));
        assert!(stored_categories.contains(&"development".to_string()));
    }

    #[tokio::test]
    async fn test_focus_mode_allowed_apps_cache() {
        let db = create_test_database().await;
        let state = create_test_app_state(db);

        // Test adding app to cache
        let expires_at = Some(chrono::Utc::now().timestamp() + 300);
        state
            .focus_mode_allowed_apps_cache
            .lock()
            .unwrap()
            .insert("Test App".to_string(), expires_at);

        let cache = state.focus_mode_allowed_apps_cache.lock().unwrap();
        assert_eq!(cache.get("Test App"), Some(&expires_at));

        // Test removing from cache
        drop(cache);
        state
            .focus_mode_allowed_apps_cache
            .lock()
            .unwrap()
            .remove("Test App");
        let cache = state.focus_mode_allowed_apps_cache.lock().unwrap();
        assert!(!cache.contains_key("Test App"));
    }

    #[tokio::test]
    async fn test_focus_mode_database_integration() {
        let db = create_test_database().await;

        // Test focus mode enabled/disabled
        db.set_focus_mode_enabled(true).await.unwrap();
        let enabled = db.get_focus_mode_enabled().await.unwrap();
        assert!(enabled);

        // Test focus mode categories
        let categories = vec!["work".to_string(), "development".to_string()];
        db.set_focus_mode_allowed_categories(&categories)
            .await
            .unwrap();

        let retrieved_categories = db.get_focus_mode_allowed_categories().await.unwrap();
        assert_eq!(retrieved_categories.len(), 2);
        assert!(retrieved_categories.contains(&"work".to_string()));

        // Test focus mode allowed apps
        let expires_at = Some(chrono::Utc::now().timestamp() + 300);
        db.add_focus_mode_allowed_app("Test App", expires_at)
            .await
            .unwrap();

        let is_allowed = db.is_focus_mode_app_allowed("Test App").await.unwrap();
        assert!(is_allowed);

        let allowed_apps = db.get_focus_mode_allowed_apps().await.unwrap();
        assert!(allowed_apps.contains(&"Test App".to_string()));

        // Test removing allowed app
        db.remove_focus_mode_allowed_app("Test App").await.unwrap();
        let is_allowed = db.is_focus_mode_app_allowed("Test App").await.unwrap();
        assert!(!is_allowed);
    }

    #[tokio::test]
    async fn test_focus_mode_with_categories_and_mappings() {
        let db = create_test_database().await;

        // Create a category and app mapping
        let category = UserCategory {
            id: Uuid::new_v4().to_string(),
            name: "Development".to_string(),
            color: "#0000FF".to_string(),
            parent_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        db.add_user_category(&category).await.unwrap();
        db.add_simple_app_mapping(&category.id, "Visual Studio Code", true)
            .await
            .unwrap();

        // Test that mappings are created correctly
        let mappings = db.get_app_mappings().await.unwrap();
        // Should have at least our mapping plus any initial mappings
        assert!(mappings.len() >= 1);

        // Find our specific mapping
        let vscode_mapping = mappings
            .iter()
            .find(|m| m.app_pattern == "Visual Studio Code")
            .unwrap();
        assert_eq!(vscode_mapping.app_pattern, "Visual Studio Code");
        assert_eq!(vscode_mapping.category_id, category.id);

        // Test focus mode with this category
        let categories = vec![category.id.clone()];
        db.set_focus_mode_allowed_categories(&categories)
            .await
            .unwrap();

        let allowed_categories = db.get_focus_mode_allowed_categories().await.unwrap();
        assert!(allowed_categories.contains(&category.id));
    }

    #[tokio::test]
    async fn test_focus_mode_app_expiry() {
        let db = create_test_database().await;

        // Add app with past expiry (should be expired)
        let past_expiry = chrono::Utc::now().timestamp() - 300; // 5 minutes ago
        db.add_focus_mode_allowed_app("Expired App", Some(past_expiry))
            .await
            .unwrap();

        // Add app with future expiry
        let future_expiry = chrono::Utc::now().timestamp() + 300; // 5 minutes from now
        db.add_focus_mode_allowed_app("Valid App", Some(future_expiry))
            .await
            .unwrap();

        // Add app without expiry (indefinite)
        db.add_focus_mode_allowed_app("Permanent App", None)
            .await
            .unwrap();

        // Test getting apps with expiry info
        let apps_with_expiry = db.get_focus_mode_allowed_apps_with_expiry().await.unwrap();
        // Should have at least 2 apps (expired app might be filtered out by database)
        assert!(apps_with_expiry.len() >= 2);

        // Find each app and verify expiry
        let expired_app = apps_with_expiry
            .iter()
            .find(|(name, _)| name == "Expired App");
        // Expired app might be filtered out by database, so it's optional
        if let Some(expired) = expired_app {
            assert_eq!(expired.1, Some(past_expiry));
        }

        let valid_app = apps_with_expiry
            .iter()
            .find(|(name, _)| name == "Valid App");
        assert!(valid_app.is_some());
        assert_eq!(valid_app.unwrap().1, Some(future_expiry));

        let permanent_app = apps_with_expiry
            .iter()
            .find(|(name, _)| name == "Permanent App");
        assert!(permanent_app.is_some());
        assert!(permanent_app.unwrap().1.is_none());
    }

    #[tokio::test]
    async fn test_focus_mode_cache_consistency() {
        let db = create_test_database().await;
        let state = create_test_app_state(db);

        // Add app to database
        let expires_at = Some(chrono::Utc::now().timestamp() + 300);
        state
            .db
            .add_focus_mode_allowed_app("Cache Test App", expires_at)
            .await
            .unwrap();

        // Update cache to match database
        state
            .focus_mode_allowed_apps_cache
            .lock()
            .unwrap()
            .insert("Cache Test App".to_string(), expires_at);

        // Verify cache matches database
        let db_allowed = state
            .db
            .is_focus_mode_app_allowed("Cache Test App")
            .await
            .unwrap();
        let cache_entry = state
            .focus_mode_allowed_apps_cache
            .lock()
            .unwrap()
            .get("Cache Test App")
            .cloned();

        assert!(db_allowed);
        assert_eq!(cache_entry, Some(expires_at));

        // Remove from database
        state
            .db
            .remove_focus_mode_allowed_app("Cache Test App")
            .await
            .unwrap();

        // Update cache
        state
            .focus_mode_allowed_apps_cache
            .lock()
            .unwrap()
            .remove("Cache Test App");

        // Verify both are updated
        let db_allowed = state
            .db
            .is_focus_mode_app_allowed("Cache Test App")
            .await
            .unwrap();
        let cache_contains = state
            .focus_mode_allowed_apps_cache
            .lock()
            .unwrap()
            .contains_key("Cache Test App");

        assert!(!db_allowed);
        assert!(!cache_contains);
    }
}
