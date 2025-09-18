use crate::models::{
    ActivityCategory, ActivityEntry, ActivitySummary, AppMapping, AppSummary, CategorySummary,
    TimelineActivity, TimelineData, UrlMapping, UserCategory,
};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        println!("Connecting to database: {}", database_url);

        // Use connection options for SQLite with create_if_missing
        let pool = SqlitePool::connect(&format!("{}?mode=rwc", database_url)).await?;

        // Apply migrations using our custom migration system
        crate::migrations::apply_migrations(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn start_activity(&self, entry: &ActivityEntry) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO activity_entries (id, start_time, end_time, app_name, app_bundle_id, window_title, url, category)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(entry.id.to_string())
        .bind(entry.start_time.to_rfc3339())
        .bind(entry.end_time.as_ref().map(|dt| dt.to_rfc3339()))
        .bind(&entry.app_name)
        .bind(&entry.app_bundle_id)
        .bind(&entry.window_title)
        .bind(&entry.url)
        .bind(serde_json::to_string(&entry.category).unwrap())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn end_current_activity(&self, end_time: DateTime<Utc>) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE activity_entries 
            SET end_time = ?1 
            WHERE end_time IS NULL
            "#,
        )
        .bind(end_time.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_current_activity(&self) -> Result<Option<ActivityEntry>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, start_time, end_time, app_name, app_bundle_id, window_title, url, category
            FROM activity_entries
            WHERE end_time IS NULL
            ORDER BY start_time DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let entry = ActivityEntry {
                id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                start_time: DateTime::parse_from_rfc3339(&row.get::<String, _>("start_time"))
                    .unwrap()
                    .with_timezone(&Utc),
                end_time: row.get::<Option<String>, _>("end_time").map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                app_name: row.get("app_name"),
                app_bundle_id: row.get("app_bundle_id"),
                window_title: row.get("window_title"),
                url: row.get("url"),
                category: serde_json::from_str(&row.get::<String, _>("category")).unwrap(),
                segments: vec![], // TODO: Load segments separately
            };
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    pub async fn get_activities_by_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<ActivityEntry>, sqlx::Error> {
        let start_of_day = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_of_day = date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        let rows = sqlx::query(
            r#"
            SELECT id, start_time, end_time, app_name, app_bundle_id, window_title, url, category
            FROM activity_entries
            WHERE start_time >= ?1 AND start_time <= ?2
            ORDER BY start_time ASC
            "#,
        )
        .bind(start_of_day.to_rfc3339())
        .bind(end_of_day.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut activities = Vec::new();
        for row in rows {
            let category: ActivityCategory =
                serde_json::from_str(&row.get::<String, _>("category"))
                    .unwrap_or(ActivityCategory::Unknown);

            activities.push(ActivityEntry {
                id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                start_time: DateTime::parse_from_rfc3339(&row.get::<String, _>("start_time"))
                    .unwrap()
                    .with_timezone(&Utc),
                end_time: row.get::<Option<String>, _>("end_time").map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                app_name: row.get("app_name"),
                app_bundle_id: row.get("app_bundle_id"),
                window_title: row.get("window_title"),
                url: row.get("url"),
                category,
                segments: vec![], // TODO: Load segments separately
            });
        }

        Ok(activities)
    }

    pub async fn get_activities_by_date_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ActivityEntry>, sqlx::Error> {
        let start_of_period = start_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_of_period = end_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

        let rows = sqlx::query(
            r#"
            SELECT id, start_time, end_time, app_name, app_bundle_id, window_title, url, category
            FROM activity_entries
            WHERE start_time >= ?1 AND start_time <= ?2
            ORDER BY start_time ASC
            "#,
        )
        .bind(start_of_period.to_rfc3339())
        .bind(end_of_period.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut activities = Vec::new();
        for row in rows {
            let category: ActivityCategory =
                serde_json::from_str(&row.get::<String, _>("category"))
                    .unwrap_or(ActivityCategory::Unknown);

            activities.push(ActivityEntry {
                id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                start_time: DateTime::parse_from_rfc3339(&row.get::<String, _>("start_time"))
                    .unwrap()
                    .with_timezone(&Utc),
                end_time: row.get::<Option<String>, _>("end_time").map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                app_name: row.get("app_name"),
                app_bundle_id: row.get("app_bundle_id"),
                window_title: row.get("window_title"),
                url: row.get("url"),
                category,
                segments: vec![], // TODO: Load segments separately
            });
        }

        Ok(activities)
    }

    pub async fn get_activity_summary(
        &self,
        date: NaiveDate,
    ) -> Result<ActivitySummary, sqlx::Error> {
        let activities = self.get_activities_by_date(date).await?;

        // Calculate duration for each activity
        let activities_with_duration: Vec<(ActivityEntry, i64)> = activities
            .into_iter()
            .filter_map(|activity| {
                if let Some(end_time) = activity.end_time {
                    let duration = end_time
                        .signed_duration_since(activity.start_time)
                        .num_seconds();
                    Some((activity, duration))
                } else {
                    // For ongoing activities, calculate duration from start_time to now
                    let now = Utc::now();
                    let duration = now.signed_duration_since(activity.start_time).num_seconds();
                    Some((activity, duration))
                }
            })
            .collect();

        let total_active_time: i64 = activities_with_duration
            .iter()
            .map(|(_, duration)| *duration)
            .sum();

        // Calculate category summaries
        let mut category_durations = std::collections::HashMap::new();
        for (activity, duration) in &activities_with_duration {
            *category_durations
                .entry(activity.category.clone())
                .or_insert(0) += duration;
        }

        let categories: Vec<CategorySummary> = category_durations
            .into_iter()
            .map(|(category, duration)| CategorySummary {
                category,
                duration_seconds: duration,
                percentage: if total_active_time > 0 {
                    (duration as f64 / total_active_time as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        // Calculate app summaries
        let mut app_durations = std::collections::HashMap::new();
        for (activity, duration) in &activities_with_duration {
            *app_durations.entry(activity.app_name.clone()).or_insert(0) += duration;
        }

        let mut top_apps: Vec<AppSummary> = app_durations
            .into_iter()
            .map(|(app_name, duration)| AppSummary {
                app_name,
                duration_seconds: duration,
                percentage: if total_active_time > 0 {
                    (duration as f64 / total_active_time as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        top_apps.sort_by(|a, b| b.duration_seconds.cmp(&a.duration_seconds));
        top_apps.truncate(10); // Top 10 apps

        Ok(ActivitySummary {
            date: date.to_string(),
            total_active_time,
            categories,
            top_apps,
            detailed_activities: vec![], // TODO: Implement detailed activities with segments
        })
    }

    pub async fn get_recent_timeline(&self, minutes: i64) -> Result<TimelineData, sqlx::Error> {
        let now = Utc::now();
        let start_time = now - Duration::minutes(minutes);

        let rows = sqlx::query(
            r#"
            SELECT 
                id, start_time, end_time, app_name, app_bundle_id, 
                window_title, url, category
            FROM activity_entries 
            WHERE start_time >= ? OR (start_time < ? AND (end_time IS NULL OR end_time >= ?))
            ORDER BY start_time ASC
            "#,
        )
        .bind(start_time.to_rfc3339())
        .bind(start_time.to_rfc3339())
        .bind(start_time.to_rfc3339())
        .fetch_all(&self.pool)
        .await?;

        let mut activities = Vec::new();
        for row in rows {
            let start_time_parsed =
                DateTime::parse_from_rfc3339(&row.get::<String, _>("start_time"))
                    .unwrap()
                    .with_timezone(&Utc);

            let end_time_parsed = row.get::<Option<String>, _>("end_time").map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .unwrap()
                    .with_timezone(&Utc)
            });

            // Calculate actual start and end times within the timeline window
            let timeline_start = start_time_parsed.max(start_time);
            let timeline_end = end_time_parsed.unwrap_or(now).min(now);

            // Only include if there's actual overlap with our timeline
            if timeline_start < timeline_end {
                let duration_seconds = (timeline_end - timeline_start).num_seconds();

                let category: ActivityCategory =
                    serde_json::from_str(&row.get::<String, _>("category"))
                        .unwrap_or(ActivityCategory::Unknown);

                activities.push(TimelineActivity {
                    id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                    start_time: timeline_start,
                    end_time: Some(timeline_end),
                    duration_seconds,
                    app_name: row.get("app_name"),
                    app_bundle_id: row.get("app_bundle_id"),
                    window_title: row.get("window_title"),
                    url: row.get("url"),
                    category,
                    segments: vec![], // TODO: Load segments
                });
            }
        }

        Ok(TimelineData {
            start_time,
            end_time: now,
            activities,
        })
    }

    pub async fn update_activity_category(
        &self,
        activity_id: &str,
        category: &ActivityCategory,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE activity_entries 
            SET category = ?1 
            WHERE id = ?2
            "#,
        )
        .bind(serde_json::to_string(category).unwrap())
        .bind(activity_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // User category management
    pub async fn add_user_category(&self, category: &UserCategory) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO user_categories (id, name, color, parent_id, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(&category.id)
        .bind(&category.name)
        .bind(&category.color)
        .bind(&category.parent_id)
        .bind(category.created_at.to_rfc3339())
        .bind(category.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_categories(&self) -> Result<Vec<UserCategory>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, color, parent_id, created_at, updated_at
            FROM user_categories
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut categories = Vec::new();
        for row in rows {
            let created_at = DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .map_err(|_| sqlx::Error::Decode("Invalid created_at format".into()))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                .map_err(|_| sqlx::Error::Decode("Invalid updated_at format".into()))?
                .with_timezone(&Utc);

            categories.push(UserCategory {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
                parent_id: row.get("parent_id"),
                created_at,
                updated_at,
            });
        }

        Ok(categories)
    }

    pub async fn update_user_category(&self, category: &UserCategory) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE user_categories 
            SET name = ?2, color = ?3, parent_id = ?4, updated_at = ?5
            WHERE id = ?1
            "#,
        )
        .bind(&category.id)
        .bind(&category.name)
        .bind(&category.color)
        .bind(&category.parent_id)
        .bind(category.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_category_by_id(
        &self,
        id: &str,
    ) -> Result<Option<UserCategory>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, name, color, parent_id, created_at, updated_at
            FROM user_categories
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let created_at = DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                    .map_err(|_| sqlx::Error::Decode("Invalid created_at format".into()))?
                    .with_timezone(&Utc);
                let updated_at = DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                    .map_err(|_| sqlx::Error::Decode("Invalid updated_at format".into()))?
                    .with_timezone(&Utc);

                Ok(Some(UserCategory {
                    id: row.get("id"),
                    name: row.get("name"),
                    color: row.get("color"),
                    parent_id: row.get("parent_id"),
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_user_category(&self, id: &str) -> Result<(), sqlx::Error> {
        // First, get the category to check if it's "Unknown"
        let category = self.get_user_category_by_id(id).await?;
        if let Some(cat) = category {
            if cat.name.to_lowercase() == "unknown" {
                return Err(sqlx::Error::RowNotFound); // Return error for "Unknown" category
            }

            // Reassign all activities with this category to "Unknown"
            let unknown_category = crate::models::ActivityCategory::Unknown;
            sqlx::query(
                r#"
                UPDATE activity_entries 
                SET category = ?1 
                WHERE category = ?2
                "#,
            )
            .bind(serde_json::to_string(&unknown_category).unwrap())
            .bind(
                serde_json::to_string(&crate::models::ActivityCategory::Custom(cat.id.clone()))
                    .unwrap(),
            )
            .execute(&self.pool)
            .await?;

            // Delete app mappings for this category
            sqlx::query("DELETE FROM app_mappings WHERE category_id = ?1")
                .bind(&cat.id)
                .execute(&self.pool)
                .await?;

            // Delete URL mappings for this category
            sqlx::query("DELETE FROM url_mappings WHERE category_id = ?1")
                .bind(&cat.id)
                .execute(&self.pool)
                .await?;
        }

        // Finally, delete the category
        sqlx::query("DELETE FROM user_categories WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // App mapping management
    pub async fn add_app_mapping(&self, mapping: &AppMapping) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO app_mappings (id, app_pattern, category_id, is_custom, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(mapping.id.to_string())
        .bind(&mapping.app_pattern)
        .bind(&mapping.category_id)
        .bind(mapping.is_custom as i32)
        .bind(mapping.created_at.to_rfc3339())
        .bind(mapping.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_app_mappings(&self) -> Result<Vec<AppMapping>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, app_pattern, category_id, is_custom, created_at, updated_at
            FROM app_mappings
            ORDER BY app_pattern
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut mappings = Vec::new();
        for row in rows {
            let id = Uuid::parse_str(&row.get::<String, _>("id"))
                .map_err(|_| sqlx::Error::Decode("Invalid UUID format".into()))?;
            let created_at = DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .map_err(|_| sqlx::Error::Decode("Invalid created_at format".into()))?
                .with_timezone(&Utc);
            let updated_at = DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                .map_err(|_| sqlx::Error::Decode("Invalid updated_at format".into()))?
                .with_timezone(&Utc);

            mappings.push(AppMapping {
                id,
                app_pattern: row.get("app_pattern"),
                category_id: row.get("category_id"),
                is_custom: row.get::<i32, _>("is_custom") != 0,
                created_at,
                updated_at,
            });
        }

        Ok(mappings)
    }

    #[allow(dead_code)]
    pub async fn update_app_mapping(&self, mapping: &AppMapping) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE app_mappings 
            SET app_pattern = ?2, category_id = ?3, is_custom = ?4, updated_at = ?5
            WHERE id = ?1
            "#,
        )
        .bind(mapping.id.to_string())
        .bind(&mapping.app_pattern)
        .bind(&mapping.category_id)
        .bind(mapping.is_custom as i32)
        .bind(mapping.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete_app_mapping(&self, id: &Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM app_mappings WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn remove_app_mapping(
        &self,
        category_id: &str,
        app_pattern: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM app_mappings WHERE category_id = ?1 AND app_pattern = ?2")
            .bind(category_id)
            .bind(app_pattern)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn add_simple_app_mapping(
        &self,
        category_id: &str,
        app_pattern: &str,
        is_custom: bool,
    ) -> Result<(), sqlx::Error> {
        let mapping = AppMapping {
            id: Uuid::new_v4(),
            app_pattern: app_pattern.to_string(),
            category_id: category_id.to_string(),
            is_custom,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.add_app_mapping(&mapping).await
    }

    // URL Mapping methods
    pub async fn add_url_mapping(&self, mapping: &UrlMapping) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO url_mappings (id, url_pattern, category_id, is_custom, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(mapping.id.to_string())
        .bind(&mapping.url_pattern)
        .bind(&mapping.category_id)
        .bind(mapping.is_custom as i32)
        .bind(mapping.created_at.to_rfc3339())
        .bind(mapping.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_url_mappings(&self) -> Result<Vec<UrlMapping>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, url_pattern, category_id, is_custom, created_at, updated_at
            FROM url_mappings
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut mappings = Vec::new();
        for row in rows {
            let id_str: String = row.get("id");
            let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

            let created_at_str: String = row.get("created_at");
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                .with_timezone(&Utc);

            let updated_at_str: String = row.get("updated_at");
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| sqlx::Error::Decode(Box::new(e)))?
                .with_timezone(&Utc);

            mappings.push(UrlMapping {
                id,
                url_pattern: row.get("url_pattern"),
                category_id: row.get("category_id"),
                is_custom: row.get::<i32, _>("is_custom") != 0,
                created_at,
                updated_at,
            });
        }

        Ok(mappings)
    }

    #[allow(dead_code)]
    pub async fn update_url_mapping(&self, mapping: &UrlMapping) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE url_mappings 
            SET url_pattern = ?2, category_id = ?3, is_custom = ?4, updated_at = ?5
            WHERE id = ?1
            "#,
        )
        .bind(mapping.id.to_string())
        .bind(&mapping.url_pattern)
        .bind(&mapping.category_id)
        .bind(mapping.is_custom as i32)
        .bind(mapping.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_url_mapping(
        &self,
        category_id: &str,
        url_pattern: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM url_mappings WHERE category_id = ?1 AND url_pattern = ?2")
            .bind(category_id)
            .bind(url_pattern)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn add_simple_url_mapping(
        &self,
        category_id: &str,
        url_pattern: &str,
        is_custom: bool,
    ) -> Result<(), sqlx::Error> {
        let mapping = UrlMapping {
            id: Uuid::new_v4(),
            url_pattern: url_pattern.to_string(),
            category_id: category_id.to_string(),
            is_custom,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.add_url_mapping(&mapping).await
    }

    // Focus Mode Database Functions

    /// Get focus mode enabled status
    #[allow(dead_code)]
    pub async fn get_focus_mode_enabled(&self) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT value FROM focus_mode_settings WHERE key = 'enabled'")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let value: String = row.get("value");
            Ok(value == "1")
        } else {
            Ok(false) // Default to disabled
        }
    }

    /// Set focus mode enabled status
    #[allow(dead_code)]
    pub async fn set_focus_mode_enabled(&self, enabled: bool) -> Result<(), sqlx::Error> {
        let value = if enabled { "1" } else { "0" };
        sqlx::query(
            "INSERT OR REPLACE INTO focus_mode_settings (key, value) VALUES ('enabled', ?)",
        )
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get app blocking enabled status
    pub async fn get_app_blocking_enabled(&self) -> Result<bool, sqlx::Error> {
        let row =
            sqlx::query("SELECT value FROM focus_mode_settings WHERE key = 'app_blocking_enabled'")
                .fetch_optional(&self.pool)
                .await?;

        if let Some(row) = row {
            let value: String = row.get("value");
            Ok(value == "1")
        } else {
            Ok(true) // Default to enabled
        }
    }

    /// Set app blocking enabled status
    pub async fn set_app_blocking_enabled(&self, enabled: bool) -> Result<(), sqlx::Error> {
        let value = if enabled { "1" } else { "0" };
        sqlx::query(
            "INSERT OR REPLACE INTO focus_mode_settings (key, value) VALUES ('app_blocking_enabled', ?)",
        )
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get website blocking enabled status
    pub async fn get_website_blocking_enabled(&self) -> Result<bool, sqlx::Error> {
        let row = sqlx::query(
            "SELECT value FROM focus_mode_settings WHERE key = 'website_blocking_enabled'",
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let value: String = row.get("value");
            Ok(value == "1")
        } else {
            Ok(true) // Default to enabled
        }
    }

    /// Set website blocking enabled status
    pub async fn set_website_blocking_enabled(&self, enabled: bool) -> Result<(), sqlx::Error> {
        let value = if enabled { "1" } else { "0" };
        sqlx::query(
            "INSERT OR REPLACE INTO focus_mode_settings (key, value) VALUES ('website_blocking_enabled', ?)",
        )
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get allowed category IDs for focus mode
    pub async fn get_focus_mode_allowed_categories(&self) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT category_id FROM focus_mode_allowed_categories ORDER BY category_id",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.get("category_id")).collect())
    }

    /// Set allowed category IDs for focus mode (replaces existing)
    pub async fn set_focus_mode_allowed_categories(
        &self,
        category_ids: &[String],
    ) -> Result<(), sqlx::Error> {
        // Clear existing categories
        sqlx::query("DELETE FROM focus_mode_allowed_categories")
            .execute(&self.pool)
            .await?;

        // Insert new categories
        for category_id in category_ids {
            sqlx::query("INSERT INTO focus_mode_allowed_categories (category_id) VALUES (?)")
                .bind(category_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    /// Add a temporarily allowed app (with expiry)
    pub async fn add_focus_mode_allowed_app(
        &self,
        app_pattern: &str,
        expires_at: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO focus_mode_allowed_apps (app_pattern, expires_at) VALUES (?, ?)")
            .bind(app_pattern)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Remove an allowed app
    pub async fn remove_focus_mode_allowed_app(
        &self,
        app_pattern: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM focus_mode_allowed_apps WHERE app_pattern = ?")
            .bind(app_pattern)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get allowed apps (non-expired ones)
    pub async fn get_focus_mode_allowed_apps(&self) -> Result<Vec<String>, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let rows = sqlx::query(
            "SELECT app_pattern FROM focus_mode_allowed_apps 
             WHERE expires_at IS NULL OR expires_at > ?
             ORDER BY app_pattern",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.get("app_pattern")).collect())
    }

    /// Get allowed apps with their expiry times (for caching)
    pub async fn get_focus_mode_allowed_apps_with_expiry(
        &self,
    ) -> Result<Vec<(String, Option<i64>)>, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();
        let rows = sqlx::query(
            "SELECT app_pattern, expires_at FROM focus_mode_allowed_apps 
             WHERE expires_at IS NULL OR expires_at > ?
             ORDER BY app_pattern",
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let app_pattern: String = row.get("app_pattern");
                let expires_at: Option<i64> = row.get("expires_at");
                (app_pattern, expires_at)
            })
            .collect())
    }

    /// Check if an app is allowed (either permanently or temporarily)
    pub async fn is_focus_mode_app_allowed(&self, app_pattern: &str) -> Result<bool, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        // First try exact match
        let row = sqlx::query(
            "SELECT 1 FROM focus_mode_allowed_apps 
             WHERE app_pattern = ? AND (expires_at IS NULL OR expires_at > ?)
             LIMIT 1",
        )
        .bind(app_pattern)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        if row.is_some() {
            return Ok(true);
        }

        // If no exact match, try pattern matching (case-insensitive)
        let pattern_like = format!("%{}%", app_pattern.to_lowercase());
        let row = sqlx::query(
            "SELECT 1 FROM focus_mode_allowed_apps 
             WHERE LOWER(app_pattern) LIKE ? AND (expires_at IS NULL OR expires_at > ?)
             LIMIT 1",
        )
        .bind(&pattern_like)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        if row.is_some() {
            return Ok(true);
        }

        // Also check the reverse - if the stored pattern contains the app name
        let app_like = format!("%{}%", app_pattern.to_lowercase());
        let row = sqlx::query(
            "SELECT 1 FROM focus_mode_allowed_apps 
             WHERE ? LIKE LOWER(app_pattern) AND (expires_at IS NULL OR expires_at > ?)
             LIMIT 1",
        )
        .bind(&app_like)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.is_some())
    }

    /// Get proxy port setting
    pub async fn get_proxy_port(&self) -> Result<u16, sqlx::Error> {
        let row = sqlx::query("SELECT value FROM focus_mode_settings WHERE key = 'proxy_port'")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let value: String = row.get("value");
            Ok(value.parse::<u16>().unwrap_or(62828)) // Default to 62828 if parsing fails
        } else {
            Ok(62828) // Default port
        }
    }

    /// Set proxy port setting
    pub async fn set_proxy_port(&self, port: u16) -> Result<(), sqlx::Error> {
        let value = port.to_string();
        sqlx::query(
            "INSERT OR REPLACE INTO focus_mode_settings (key, value) VALUES ('proxy_port', ?)",
        )
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Pomodoro session methods
    pub async fn save_pomodoro_session(
        &self,
        session: &crate::models::PomodoroSession,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO pomodoro_sessions (id, session_type, start_time, end_time, duration_minutes, 
                                         actual_duration_seconds, work_description, completed, 
                                         focus_mode_enabled, app_tracking_enabled)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
        )
        .bind(session.id.to_string())
        .bind(match session.session_type {
            crate::models::PomodoroSessionType::Work => "work",
            crate::models::PomodoroSessionType::Break => "break",
        })
        .bind(session.start_time.to_rfc3339())
        .bind(session.end_time.as_ref().map(|dt| dt.to_rfc3339()))
        .bind(session.duration_minutes)
        .bind(session.actual_duration_seconds)
        .bind(&session.work_description)
        .bind(if session.completed { 1 } else { 0 })
        .bind(if session.focus_mode_enabled { 1 } else { 0 })
        .bind(if session.app_tracking_enabled { 1 } else { 0 })
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_pomodoro_session(
        &self,
        session: &crate::models::PomodoroSession,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE pomodoro_sessions 
            SET session_type = ?2, start_time = ?3, end_time = ?4, duration_minutes = ?5,
                actual_duration_seconds = ?6, work_description = ?7, completed = ?8,
                focus_mode_enabled = ?9, app_tracking_enabled = ?10
            WHERE id = ?1
            "#,
        )
        .bind(session.id.to_string())
        .bind(match session.session_type {
            crate::models::PomodoroSessionType::Work => "work",
            crate::models::PomodoroSessionType::Break => "break",
        })
        .bind(session.start_time.to_rfc3339())
        .bind(session.end_time.as_ref().map(|dt| dt.to_rfc3339()))
        .bind(session.duration_minutes)
        .bind(session.actual_duration_seconds)
        .bind(&session.work_description)
        .bind(if session.completed { 1 } else { 0 })
        .bind(if session.focus_mode_enabled { 1 } else { 0 })
        .bind(if session.app_tracking_enabled { 1 } else { 0 })
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pomodoro_session_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<crate::models::PomodoroSession>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, session_type, start_time, end_time, duration_minutes, 
                   actual_duration_seconds, work_description, completed, 
                   focus_mode_enabled, app_tracking_enabled
            FROM pomodoro_sessions 
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let session = crate::models::PomodoroSession {
                id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                session_type: match row.get::<String, _>("session_type").as_str() {
                    "work" => crate::models::PomodoroSessionType::Work,
                    "break" => crate::models::PomodoroSessionType::Break,
                    _ => crate::models::PomodoroSessionType::Work,
                },
                start_time: DateTime::parse_from_rfc3339(&row.get::<String, _>("start_time"))
                    .unwrap()
                    .with_timezone(&Utc),
                end_time: row.get::<Option<String>, _>("end_time").map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                duration_minutes: row.get("duration_minutes"),
                actual_duration_seconds: row.get("actual_duration_seconds"),
                work_description: row.get("work_description"),
                completed: row.get::<i32, _>("completed") == 1,
                focus_mode_enabled: row.get::<i32, _>("focus_mode_enabled") == 1,
                app_tracking_enabled: row.get::<i32, _>("app_tracking_enabled") == 1,
            };
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    pub async fn get_pomodoro_sessions(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
        session_type: Option<crate::models::PomodoroSessionType>,
        limit: Option<i64>,
    ) -> Result<Vec<crate::models::PomodoroSession>, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT id, session_type, start_time, end_time, duration_minutes, 
                   actual_duration_seconds, work_description, completed, 
                   focus_mode_enabled, app_tracking_enabled
            FROM pomodoro_sessions 
            WHERE 1=1
            "#,
        );

        let mut bind_params: Vec<String> = Vec::new();

        if let Some(start) = start_date {
            query.push_str(" AND DATE(start_time) >= ?");
            bind_params.push(start);
        }

        if let Some(end) = end_date {
            query.push_str(" AND DATE(start_time) <= ?");
            bind_params.push(end);
        }

        if let Some(s_type) = session_type {
            query.push_str(" AND session_type = ?");
            bind_params.push(match s_type {
                crate::models::PomodoroSessionType::Work => "work".to_string(),
                crate::models::PomodoroSessionType::Break => "break".to_string(),
            });
        }

        query.push_str(" ORDER BY start_time DESC");

        // Apply limit for performance (default to 10)
        let limit_val = limit.unwrap_or(10);
        query.push_str(" LIMIT ?");
        bind_params.push(limit_val.to_string());

        let mut sql_query = sqlx::query(&query);
        for param in bind_params {
            sql_query = sql_query.bind(param);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        let sessions = rows
            .into_iter()
            .map(|row| crate::models::PomodoroSession {
                id: Uuid::parse_str(&row.get::<String, _>("id")).unwrap(),
                session_type: match row.get::<String, _>("session_type").as_str() {
                    "work" => crate::models::PomodoroSessionType::Work,
                    "break" => crate::models::PomodoroSessionType::Break,
                    _ => crate::models::PomodoroSessionType::Work,
                },
                start_time: DateTime::parse_from_rfc3339(&row.get::<String, _>("start_time"))
                    .unwrap()
                    .with_timezone(&Utc),
                end_time: row.get::<Option<String>, _>("end_time").map(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                duration_minutes: row.get("duration_minutes"),
                actual_duration_seconds: row.get("actual_duration_seconds"),
                work_description: row.get("work_description"),
                completed: row.get::<i32, _>("completed") == 1,
                focus_mode_enabled: row.get::<i32, _>("focus_mode_enabled") == 1,
                app_tracking_enabled: row.get::<i32, _>("app_tracking_enabled") == 1,
            })
            .collect();

        Ok(sessions)
    }

    pub async fn delete_pomodoro_session(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM pomodoro_sessions WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_pomodoro_settings(
        &self,
    ) -> Result<crate::models::PomodoroSettings, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, work_duration_minutes, break_duration_minutes, enable_focus_mode, 
                   enable_app_tracking, auto_start_breaks, auto_start_work, updated_at
            FROM pomodoro_settings 
            WHERE id = 'default'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let settings = crate::models::PomodoroSettings {
            id: row.get("id"),
            work_duration_minutes: row.get("work_duration_minutes"),
            break_duration_minutes: row.get("break_duration_minutes"),
            enable_focus_mode: row.get::<i32, _>("enable_focus_mode") == 1,
            enable_app_tracking: row.get::<i32, _>("enable_app_tracking") == 1,
            auto_start_breaks: row.get::<i32, _>("auto_start_breaks") == 1,
            auto_start_work: row.get::<i32, _>("auto_start_work") == 1,
            updated_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("updated_at"))
                .unwrap()
                .with_timezone(&Utc),
        };

        Ok(settings)
    }

    pub async fn update_pomodoro_settings(
        &self,
        settings: &crate::models::PomodoroSettings,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE pomodoro_settings 
            SET work_duration_minutes = ?2, break_duration_minutes = ?3, enable_focus_mode = ?4,
                enable_app_tracking = ?5, auto_start_breaks = ?6, auto_start_work = ?7, updated_at = ?8
            WHERE id = ?1
            "#,
        )
        .bind(&settings.id)
        .bind(settings.work_duration_minutes)
        .bind(settings.break_duration_minutes)
        .bind(if settings.enable_focus_mode { 1 } else { 0 })
        .bind(if settings.enable_app_tracking { 1 } else { 0 })
        .bind(if settings.auto_start_breaks { 1 } else { 0 })
        .bind(if settings.auto_start_work { 1 } else { 0 })
        .bind(settings.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pomodoro_summary(
        &self,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<crate::models::PomodoroSummary, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT 
                COUNT(*) as total_sessions,
                SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) as completed_sessions,
                SUM(CASE WHEN session_type = 'work' AND completed = 1 THEN COALESCE(actual_duration_seconds, duration_minutes * 60) ELSE 0 END) as total_work_time,
                SUM(CASE WHEN session_type = 'break' AND completed = 1 THEN COALESCE(actual_duration_seconds, duration_minutes * 60) ELSE 0 END) as total_break_time,
                AVG(CASE WHEN completed = 1 THEN COALESCE(actual_duration_seconds, duration_minutes * 60) ELSE NULL END) as avg_duration
            FROM pomodoro_sessions 
            WHERE 1=1
            "#,
        );

        let mut bind_params: Vec<String> = Vec::new();

        if let Some(start) = &start_date {
            query.push_str(" AND DATE(start_time) >= ?");
            bind_params.push(start.clone());
        }

        if let Some(end) = &end_date {
            query.push_str(" AND DATE(start_time) <= ?");
            bind_params.push(end.clone());
        }

        let mut sql_query = sqlx::query(&query);
        for param in &bind_params {
            sql_query = sql_query.bind(param);
        }

        let row = sql_query.fetch_one(&self.pool).await?;

        // Get sessions by date
        let mut date_query = String::from(
            r#"
            SELECT 
                DATE(start_time) as date,
                SUM(CASE WHEN session_type = 'work' THEN 1 ELSE 0 END) as work_sessions,
                SUM(CASE WHEN session_type = 'break' THEN 1 ELSE 0 END) as break_sessions,
                SUM(CASE WHEN session_type = 'work' AND completed = 1 THEN COALESCE(actual_duration_seconds, duration_minutes * 60) ELSE 0 END) as work_time,
                SUM(CASE WHEN session_type = 'break' AND completed = 1 THEN COALESCE(actual_duration_seconds, duration_minutes * 60) ELSE 0 END) as break_time
            FROM pomodoro_sessions 
            WHERE 1=1
            "#,
        );

        if start_date.is_some() {
            date_query.push_str(" AND DATE(start_time) >= ?");
        }

        if end_date.is_some() {
            date_query.push_str(" AND DATE(start_time) <= ?");
        }

        date_query.push_str(" GROUP BY DATE(start_time) ORDER BY DATE(start_time)");

        let mut date_sql_query = sqlx::query(&date_query);
        for param in bind_params {
            date_sql_query = date_sql_query.bind(param);
        }

        let date_rows = date_sql_query.fetch_all(&self.pool).await?;

        let sessions_by_date = date_rows
            .into_iter()
            .map(|row| crate::models::PomodoroDateSummary {
                date: row.get("date"),
                work_sessions: row.get("work_sessions"),
                break_sessions: row.get("break_sessions"),
                total_work_time_seconds: row.get("work_time"),
                total_break_time_seconds: row.get("break_time"),
            })
            .collect();

        let summary = crate::models::PomodoroSummary {
            total_sessions: row.get("total_sessions"),
            completed_sessions: row.get("completed_sessions"),
            total_work_time_seconds: row.get("total_work_time"),
            total_break_time_seconds: row.get("total_break_time"),
            average_session_duration: row.get::<Option<f64>, _>("avg_duration").unwrap_or(0.0),
            sessions_by_date,
        };

        Ok(summary)
    }
}
