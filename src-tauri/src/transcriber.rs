use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct TranscriberState {
    pub active_process: Arc<Mutex<Option<Child>>>,
}

impl Default for TranscriberState {
    fn default() -> Self {
        Self {
            active_process: Arc::new(Mutex::new(None)),
        }
    }
}

pub fn start_transcription(
    app: AppHandle,
    state: State<'_, TranscriberState>,
    project_id: String,
    input_path: String,
    duration: f64,
    model: String,
    language: String,
    _device: String,
    format: String,
    custom_output_prefix: Option<String>,
) -> Result<(), String> {
    {
        let mut child_guard = state.active_process.lock().unwrap();
        if let Some(child) = child_guard.as_mut() {
            if let Ok(None) = child.try_wait() {
                return Err("A transcription is already running. Please cancel it first.".into());
            }
        }
    }

    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    
    let output_dir = app_dir.join("subtitles").join(&project_id);
    std::fs::create_dir_all(&output_dir).unwrap();
    
    let output_prefix = match custom_output_prefix {
        Some(prefix) => std::path::PathBuf::from(prefix),
        None => output_dir.join(&project_id)
    };
    let wav_path = output_dir.join(&format!("{}.wav", project_id));
    
    let models_dir = app_dir.join("models");
    let model_bin = models_dir.join(format!("ggml-{}.bin", model));
    
    let bin_dir = app_dir.join("bin");
    let whisper_dir = bin_dir.join("whisper");
    let whisper_exe = whisper_dir.join("whisper-cli.exe");

    if !whisper_exe.exists() {
        return Err("whisper-cli.exe not found. Please ensure it is downloaded to AppData/bin/whisper.".into());
    }
    if !model_bin.exists() {
        return Err(format!("Model {} not found. Please download it first.", model));
    }

    let state_clone = Arc::clone(&state.inner().active_process);
    
    thread::spawn(move || {
        let _ = app.emit("transcription-event", r#"{"type": "progress", "percent": 0, "current_segment": "Converting audio to 16kHz WAV (FFmpeg)..."}"#);
        
        let mut ffmpeg_cmd = Command::new("ffmpeg");
        ffmpeg_cmd.arg("-y")
            .arg("-i").arg(&input_path)
            .arg("-ar").arg("16000")
            .arg("-ac").arg("1")
            .arg("-c:a").arg("pcm_s16le")
            .arg(&wav_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
            
        let ffmpeg_child = match ffmpeg_cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = app.emit("transcription-event", &format!(r#"{{"type": "error", "message": "FFmpeg failed to start: {}"}}"#, e));
                return;
            }
        };
        
        {
            let mut guard = state_clone.lock().unwrap();
            *guard = Some(ffmpeg_child);
        }
        
        let mut child = {
            let mut guard = state_clone.lock().unwrap();
            guard.take().unwrap()
        };
        
        let status = child.wait();
        if status.is_err() || !status.unwrap().success() {
            let _ = app.emit("transcription-event", r#"{"type": "error", "message": "Audio conversion failed or cancelled."}"#);
            let _ = std::fs::remove_file(&wav_path);
            return;
        }

        let _ = app.emit("transcription-event", r#"{"type": "progress", "percent": 5, "current_segment": "Starting whisper.cpp..."}"#);

        let mut whisper_cmd = Command::new(&whisper_exe);
        whisper_cmd.arg("-m").arg(&model_bin)
            .arg("-f").arg(&wav_path);
        
        if language != "auto" {
            whisper_cmd.arg("-l").arg(&language);
        }

        whisper_cmd.arg("-of").arg(&output_prefix);
        
        if format == "srt" || format == "ass" {
            whisper_cmd.arg("-osrt");
        } else {
            whisper_cmd.arg("-otxt");
        }

        whisper_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut whisper_child = match whisper_cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = app.emit("transcription-event", &format!(r#"{{"type": "error", "message": "whisper.cpp failed to start: {}"}}"#, e));
                let _ = std::fs::remove_file(&wav_path);
                return;
            }
        };

        let stdout = whisper_child.stdout.take().unwrap();
        let stderr = whisper_child.stderr.take().unwrap();
        
        {
            let mut guard = state_clone.lock().unwrap();
            *guard = Some(whisper_child);
        }
        
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for _ in reader.lines() {}
        });

        let start_time = Instant::now();
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line_str) = line {
                if line_str.starts_with('[') && line_str.contains("-->") {
                    let parts: Vec<&str> = line_str.split("-->").collect();
                    if parts.len() == 2 {
                        let end_time_str = parts[1].split(']').next().unwrap_or("").trim();
                        let time_parts: Vec<&str> = end_time_str.split(':').collect();
                        if time_parts.len() == 3 {
                            let h: f64 = time_parts[0].parse().unwrap_or(0.0);
                            let m: f64 = time_parts[1].parse().unwrap_or(0.0);
                            let s_parts: Vec<&str> = time_parts[2].split('.').collect();
                            let s: f64 = s_parts.get(0).unwrap_or(&"0").parse().unwrap_or(0.0);
                            
                            let current_sec = h * 3600.0 + m * 60.0 + s;
                            let mut pct = (current_sec / duration) * 100.0;
                            if pct > 99.0 { pct = 99.0; }
                            if pct < 5.0 { pct = 5.0; }
                            
                            let elapsed_real = start_time.elapsed().as_secs_f64();
                            let speed = if elapsed_real > 0.0 { current_sec / elapsed_real } else { 1.0 };
                            let remaining_audio = duration - current_sec;
                            let eta_sec = if speed > 0.0 { remaining_audio / speed } else { 0.0 };
                            
                            let safe_segment = line_str.replace("\"", "\\\"");
                            
                            let event_json = format!(
                                r#"{{"type": "progress", "percent": {:.1}, "current_segment": "{}", "eta": {:.0}, "speed": {:.1}}}"#,
                                pct, safe_segment, eta_sec, speed
                            );
                            let _ = app.emit("transcription-event", &event_json);
                        }
                    }
                }
            }
        }

        let mut child = {
            let mut guard = state_clone.lock().unwrap();
            if let Some(c) = guard.take() { c } else { return; }
        };
        let status = child.wait();
        
        let _ = std::fs::remove_file(&wav_path);

        if status.is_ok() && status.unwrap().success() {
            let _ = app.emit("transcription-event", r#"{"type": "done", "message": "Transcription completed."}"#);
        } else {
            let _ = app.emit("transcription-event", r#"{"type": "error", "message": "Transcription failed or cancelled."}"#);
        }
    });

    Ok(())
}

pub fn cancel_transcription(state: State<'_, TranscriberState>) -> Result<(), String> {
    let mut child_guard = state.active_process.lock().unwrap();
    if let Some(mut child) = child_guard.take() {
        match child.kill() {
            Ok(_) => {
                let _ = child.wait();
                Ok(())
            }
            Err(e) => Err(format!("Failed to kill process: {}", e)),
        }
    } else {
        Err("No active transcription found.".into())
    }
}
