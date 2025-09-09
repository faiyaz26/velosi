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
        // Add more migrations here as needed
        // Migration {
        //     version: 2,
        //     description: "add_new_column".to_string(),
        //     sql: include_str!("../migrations/2_add_new_column.sql").to_string(),
        // },
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
    Ok(())
}
