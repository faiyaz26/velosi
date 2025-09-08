use crate::models::{
    ActivityCategory, ActivityEntry, ActivitySegment, ActivitySummary, AppSummary, CategorySummary,
    DetailedActivity, SegmentSummary, SegmentType, TimelineActivity, TimelineData, TimelineSegment,
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

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_start_time ON activity_entries(start_time);
            CREATE INDEX IF NOT EXISTS idx_app_name ON activity_entries(app_name);
            CREATE INDEX IF NOT EXISTS idx_category ON activity_entries(category);
            CREATE INDEX IF NOT EXISTS idx_segment_activity_id ON activity_segments(activity_id);
            CREATE INDEX IF NOT EXISTS idx_segment_start_time ON activity_segments(start_time);
            CREATE INDEX IF NOT EXISTS idx_segment_type ON activity_segments(segment_type);
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

    // Keep old method for backward compatibility but update it
    pub async fn insert_activity(&self, entry: &ActivityEntry) -> Result<(), sqlx::Error> {
        self.start_activity(entry).await
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
}
