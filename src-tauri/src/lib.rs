use rusqlite::Connection;
use serde::Serialize;
use std::fs;
use std::sync::Mutex;
use sysinfo::System;
use tauri::{Manager, State};

mod ai_provider;
mod db;
mod encoder;
mod media;
mod model_manager;
mod runtime;
mod transcriber;
mod diagnostics;

use db::Project;
use media::MediaInfo;
use transcriber::TranscriberState;

#[derive(Serialize)]
struct SystemStats {
    cpu: f32,
    ram: u64,
}

struct SysInfoState(Mutex<System>);
struct DbState(Mutex<Connection>);

#[tauri::command]
fn get_system_stats(state: State<'_, SysInfoState>) -> SystemStats {
    let mut sys = state.0.lock().unwrap();
    sys.refresh_cpu();
    sys.refresh_memory();

    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let ram_usage = sys.used_memory() / 1024 / 1024;

    SystemStats {
        cpu: (cpu_usage * 10.0).round() / 10.0,
        ram: ram_usage,
    }
}

#[tauri::command]
fn get_recent_projects(state: State<'_, DbState>) -> Result<Vec<Project>, String> {
    let conn = state.0.lock().unwrap();
    db::get_projects(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_project(state: State<'_, DbState>, project: Project) -> Result<Project, String> {
    let conn = state.0.lock().unwrap();
    db::create_project(&conn, project).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_project(state: State<'_, DbState>, project: Project) -> Result<Project, String> {
    let conn = state.0.lock().unwrap();
    db::update_project(&conn, project).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_project(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::delete_project(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
fn duplicate_project(state: State<'_, DbState>, id: String) -> Result<Project, String> {
    let conn = state.0.lock().unwrap();
    db::duplicate_project(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_dictionary(state: State<'_, DbState>) -> Result<Vec<db::DictionaryEntry>, String> {
    let conn = state.0.lock().unwrap();
    db::get_dictionary(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn add_dictionary_entry(
    state: State<'_, DbState>,
    entry: db::DictionaryEntry,
) -> Result<db::DictionaryEntry, String> {
    let conn = state.0.lock().unwrap();
    db::add_dictionary_entry(&conn, entry).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_dictionary_entry(
    state: State<'_, DbState>,
    entry: db::DictionaryEntry,
) -> Result<db::DictionaryEntry, String> {
    let conn = state.0.lock().unwrap();
    db::update_dictionary_entry(&conn, entry).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_dictionary_entry(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::delete_dictionary_entry(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_ass_styles(state: State<'_, DbState>) -> Result<Vec<db::AssStyle>, String> {
    let conn = state.0.lock().unwrap();
    db::get_ass_styles(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_ass_style(state: State<'_, DbState>, style: db::AssStyle) -> Result<db::AssStyle, String> {
    let conn = state.0.lock().unwrap();
    db::save_ass_style(&conn, style).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_ass_style(state: State<'_, DbState>, name: String) -> Result<(), String> {
    let conn = state.0.lock().unwrap();
    db::delete_ass_style(&conn, &name).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_system_fonts() -> Result<Vec<String>, String> {
    let output = std::process::Command::new("reg")
        .args(&["query", r#"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Fonts"#])
        .output()
        .map_err(|e| e.to_string())?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut fonts = Vec::new();

    for line in output_str.lines() {
        let line = line.trim();
        if line.is_empty() || !line.contains("REG_SZ") {
            continue;
        }

        if let Some(idx) = line.find(" (TrueType)") {
            fonts.push(line[..idx].trim().to_string());
        } else if let Some(idx) = line.find(" (OpenType)") {
            fonts.push(line[..idx].trim().to_string());
        } else if let Some(idx) = line.find("    REG_SZ") {
            fonts.push(line[..idx].trim().to_string());
        } else if let Some(idx) = line.find("\tREG_SZ") {
            fonts.push(line[..idx].trim().to_string());
        }
    }

    fonts.sort();
    fonts.dedup();
    Ok(fonts)
}

#[tauri::command]
async fn analyze_media(path: String) -> Result<MediaInfo, String> {
    media::analyze_media(&path)
}

#[tauri::command]
async fn generate_thumbnail(
    app: tauri::AppHandle,
    path: String,
    project_id: String,
) -> Result<String, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let thumb_dir = app_dir.join("thumbnails");
    fs::create_dir_all(&thumb_dir).unwrap();

    let output_path = thumb_dir.join(format!("{}.jpg", project_id));
    media::generate_thumbnail(&path, output_path.to_str().unwrap())
}

#[tauri::command]
async fn generate_waveform(
    app: tauri::AppHandle,
    path: String,
    project_id: String,
) -> Result<String, String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let wave_dir = app_dir.join("waveforms");
    fs::create_dir_all(&wave_dir).unwrap();

    let output_path = wave_dir.join(format!("{}.png", project_id));
    media::generate_waveform(&path, output_path.to_str().unwrap())
}

#[tauri::command]
fn start_transcription_cmd(
    app: tauri::AppHandle,
    state: State<'_, TranscriberState>,
    project_id: String,
    input_path: String,
    duration: f64,
    model: String,
    language: String,
    device: String,
    format: String,
    custom_output_prefix: Option<String>,
) -> Result<(), String> {
    transcriber::start_transcription(
        app, state, project_id, input_path, duration, model, language, device, format, custom_output_prefix
    )
}

#[tauri::command]
fn cancel_transcription_cmd(state: State<'_, TranscriberState>) -> Result<(), String> {
    transcriber::cancel_transcription(state)
}

#[tauri::command]
fn start_encoding_cmd(app: tauri::AppHandle, config: encoder::EncodeConfig) -> Result<(), String> {
    encoder::start_encoding(app, config)
}

#[tauri::command]
fn cancel_encoding_cmd() -> Result<(), String> {
    encoder::cancel_encoding()
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if let Some(parent) = p.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_file(path: String) -> Result<(), String> {
    fs::remove_file(path).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_directory_files(path: String) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    files.push(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    Ok(files)
}

#[tauri::command]
fn scan_directory(path: String) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_file() {
                    if let Some(ext) = entry.path().extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if ext_str == "mp4"
                            || ext_str == "mkv"
                            || ext_str == "avi"
                            || ext_str == "mov"
                            || ext_str == "ts"
                        {
                            files.push(entry.path().to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(files)
}

#[tauri::command]
async fn llm_request(
    url: String,
    headers: std::collections::HashMap<String, String>,
    body: String,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut builder = client.post(&url);

    for (k, v) in headers {
        builder = builder.header(k, v);
    }

    let res = builder
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let text = res.text().await.map_err(|e| e.to_string())?;
    Ok(text)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut sys = System::new_all();
    sys.refresh_all();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::current_dir().unwrap());
            fs::create_dir_all(&app_dir).unwrap();

            let dirs = [
                "models", "cache", "projects", "logs", "temp", "database", "bin",
            ];
            for dir in dirs.iter() {
                fs::create_dir_all(app_dir.join(dir)).unwrap();
            }

            let db_dir = app_dir.join("database");
            let conn = db::init_db(&db_dir).expect("Failed to initialize database");

            app.manage(DbState(Mutex::new(conn)));
            app.manage(TranscriberState::default());

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .manage(SysInfoState(Mutex::new(sys)))
        .invoke_handler(tauri::generate_handler![
            get_system_stats,
            get_recent_projects,
            create_project,
            update_project,
            delete_project,
            duplicate_project,
            get_dictionary,
            add_dictionary_entry,
            update_dictionary_entry,
            delete_dictionary_entry,
            analyze_media,
            generate_thumbnail,
            generate_waveform,
            start_transcription_cmd,
            cancel_transcription_cmd,
            start_encoding_cmd,
            cancel_encoding_cmd,
            read_text_file,
            write_text_file,
            delete_file,
            scan_directory,
            list_directory_files,
            llm_request,
            runtime::check_runtime,
            runtime::repair_runtime,
            runtime::check_cuda,
            model_manager::get_available_models,
            model_manager::start_model_download,
            model_manager::pause_model_download,
            model_manager::resume_model_download,
            model_manager::cancel_model_download,
            model_manager::delete_model,
            ai_provider::get_providers,
            ai_provider::get_api_key,
            ai_provider::save_provider,
            ai_provider::test_connection,
            get_ass_styles,
            save_ass_style,
            delete_ass_style,
            get_system_fonts,
            diagnostics::run_diagnostics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
