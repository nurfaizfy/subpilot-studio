use serde::{Deserialize, Serialize};
use std::time::Instant;
use std::path::PathBuf;
use std::fs;
use tauri::{State, Manager};

use crate::DbState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub api_url: String,
    pub default_model: String,
    pub is_fallback: bool,
    pub priority: i32,
    pub has_key: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TestResult {
    pub success: bool,
    pub latency_ms: u64,
    pub message: String,
}

fn get_keys_file_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let app_dir = app_handle.path().app_data_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
    fs::create_dir_all(&app_dir).unwrap_or(());
    app_dir.join("api_keys.json")
}

fn get_api_key_from_file(app_handle: &tauri::AppHandle, provider_id: &str) -> Option<String> {
    let path = get_keys_file_path(app_handle);
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(key) = json.get(provider_id) {
                if let Some(key_str) = key.as_str() {
                    return Some(key_str.to_string());
                }
            }
        }
    }
    None
}

fn save_api_key_to_file(app_handle: &tauri::AppHandle, provider_id: &str, api_key: &str) -> Result<(), String> {
    let path = get_keys_file_path(app_handle);
    let mut json = if let Ok(content) = fs::read_to_string(&path) {
        serde_json::from_str::<serde_json::Value>(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if let Some(obj) = json.as_object_mut() {
        obj.insert(provider_id.to_string(), serde_json::json!(api_key));
    }

    fs::write(&path, serde_json::to_string_pretty(&json).unwrap()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_providers(app: tauri::AppHandle, state: State<'_, DbState>) -> Result<Vec<ProviderConfig>, String> {
    let conn = state.0.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, api_url, default_model, is_fallback, priority FROM ai_providers ORDER BY priority DESC")
        .map_err(|e| e.to_string())?;

    let provider_iter = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let is_fallback: i32 = row.get(4)?;

            let has_key = get_api_key_from_file(&app, &id).is_some();

            Ok(ProviderConfig {
                id,
                name: row.get(1)?,
                api_url: row.get(2)?,
                default_model: row.get(3)?,
                is_fallback: is_fallback == 1,
                priority: row.get(5)?,
                has_key,
            })
        })
        .map_err(|e| e.to_string())?;

    let providers = provider_iter
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(providers)
}

#[tauri::command]
pub fn get_api_key(app: tauri::AppHandle, provider_id: String) -> Result<String, String> {
    get_api_key_from_file(&app, &provider_id).ok_or_else(|| "API key not found".to_string())
}

#[tauri::command]
pub fn save_provider(
    app: tauri::AppHandle,
    state: State<'_, DbState>,
    config: ProviderConfig,
    api_key: Option<String>,
) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    let is_fallback = if config.is_fallback { 1 } else { 0 };

    conn.execute(
        "UPDATE ai_providers SET api_url = ?1, default_model = ?2, is_fallback = ?3 WHERE id = ?4",
        rusqlite::params![config.api_url, config.default_model, is_fallback, config.id],
    )
    .map_err(|e| e.to_string())?;

    if let Some(key) = api_key {
        if !key.is_empty() {
            save_api_key_to_file(&app, &config.id, &key)?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn test_connection(
    app: tauri::AppHandle,
    provider_id: String,
    state: State<'_, DbState>,
) -> Result<TestResult, String> {
    let config = {
        let conn = state.0.lock().unwrap();
        let mut stmt = conn.prepare("SELECT api_url, default_model FROM ai_providers WHERE id = ?1").map_err(|e| e.to_string())?;
        
        let row = stmt.query_row(rusqlite::params![provider_id], |row| {
            let url: String = row.get(0)?;
            let model: String = row.get(1)?;
            Ok((url, model))
        });

        match row {
            Ok(c) => c,
            Err(_) => return Ok(TestResult {
                success: false,
                latency_ms: 0,
                message: "Provider not found in database".to_string()
            })
        }
    };

    let api_url = config.0;
    let model = config.1;

    let api_key = match get_api_key_from_file(&app, &provider_id) {
        Some(pwd) => pwd,
        None => {
            return Ok(TestResult {
                success: false,
                latency_ms: 0,
                message: "No API Key configured".to_string(),
            })
        }
    };

    let start = Instant::now();
    let client = reqwest::Client::new();

    let mut builder = client.post(&api_url);
    builder = builder.header("Content-Type", "application/json");

    if provider_id == "gemini" {
        let url_with_key = format!(
            "{}?key={}",
            api_url.replace("/models", &format!("/models/{}:generateContent", model)),
            api_key
        );
        builder = client.post(&url_with_key);
        let body = serde_json::json!({
            "contents": [{"parts": [{"text": "ping"}]}]
        });
        builder = builder.body(body.to_string());
    } else {
        builder = builder.header("Authorization", format!("Bearer {}", api_key));
        let body = serde_json::json!({
            "model": model,
            "messages": [{"role": "user", "content": "ping"}],
            "max_tokens": 5
        });
        builder = builder.body(body.to_string());
    }

    match builder.send().await {
        Ok(res) => {
            let latency = start.elapsed().as_millis() as u64;
            if res.status().is_success() {
                Ok(TestResult {
                    success: true,
                    latency_ms: latency,
                    message: "Online".into(),
                })
            } else {
                let status = res.status();
                let err_text = res.text().await.unwrap_or_default();
                Ok(TestResult {
                    success: false,
                    latency_ms: latency,
                    message: format!("HTTP {}: {}", status, err_text),
                })
            }
        }
        Err(e) => Ok(TestResult {
            success: false,
            latency_ms: 0,
            message: format!("Network Error: {}", e),
        }),
    }
}
