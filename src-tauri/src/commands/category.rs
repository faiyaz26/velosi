use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::models::UserCategory;
use crate::AppState;

#[tauri::command]
pub async fn get_categories(state: State<'_, AppState>) -> Result<Vec<UserCategory>, String> {
    state
        .db
        .get_user_categories()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_categories(state: State<'_, AppState>) -> Result<Vec<UserCategory>, String> {
    state
        .db
        .get_user_categories()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_category(
    state: State<'_, AppState>,
    name: String,
    color: String,
) -> Result<UserCategory, String> {
    let now = Utc::now();
    let category = UserCategory {
        id: Uuid::new_v4().to_string(),
        name,
        color,
        parent_id: None,
        created_at: now,
        updated_at: now,
    };

    state
        .db
        .add_user_category(&category)
        .await
        .map_err(|e| e.to_string())?;

    Ok(category)
}

#[tauri::command]
pub async fn update_category(
    state: State<'_, AppState>,
    id: String,
    name: String,
    color: String,
) -> Result<(), String> {
    let category = UserCategory {
        id: id.clone(),
        name,
        color,
        parent_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    state
        .db
        .update_user_category(&category)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_category(state: State<'_, AppState>, id: String) -> Result<(), String> {
    // Check if this is the "Unknown" category
    if let Ok(Some(category)) = state.db.get_user_category_by_id(&id).await {
        if category.name.to_lowercase() == "unknown" {
            return Err(
                "Cannot delete the 'Unknown' category as it is required for the system."
                    .to_string(),
            );
        }
    }

    state
        .db
        .delete_user_category(&id)
        .await
        .map_err(|e| format!("Failed to delete category: {}", e))
}