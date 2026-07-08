use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

lazy_static::lazy_static! {
    static ref DOWNLOADS: Arc<Mutex<HashMap<String, DownloadState>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AIModel {
    pub id: String,
    pub name: String,
    pub size_mb: f64,
    pub is_installed: bool,
    pub disk_usage_mb: f64,
    pub download_url: String,
}

#[derive(Clone, Serialize)]
pub struct DownloadProgressEvent {
    pub id: String,
    pub percent: f64,
    pub speed_mbps: f64,
    pub eta_seconds: f64,
    pub status: String,
}

#[derive(Clone)]
enum DownloadState {
    Downloading,
    Paused,
    Cancelled,
}

pub fn get_hardcoded_models() -> Vec<AIModel> {
    vec![
        AIModel {
            id: "tiny".into(),
            name: "Tiny".into(),
            size_mb: 78.0,
            is_installed: false,
            disk_usage_mb: 0.0,
            download_url:
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin".into(),
        },
        AIModel {
            id: "base".into(),
            name: "Base".into(),
            size_mb: 148.0,
            is_installed: false,
            disk_usage_mb: 0.0,
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin".into(),
        },
        AIModel {
            id: "small".into(),
            name: "Small".into(),
            size_mb: 488.0,
            is_installed: false,
            disk_usage_mb: 0.0,
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin".into(),
        },
        AIModel {
            id: "medium".into(),
            name: "Medium".into(),
            size_mb: 1530.0,
            is_installed: false,
            disk_usage_mb: 0.0,
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin".into(),
        },
        AIModel {
            id: "large-v3".into(),
            name: "Large V3".into(),
            size_mb: 3090.0,
            is_installed: false,
            disk_usage_mb: 0.0,
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin".into(),
        },
        AIModel {
            id: "large-v3-turbo".into(),
            name: "Large V3 Turbo".into(),
            size_mb: 1625.0,
            is_installed: false,
            disk_usage_mb: 0.0,
            download_url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin".into(),
        },
    ]
}

#[tauri::command]
pub fn get_available_models(app: AppHandle) -> Vec<AIModel> {
    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let models_dir = app_dir.join("models");

    let mut models = get_hardcoded_models();
    for m in models.iter_mut() {
        let model_path = models_dir.join(format!("ggml-{}.bin", m.id));
        if model_path.exists() {
            m.is_installed = true;
            if let Ok(meta) = fs::metadata(&model_path) {
                m.disk_usage_mb = (meta.len() as f64) / 1024.0 / 1024.0;
            }
        }
    }

    models
}

#[tauri::command]
pub async fn start_model_download(app: AppHandle, id: String, url: String) -> Result<(), String> {
    {
        let mut downloads = DOWNLOADS.lock().await;
        downloads.insert(id.clone(), DownloadState::Downloading);
    }

    let app_clone = app.clone();
    let id_clone = id.clone();

    tokio::spawn(async move {
        let app_dir = app_clone
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        let models_dir = app_dir.join("models");
        let _ = fs::create_dir_all(&models_dir);
        let bin_path = models_dir.join(format!("ggml-{}.bin", id_clone));
        let temp_path = models_dir.join(format!("ggml-{}.bin.download", id_clone));

        let client = reqwest::Client::new();
        let mut res = match client.get(&url).send().await {
            Ok(r) => r,
            Err(_e) => {
                let mut d = DOWNLOADS.lock().await;
                d.remove(&id_clone);
                return;
            }
        };

        let total_bytes = res.content_length().unwrap_or(0) as f64;
        let mut downloaded_bytes: f64 = 0.0;

        let mut file = match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
        {
            Ok(f) => f,
            Err(_) => return,
        };

        let start_time = std::time::Instant::now();
        let mut last_emit = std::time::Instant::now();

        while let Ok(Some(chunk)) = res.chunk().await {
            let state = {
                let d = DOWNLOADS.lock().await;
                d.get(&id_clone).cloned()
            };

            match state {
                Some(DownloadState::Paused) => {
                    let _ = app_clone.emit("model-download-progress", DownloadProgressEvent {
                        id: id_clone.clone(),
                        percent: (downloaded_bytes / total_bytes) * 100.0,
                        speed_mbps: 0.0,
                        eta_seconds: 0.0,
                        status: "paused".into(),
                    });
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
                Some(DownloadState::Cancelled) => {
                    let _ = fs::remove_file(&temp_path);
                    let mut d = DOWNLOADS.lock().await;
                    d.remove(&id_clone);
                    return;
                }
                _ => {}
            }

            if let Err(_) = file.write_all(&chunk) {
                break;
            }

            downloaded_bytes += chunk.len() as f64;

            if last_emit.elapsed().as_millis() > 500 {
                let elapsed_secs = start_time.elapsed().as_secs_f64();
                let speed = if elapsed_secs > 0.0 {
                    (downloaded_bytes / 1_048_576.0) / elapsed_secs
                } else {
                    0.0
                };

                let percent = if total_bytes > 0.0 {
                    (downloaded_bytes / total_bytes) * 100.0
                } else {
                    0.0
                };
                
                let remaining = total_bytes - downloaded_bytes;
                let eta = if speed > 0.0 {
                    (remaining / 1_048_576.0) / speed
                } else {
                    0.0
                };

                let _ = app_clone.emit("model-download-progress", DownloadProgressEvent {
                    id: id_clone.clone(),
                    percent,
                    speed_mbps: speed,
                    eta_seconds: eta,
                    status: "downloading".into(),
                });
                last_emit = std::time::Instant::now();
            }
        }
        
        let _ = fs::rename(&temp_path, &bin_path);
        let mut d = DOWNLOADS.lock().await;
        d.remove(&id_clone);

        let _ = app_clone.emit(
            "model-download-progress",
            DownloadProgressEvent {
                id: id_clone.clone(),
                percent: 100.0,
                speed_mbps: 0.0,
                eta_seconds: 0.0,
                status: "completed".into(),
            },
        );
    });

    Ok(())
}

#[tauri::command]
pub async fn pause_model_download(id: String) -> Result<(), String> {
    let mut downloads = DOWNLOADS.lock().await;
    if let Some(state) = downloads.get_mut(&id) {
        *state = DownloadState::Paused;
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_model_download(id: String) -> Result<(), String> {
    let mut downloads = DOWNLOADS.lock().await;
    if let Some(state) = downloads.get_mut(&id) {
        *state = DownloadState::Downloading;
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel_model_download(id: String) -> Result<(), String> {
    let mut downloads = DOWNLOADS.lock().await;
    if let Some(state) = downloads.get_mut(&id) {
        *state = DownloadState::Cancelled;
    }
    Ok(())
}

#[tauri::command]
pub fn delete_model(app: AppHandle, id: String) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let target_file = app_dir.join("models").join(format!("ggml-{}.bin", id));
    fs::remove_file(target_file).map_err(|e| e.to_string())
}
