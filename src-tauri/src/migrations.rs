use serde_json;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Migration {
    pub version: i32,
    pub description: String,
    pub sql: String,
}

/// Returns the list of SQL migrations to apply for the SQLite database.
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create_initial_tables".to_string(),
            sql: include_str!("../migrations/1_create_tables.sql").to_string(),
        },
        Migration {
            version: 2,
            description: "create_focus_mode_preferences".to_string(),
            sql: include_str!("../migrations/2_create_focus_mode_preferences.sql").to_string(),
        },
        Migration {
            version: 3,
            description: "add_blocking_preferences".to_string(),
            sql: include_str!("../migrations/3_add_blocking_preferences.sql").to_string(),
        },
        Migration {
            version: 4,
            description: "add_proxy_port".to_string(),
            sql: include_str!("../migrations/4_add_proxy_port.sql").to_string(),
        },
        Migration {
            version: 5,
            description: "create_pomodoro_tables".to_string(),
            sql: include_str!("../migrations/5_create_pomodoro_tables.sql").to_string(),
        },
    ]
}

/// Apply migrations to the database
pub async fn apply_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create migrations table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Get already applied migrations
    let applied_migrations: HashMap<i32, String> =
        sqlx::query("SELECT version, applied_at FROM schema_migrations ORDER BY version")
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(|row| {
                let version: i32 = row.get("version");
                let applied_at: String = row.get("applied_at");
                (version, applied_at)
            })
            .collect();

    let migrations = get_migrations();

    println!("🔄 Checking migrations...");
    println!("Found {} migration(s) to check", migrations.len());

    for migration in migrations {
        if applied_migrations.contains_key(&migration.version) {
            println!(
                "✅ Migration {} ({}) already applied",
                migration.version, migration.description
            );
            continue;
        }

        println!(
            "🔧 Applying migration {} ({})",
            migration.version, migration.description
        );

        // Apply the migration
        sqlx::query(&migration.sql)
            .execute(pool)
            .await
            .map_err(|e| {
                eprintln!("❌ Failed to apply migration {}: {}", migration.version, e);
                e
            })?;

        // Record the migration as applied
        sqlx::query(
            "INSERT INTO schema_migrations (version, description, applied_at) VALUES (?, ?, ?)",
        )
        .bind(migration.version)
        .bind(&migration.description)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(pool)
        .await?;

        println!("✅ Migration {} applied successfully", migration.version);
    }

    println!("🎉 All migrations applied successfully");

    // After migrations, load initial data
    load_initial_data(pool).await?;

    Ok(())
}

/// Load initial data from JSON files into the database
pub async fn load_initial_data(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("🔄 Loading initial data...");

    // Check if data has already been loaded
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS data_seeds (
            id TEXT PRIMARY KEY,
            loaded_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    let existing_seed = sqlx::query("SELECT id FROM data_seeds WHERE id = 'initial_data'")
        .fetch_optional(pool)
        .await?;

    if existing_seed.is_some() {
        println!("✅ Initial data already loaded");
        return Ok(());
    }

    // Load categories
    load_categories(pool).await?;

    // Load app mappings
    load_app_mappings(pool).await?;

    // Load URL mappings
    load_url_mappings(pool).await?;

    // Mark data as loaded
    sqlx::query("INSERT INTO data_seeds (id, loaded_at) VALUES (?, ?)")
        .bind("initial_data")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(pool)
        .await?;

    println!("🎉 Initial data loaded successfully");
    Ok(())
}

async fn load_categories(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("📂 Loading categories...");

    let categories_json = include_str!("../../data/categories.json");
    let categories_data: serde_json::Value =
        serde_json::from_str(categories_json).map_err(|e| {
            sqlx::Error::Decode(format!("Failed to parse categories.json: {}", e).into())
        })?;

    if let Some(categories) = categories_data.get("categories").and_then(|c| c.as_array()) {
        for category in categories {
            if let (Some(id), Some(name), Some(color)) = (
                category.get("id").and_then(|v| v.as_str()),
                category.get("name").and_then(|v| v.as_str()),
                category.get("color").and_then(|v| v.as_str()),
            ) {
                let parent_id = category.get("parent_id").and_then(|v| v.as_str());
                let now = chrono::Utc::now().to_rfc3339();

                sqlx::query(
                    "INSERT OR IGNORE INTO user_categories (id, name, color, parent_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
                )
                .bind(id)
                .bind(name)
                .bind(color)
                .bind(parent_id)
                .bind(&now)
                .bind(&now)
                .execute(pool)
                .await?;
            }
        }
        println!("✅ Categories loaded");
    }

    Ok(())
}

async fn load_app_mappings(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("📱 Loading app mappings...");

    let app_mappings_json = include_str!("../../data/app-mappings.json");
    let app_mappings_data: serde_json::Value =
        serde_json::from_str(app_mappings_json).map_err(|e| {
            sqlx::Error::Decode(format!("Failed to parse app-mappings.json: {}", e).into())
        })?;

    if let Some(mappings) = app_mappings_data.get("mappings").and_then(|m| m.as_array()) {
        for mapping in mappings {
            if let (Some(category), Some(apps)) = (
                mapping.get("category").and_then(|v| v.as_str()),
                mapping.get("apps").and_then(|v| v.as_array()),
            ) {
                for app in apps {
                    if let Some(app_pattern) = app.as_str() {
                        let id = uuid::Uuid::new_v4().to_string();
                        let now = chrono::Utc::now().to_rfc3339();

                        sqlx::query(
                            "INSERT OR IGNORE INTO app_mappings (id, app_pattern, category_id, is_custom, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
                        )
                        .bind(&id)
                        .bind(app_pattern)
                        .bind(category)
                        .bind(false) // Built-in mappings are not custom
                        .bind(&now)
                        .bind(&now)
                        .execute(pool)
                        .await?;
                    }
                }
            }
        }
        println!("✅ App mappings loaded");
    }

    Ok(())
}

async fn load_url_mappings(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    println!("🔗 Loading URL mappings...");

    // Use include_str! to embed the file at compile time instead of runtime file reading
    let url_mappings_json = include_str!("../../data/url-mappings.json");
    let url_mappings_data: serde_json::Value =
        serde_json::from_str(url_mappings_json).map_err(|e| {
            sqlx::Error::Decode(format!("Failed to parse url-mappings.json: {}", e).into())
        })?;

    if let Some(mappings) = url_mappings_data.get("mappings").and_then(|m| m.as_array()) {
        for mapping in mappings {
            if let (Some(category), Some(urls)) = (
                mapping.get("category").and_then(|v| v.as_str()),
                mapping.get("urls").and_then(|v| v.as_array()),
            ) {
                for url in urls {
                    if let Some(url_pattern) = url.as_str() {
                        let id = uuid::Uuid::new_v4().to_string();
                        let now = chrono::Utc::now().to_rfc3339();

                        sqlx::query(
                            "INSERT OR IGNORE INTO url_mappings (id, url_pattern, category_id, is_custom, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)"
                        )
                        .bind(&id)
                        .bind(url_pattern)
                        .bind(category)
                        .bind(false) // Built-in mappings are not custom
                        .bind(&now)
                        .bind(&now)
                        .execute(pool)
                        .await?;
                    }
                }
            }
        }
        println!("✅ URL mappings loaded");
    }

    Ok(())
}
