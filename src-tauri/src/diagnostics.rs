use serde::Serialize;
use std::fs;
use std::process::Command;
use sysinfo::{System, Disks};
use tauri::{AppHandle, Manager};
use reqwest;

#[derive(Serialize)]
pub struct DiagnosticItem {
    pub name: String,
    pub status: String,
    pub version: Option<String>,
    pub path: Option<String>,
    pub health: String,
    pub error_msg: Option<String>,
    pub fix_suggestion: Option<String>,
}

#[tauri::command]
pub async fn run_diagnostics(app: AppHandle) -> Result<Vec<DiagnosticItem>, String> {
    let mut results = Vec::new();
    
    results.push(DiagnosticItem {
        name: "Rust Backend".to_string(),
        status: "OK".to_string(),
        version: Some(env!("CARGO_PKG_VERSION").to_string()),
        path: None,
        health: "healthy".to_string(),
        error_msg: None,
        fix_suggestion: None,
    });
    
    let db_health = match app.try_state::<crate::DbState>() {
        Some(state) => {
            let conn = state.0.lock().unwrap();
            match conn.query_row("SELECT sqlite_version()", [], |row| row.get::<_, String>(0)) {
                Ok(ver) => DiagnosticItem {
                    name: "SQLite".to_string(),
                    status: "OK".to_string(),
                    version: Some(ver),
                    path: None,
                    health: "healthy".to_string(),
                    error_msg: None,
                    fix_suggestion: None,
                },
                Err(e) => DiagnosticItem {
                    name: "SQLite".to_string(),
                    status: "ERROR".to_string(),
                    version: None,
                    path: None,
                    health: "error".to_string(),
                    error_msg: Some(e.to_string()),
                    fix_suggestion: Some("Check database file permissions or recreate database.".to_string()),
                }
            }
        },
        None => DiagnosticItem {
            name: "SQLite".to_string(),
            status: "ERROR".to_string(),
            version: None,
            path: None,
            health: "error".to_string(),
            error_msg: Some("DbState not found".to_string()),
            fix_suggestion: Some("Restart the application.".to_string()),
        }
    };
    results.push(db_health);
    
    let ffmpeg_res = match Command::new("ffmpeg").arg("-version").output() {
        Ok(out) => {
            let out_str = String::from_utf8_lossy(&out.stdout);
            let version = out_str.lines().next().unwrap_or("Unknown").to_string();
            DiagnosticItem {
                name: "FFmpeg".to_string(),
                status: "OK".to_string(),
                version: Some(version),
                path: Some("ffmpeg (from PATH)".to_string()),
                health: "healthy".to_string(),
                error_msg: None,
                fix_suggestion: None,
            }
        },
        Err(e) => DiagnosticItem {
            name: "FFmpeg".to_string(),
            status: "ERROR".to_string(),
            version: None,
            path: None,
            health: "error".to_string(),
            error_msg: Some(e.to_string()),
            fix_suggestion: Some("Install FFmpeg and add it to system PATH.".to_string()),
        }
    };
    results.push(ffmpeg_res);
    
    let ffprobe_res = match Command::new("ffprobe").arg("-version").output() {
        Ok(out) => {
            let out_str = String::from_utf8_lossy(&out.stdout);
            let version = out_str.lines().next().unwrap_or("Unknown").to_string();
            DiagnosticItem {
                name: "FFprobe".to_string(),
                status: "OK".to_string(),
                version: Some(version),
                path: Some("ffprobe (from PATH)".to_string()),
                health: "healthy".to_string(),
                error_msg: None,
                fix_suggestion: None,
            }
        },
        Err(e) => DiagnosticItem {
            name: "FFprobe".to_string(),
            status: "ERROR".to_string(),
            version: None,
            path: None,
            health: "error".to_string(),
            error_msg: Some(e.to_string()),
            fix_suggestion: Some("Install FFprobe and add it to system PATH.".to_string()),
        }
    };
    results.push(ffprobe_res);
    
    let runtime_status = crate::runtime::check_runtime(app.clone());
    if runtime_status.models_ok {
        results.push(DiagnosticItem {
            name: "Whisper Installation".to_string(),
            status: "OK".to_string(),
            version: None,
            path: None,
            health: "healthy".to_string(),
            error_msg: None,
            fix_suggestion: None,
        });
    } else {
        results.push(DiagnosticItem {
            name: "Whisper Installation".to_string(),
            status: "WARNING".to_string(),
            version: None,
            path: None,
            health: "warning".to_string(),
            error_msg: Some("Whisper models not found.".to_string()),
            fix_suggestion: Some("Go to Setup or Models page to download a Whisper model.".to_string()),
        });
    }
    
    let internet = match reqwest::get("https://1.1.1.1").await {
        Ok(_) => DiagnosticItem {
            name: "Internet Connection".to_string(),
            status: "OK".to_string(),
            version: None,
            path: None,
            health: "healthy".to_string(),
            error_msg: None,
            fix_suggestion: None,
        },
        Err(e) => DiagnosticItem {
            name: "Internet Connection".to_string(),
            status: "ERROR".to_string(),
            version: None,
            path: None,
            health: "error".to_string(),
            error_msg: Some(e.to_string()),
            fix_suggestion: Some("Check your internet connection or proxy settings.".to_string()),
        }
    };
    results.push(internet);
    
    let app_dir = app.path().app_data_dir().unwrap_or_else(|_| std::env::current_dir().unwrap());
    let test_file = app_dir.join(".diagnostic_test");
    let write_res = match fs::write(&test_file, b"test") {
        Ok(_) => DiagnosticItem {
            name: "Write Permission".to_string(),
            status: "OK".to_string(),
            version: None,
            path: Some(app_dir.to_string_lossy().to_string()),
            health: "healthy".to_string(),
            error_msg: None,
            fix_suggestion: None,
        },
        Err(e) => DiagnosticItem {
            name: "Write Permission".to_string(),
            status: "ERROR".to_string(),
            version: None,
            path: Some(app_dir.to_string_lossy().to_string()),
            health: "error".to_string(),
            error_msg: Some(e.to_string()),
            fix_suggestion: Some("Run application as administrator or fix directory permissions.".to_string()),
        }
    };
    results.push(write_res);
    
    let read_res = match fs::read(&test_file) {
        Ok(_) => {
            let _ = fs::remove_file(&test_file);
            DiagnosticItem {
                name: "Read Permission".to_string(),
                status: "OK".to_string(),
                version: None,
                path: Some(app_dir.to_string_lossy().to_string()),
                health: "healthy".to_string(),
                error_msg: None,
                fix_suggestion: None,
            }
        },
        Err(e) => DiagnosticItem {
            name: "Read Permission".to_string(),
            status: "ERROR".to_string(),
            version: None,
            path: Some(app_dir.to_string_lossy().to_string()),
            health: "error".to_string(),
            error_msg: Some(e.to_string()),
            fix_suggestion: Some("Check directory permissions.".to_string()),
        }
    };
    results.push(read_res);
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let total_ram = sys.total_memory() / 1024 / 1024;
    let avail_ram = sys.available_memory() / 1024 / 1024;
    results.push(DiagnosticItem {
        name: "Available RAM".to_string(),
        status: if avail_ram > 1024 { "OK".to_string() } else { "WARNING".to_string() },
        version: Some(format!("{} MB free of {} MB", avail_ram, total_ram)),
        path: None,
        health: if avail_ram > 1024 { "healthy".to_string() } else { "warning".to_string() },
        error_msg: if avail_ram > 1024 { None } else { Some(format!("Only {} MB available", avail_ram)) },
        fix_suggestion: if avail_ram > 1024 { None } else { Some("Close other applications to free up RAM.".to_string()) },
    });
    
    let cpu = sys.cpus().first().map(|c| c.brand()).unwrap_or("Unknown CPU");
    results.push(DiagnosticItem {
        name: "CPU Information".to_string(),
        status: "OK".to_string(),
        version: Some(cpu.to_string()),
        path: None,
        health: "healthy".to_string(),
        error_msg: None,
        fix_suggestion: None,
    });
    
    let disks = Disks::new_with_refreshed_list();
    let mut disk_space = "Unknown".to_string();
    let mut health = "healthy";
    let mut error = None;
    let mut suggestion = None;
    
    let c_drive = disks.iter().find(|d| d.mount_point().to_string_lossy().to_uppercase().starts_with("C:"));
    
    if let Some(disk) = c_drive.or_else(|| disks.first()) {
        let avail_gb = disk.available_space() / 1024 / 1024 / 1024;
        let total_gb = disk.total_space() / 1024 / 1024 / 1024;
        disk_space = format!("{} GB free of {} GB (C: Drive)", avail_gb, total_gb);
        if avail_gb < 10 {
            health = "warning";
            error = Some("Low disk space on C: Drive".to_string());
            suggestion = Some("Free up space on C: for downloading models.".to_string());
        }
    }
    
    results.push(DiagnosticItem {
        name: "Disk Space".to_string(),
        status: if health == "warning" { "WARNING".to_string() } else { "OK".to_string() },
        version: Some(disk_space),
        path: None,
        health: health.to_string(),
        error_msg: error,
        fix_suggestion: suggestion,
    });

    Ok(results)
}
