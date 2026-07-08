use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone)]
pub struct RuntimeStatus {
    pub ffmpeg_ok: bool,
    pub ffprobe_ok: bool,
    pub models_ok: bool,
    pub whisper_ok: bool,
}

fn check_executable(app: &AppHandle, exe_name: &str, subfolder: &str) -> bool {
    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let bin_path = app_dir.join("bin").join(subfolder).join(exe_name);

    if bin_path.exists() {
        return true;
    }

    let version_arg = if exe_name.contains("python") {
        "--version"
    } else {
        "-version"
    };

    if let Ok(output) = Command::new(exe_name).arg(version_arg).output() {
        if output.status.success() {
            return true;
        }
    }

    false
}

#[tauri::command]
pub fn check_runtime(app: AppHandle) -> RuntimeStatus {
    let ffmpeg_ok = check_executable(&app, "ffmpeg.exe", "ffmpeg");
    let ffprobe_ok = check_executable(&app, "ffprobe.exe", "ffmpeg");
    let whisper_ok = check_executable(&app, "whisper-cli.exe", "whisper");

    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let models_dir = app_dir.join("models");
    let mut models_ok = false;

    if let Ok(entries) = fs::read_dir(models_dir) {
        for _entry in entries.flatten() {
            models_ok = true;
            break;
        }
    }

    RuntimeStatus {
        ffmpeg_ok,
        ffprobe_ok,
        models_ok,
        whisper_ok,
    }
}

#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    component: String,
    status: String,
}

#[tauri::command]
pub async fn repair_runtime(app: AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    
    let ffmpeg_ok = check_executable(&app, "ffmpeg.exe", "ffmpeg");
    let ffprobe_ok = check_executable(&app, "ffprobe.exe", "ffmpeg");
    let whisper_ok = check_executable(&app, "whisper-cli.exe", "whisper");

    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    
    let bin_dir = app_dir.join("bin");

    if !ffmpeg_ok || !ffprobe_ok {
        let ffmpeg_dir = bin_dir.join("ffmpeg");
        std::fs::create_dir_all(&ffmpeg_dir).map_err(|e| e.to_string())?;

        let ffmpeg_url = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip";
        let zip_path = app_dir.join("ffmpeg.zip");
        let extract_dir = app_dir.join("ffmpeg_extracted");

        app.emit("repair-progress", ProgressPayload { component: "ffmpeg".into(), status: "downloading".into() }).ok();
        let dl_script = format!("$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri \"{}\" -OutFile \"{}\"", ffmpeg_url, zip_path.display());
        let out = std::process::Command::new("powershell").args(["-Command", &dl_script]).output().map_err(|e| e.to_string())?;
        if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).into()); }

        app.emit("repair-progress", ProgressPayload { component: "ffmpeg".into(), status: "extracting".into() }).ok();
        let ext_script = format!(
            r#"
            $ProgressPreference = 'SilentlyContinue'
            Expand-Archive -Path "{}" -DestinationPath "{}" -Force
            $binPath = Join-Path "{}" "ffmpeg-master-latest-win64-gpl\bin\*"
            Copy-Item $binPath -Destination "{}" -Force
            Remove-Item "{}" -Force
            Remove-Item "{}" -Recurse -Force
            "#,
            zip_path.display(), extract_dir.display(), extract_dir.display(), ffmpeg_dir.display(), zip_path.display(), extract_dir.display()
        );
        let out2 = std::process::Command::new("powershell").args(["-Command", &ext_script]).output().map_err(|e| e.to_string())?;
        if !out2.status.success() { return Err(String::from_utf8_lossy(&out2.stderr).into()); }
        
        app.emit("repair-progress", ProgressPayload { component: "ffmpeg".into(), status: "done".into() }).ok();
    }

    if !whisper_ok {
        let whisper_dir = bin_dir.join("whisper");
        std::fs::create_dir_all(&whisper_dir).map_err(|e| e.to_string())?;
        
        let whisper_url = "https://github.com/ggerganov/whisper.cpp/releases/latest/download/whisper-bin-x64.zip";
        let whisper_zip_path = app_dir.join("whisper.zip");
        let whisper_extract_dir = app_dir.join("whisper_extracted");

        app.emit("repair-progress", ProgressPayload { component: "whisper".into(), status: "downloading".into() }).ok();
        let dl_script = format!("$ProgressPreference = 'SilentlyContinue'; Invoke-WebRequest -Uri \"{}\" -OutFile \"{}\"", whisper_url, whisper_zip_path.display());
        let out = std::process::Command::new("powershell").args(["-Command", &dl_script]).output().map_err(|e| e.to_string())?;
        if !out.status.success() { return Err(String::from_utf8_lossy(&out.stderr).into()); }

        app.emit("repair-progress", ProgressPayload { component: "whisper".into(), status: "extracting".into() }).ok();
        let ext_script = format!(
            r#"
            $ProgressPreference = 'SilentlyContinue'
            Expand-Archive -Path "{}" -DestinationPath "{}" -Force
            
            $whisperExeFile = Get-ChildItem -Path "{}" -Filter "whisper-cli.exe" -Recurse | Select-Object -First 1
            if ($null -ne $whisperExeFile) {{
                $whisperInnerDir = $whisperExeFile.DirectoryName
                $whisperItems = Join-Path $whisperInnerDir "*"
                Copy-Item $whisperItems -Destination "{}" -Recurse -Force
            }} else {{
                Write-Error "whisper-cli.exe not found in downloaded zip"
            }}
            
            Remove-Item "{}" -Force
            Remove-Item "{}" -Recurse -Force
            "#,
            whisper_zip_path.display(), whisper_extract_dir.display(), whisper_extract_dir.display(), whisper_dir.display(), whisper_zip_path.display(), whisper_extract_dir.display()
        );
        let out2 = std::process::Command::new("powershell").args(["-Command", &ext_script]).output().map_err(|e| e.to_string())?;
        if !out2.status.success() { return Err(String::from_utf8_lossy(&out2.stderr).into()); }

        app.emit("repair-progress", ProgressPayload { component: "whisper".into(), status: "done".into() }).ok();
    }

    Ok(())
}

#[tauri::command]
pub fn check_cuda() -> bool {
    Command::new("nvidia-smi")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}
