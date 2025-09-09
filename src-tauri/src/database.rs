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
        // Use connection options for SQLite with create_if_missing
        let pool = SqlitePool::connect(&format!("{}?mode=rwc", database_url)).await?;

        // Create tables if they don't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS activity_entries (
                id TEXT PRIMARY KEY,
                start_time TEXT NOT NULL,
                end_time TEXT,
                app_name TEXT NOT NULL,
                app_bundle_id TEXT,
                window_title TEXT NOT NULL,
                url TEXT,
                category TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create segments table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS activity_segments (
                id TEXT PRIMARY KEY,
                activity_id TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                segment_type TEXT NOT NULL,
                title TEXT NOT NULL,
                url TEXT,
                file_path TEXT,
                metadata TEXT,
                FOREIGN KEY (activity_id) REFERENCES activity_entries (id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create user categories table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                parent_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (parent_id) REFERENCES user_categories (id)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create app mappings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_mappings (
                id TEXT PRIMARY KEY,
                app_pattern TEXT NOT NULL,
                category_id TEXT NOT NULL,
                is_custom INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS url_mappings (
                id TEXT PRIMARY KEY,
                url_pattern TEXT NOT NULL,
                category_id TEXT NOT NULL,
                is_custom INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_start_time ON activity_entries(start_time);
            CREATE INDEX IF NOT EXISTS idx_app_name ON activity_entries(app_name);
            CREATE INDEX IF NOT EXISTS idx_category ON activity_entries(category);
            CREATE INDEX IF NOT EXISTS idx_segment_activity_id ON activity_segments(activity_id);
            CREATE INDEX IF NOT EXISTS idx_segment_start_time ON activity_segments(start_time);
            CREATE INDEX IF NOT EXISTS idx_segment_type ON activity_segments(segment_type);
            CREATE INDEX IF NOT EXISTS idx_user_categories_parent ON user_categories(parent_id);
            CREATE INDEX IF NOT EXISTS idx_app_mappings_pattern ON app_mappings(app_pattern);
            CREATE INDEX IF NOT EXISTS idx_app_mappings_category ON app_mappings(category_id);
            "#,
        )
        .execute(&pool)
        .await?;

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

    pub async fn delete_user_category(&self, id: &str) -> Result<(), sqlx::Error> {
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
}
