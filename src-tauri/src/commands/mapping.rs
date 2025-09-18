use serde_json::Value;
use tauri::State;
use uuid::Uuid;

use crate::AppState;

#[tauri::command]
pub async fn get_app_mappings(state: State<'_, AppState>) -> Result<Value, String> {
    let mappings = state
        .db
        .get_app_mappings()
        .await
        .map_err(|e| e.to_string())?;

    // Get all categories to map category_id to category name
    let categories = state
        .db
        .get_user_categories()
        .await
        .map_err(|e| e.to_string())?;

    let category_map: std::collections::HashMap<String, String> = categories
        .into_iter()
        .map(|cat| (cat.id, cat.name))
        .collect();

    // Group mappings by category_id
    use std::collections::HashMap;
    let mut category_mappings: HashMap<String, Vec<String>> = HashMap::new();

    for mapping in mappings {
        category_mappings
            .entry(mapping.category_id)
            .or_insert_with(Vec::new)
            .push(mapping.app_pattern);
    }

    // Convert to the expected JSON format with category names
    let mappings_json: Vec<serde_json::Value> = category_mappings
        .into_iter()
        .map(|(category_id, apps)| {
            let category_name = category_map
                .get(&category_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            serde_json::json!({
                "category": category_name,
                "apps": apps
            })
        })
        .collect();

    Ok(serde_json::json!({
        "mappings": mappings_json
    }))
}

#[tauri::command]
pub async fn get_url_mappings(state: State<'_, AppState>) -> Result<Value, String> {
    let mappings = state
        .db
        .get_url_mappings()
        .await
        .map_err(|e| e.to_string())?;

    // Get all categories to map category_id to category name
    let categories = state
        .db
        .get_user_categories()
        .await
        .map_err(|e| e.to_string())?;

    let category_map: std::collections::HashMap<String, String> = categories
        .into_iter()
        .map(|cat| (cat.id, cat.name))
        .collect();

    // Group mappings by category_id
    use std::collections::HashMap;
    let mut category_mappings: HashMap<String, Vec<String>> = HashMap::new();

    for mapping in mappings {
        category_mappings
            .entry(mapping.category_id)
            .or_insert_with(Vec::new)
            .push(mapping.url_pattern);
    }

    // Convert to the expected JSON format with category names
    let mappings_json: Vec<serde_json::Value> = category_mappings
        .into_iter()
        .map(|(category_id, urls)| {
            let category_name = category_map
                .get(&category_id)
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());
            serde_json::json!({
                "category": category_name,
                "urls": urls
            })
        })
        .collect();

    Ok(serde_json::json!({
        "mappings": mappings_json
    }))
}

#[tauri::command]
pub async fn add_app_mapping(
    state: State<'_, AppState>,
    app_name: String,
    category_id: String,
) -> Result<(), String> {
    state
        .db
        .add_simple_app_mapping(&category_id, &app_name, true)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_app_mapping(
    state: State<'_, AppState>,
    app_name: String,
    category_id: String,
) -> Result<(), String> {
    // For update, we need to remove the old mapping and add a new one
    // since the app_name might have changed
    state
        .db
        .remove_app_mapping(&category_id, &app_name)
        .await
        .map_err(|e| e.to_string())?;

    state
        .db
        .add_simple_app_mapping(&category_id, &app_name, true)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_app_mapping(
    state: State<'_, AppState>,
    app_name: String,
) -> Result<(), String> {
    // We need to find the mapping first to get the category_id
    let mappings = state
        .db
        .get_app_mappings()
        .await
        .map_err(|e| e.to_string())?;

    for mapping in mappings {
        if mapping.app_pattern == app_name {
            return state
                .db
                .remove_app_mapping(&mapping.category_id, &app_name)
                .await
                .map_err(|e| e.to_string());
        }
    }

    Err("App mapping not found".to_string())
}

#[tauri::command]
pub async fn remove_app_mapping(
    state: State<'_, AppState>,
    app_name: String,
) -> Result<(), String> {
    delete_app_mapping(state, app_name).await
}

#[tauri::command]
pub async fn add_url_mapping(
    state: State<'_, AppState>,
    url_pattern: String,
    category_id: String,
) -> Result<(), String> {
    let mapping = crate::models::UrlMapping {
        id: Uuid::new_v4(),
        url_pattern,
        category_id,
        is_custom: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state
        .db
        .add_url_mapping(&mapping)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_url_mapping(
    state: State<'_, AppState>,
    url_pattern: String,
    category_id: String,
) -> Result<(), String> {
    state
        .db
        .remove_url_mapping(&category_id, &url_pattern)
        .await
        .map_err(|e| e.to_string())
}