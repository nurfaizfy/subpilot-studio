use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Serialize, Deserialize, Clone)]
pub struct EncodeConfig {
    pub id: String,
    pub input_video: String,
    pub input_sub: String,
    pub output_path: String,
    pub video_codec: String,
    pub resolution: String,
    pub rate_control: String,
    pub crf: u8,
    pub video_bitrate: String,
    pub preset: String,
    pub audio_codec: String,
    pub audio_bitrate: String,
    pub sub_mode: String,
    pub format: String,
    pub duration_seconds: f64,
    pub output_fps: String,
}

#[derive(Serialize, Clone)]
pub struct EncodeProgress {
    pub id: String,
    pub percent: f64,
    pub time_str: String,
    pub fps: f64,
    pub status: String,
}

lazy_static::lazy_static! {
    pub static ref ACTIVE_ENCODE_PROC: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
}

fn time_to_seconds(time_str: &str) -> f64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let h: f64 = parts[0].parse().unwrap_or(0.0);
        let m: f64 = parts[1].parse().unwrap_or(0.0);
        let s: f64 = parts[2].parse().unwrap_or(0.0);
        return h * 3600.0 + m * 60.0 + s;
    }
    0.0
}

fn get_ass_fonts(ass_path: &str) -> Vec<String> {
    let mut fonts = HashSet::new();
    if let Ok(content) = fs::read_to_string(ass_path) {
        let mut in_styles = false;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("[V4+ Styles]") {
                in_styles = true;
                continue;
            }
            if in_styles && line.starts_with("[") {
                break;
            }
            if in_styles && line.starts_with("Style:") {
                let parts: Vec<&str> = line.splitn(3, ',').collect();
                if parts.len() >= 3 {
                    let fontname = parts[1].trim().to_string();
                    if !fontname.is_empty() {
                        fonts.insert(fontname);
                    }
                }
            }
        }
    }
    fonts.into_iter().collect()
}

fn resolve_windows_fonts(font_names: Vec<String>) -> Vec<String> {
    let mut resolved_paths = Vec::new();
    let fonts_dir = "C:\\Windows\\Fonts";

    if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts", KEY_READ)
    {
        let value_names: Vec<String> = hklm.enum_values().filter_map(|x| x.ok().map(|(n, _)| n)).collect();
        for name in value_names {
            if let Ok(value_str) = hklm.get_value::<String, _>(&name) {
                let name_lower = name.to_lowercase();
                for requested_font in &font_names {
                    if name_lower.starts_with(&requested_font.to_lowercase()) {
                        let path = Path::new(fonts_dir).join(&value_str);
                        if path.exists() {
                            resolved_paths.push(path.to_string_lossy().to_string());
                        }
                        break;
                    }
                }
            }
        }
    }
    
    resolved_paths.into_iter().collect::<HashSet<_>>().into_iter().collect()
}

pub fn start_encoding(app: AppHandle, config: EncodeConfig) -> Result<(), String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        config.input_video.clone(),
    ];

    if config.sub_mode == "soft" {
        args.push("-i".to_string());
        args.push(config.input_sub.clone());
    }

    let mut vf_filters = Vec::new();

    if config.resolution != "original" {
        let scale_val = if config.resolution.ends_with("p") {
            let height = config.resolution.replace("p", "");
            format!("-2:{}", height)
        } else {
            config.resolution.replace("x", ":")
        };
        vf_filters.push(format!("scale={}", scale_val));
    }

    if config.sub_mode == "hard" {
        let escaped_path = config.input_sub.replace("\\", "/").replace(":", "\\:");
        vf_filters.push(format!("subtitles='{}'", escaped_path));
    }

    if !vf_filters.is_empty() {
        args.push("-vf".to_string());
        args.push(vf_filters.join(","));
    }

    args.push("-c:v".to_string());
    args.push(config.video_codec.clone());

    if config.video_codec != "copy" {
        if config.output_fps != "original" {
            args.push("-r".to_string());
            args.push(config.output_fps.clone());
        }

        if config.rate_control == "bitrate" {
            args.push("-b:v".to_string());
            args.push(config.video_bitrate.clone());
        } else {
            args.push("-crf".to_string());
            args.push(config.crf.to_string());
        }
        args.push("-preset".to_string());
        args.push(config.preset.clone());
    }

    args.push("-c:a".to_string());
    args.push(config.audio_codec.clone());
    
    if config.audio_codec == "aac" {
        args.push("-b:a".to_string());
        args.push(config.audio_bitrate.clone());
    }

    if config.sub_mode == "soft" {
        args.push("-c:s".to_string());
        if config.format == "mp4" {
            args.push("mov_text".to_string());
        } else {
            if config.input_sub.to_lowercase().ends_with(".ass") {
                args.push("ass".to_string());
            } else {
                args.push("srt".to_string());
            }
        }
    }

    if config.format == "mkv" && config.sub_mode == "soft" && config.input_sub.to_lowercase().ends_with(".ass") {
        let extracted_fonts = get_ass_fonts(&config.input_sub);
        let attach_fonts = resolve_windows_fonts(extracted_fonts);
        
        let mut attach_idx = 0;
        for font_path in attach_fonts {
            let escaped_font = font_path.replace("\\", "/");
            args.push("-attach".to_string());
            args.push(escaped_font.clone());
            args.push(format!("-metadata:s:t:{}", attach_idx));
            
            let mimetype = if escaped_font.to_lowercase().ends_with(".otf") {
                "mimetype=font/otf"
            } else {
                "mimetype=application/x-truetype-font"
            };
            args.push(mimetype.to_string());
            
            attach_idx += 1;
        }
    }

    args.push(config.output_path.clone());

    let mut child = match Command::new("ffmpeg")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Err(format!("Failed to start ffmpeg: {}", e)),
    };

    let stderr = child.stderr.take().unwrap();

    {
        let mut proc_guard = ACTIVE_ENCODE_PROC.lock().unwrap();
        *proc_guard = Some(child);
    }

    let app_clone = app.clone();
    let config_clone = config.clone();

    std::thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut buffer = Vec::new();

        while let Ok(bytes_read) = reader.read_until(b'\r', &mut buffer) {
            if bytes_read == 0 {
                break;
            }

            let line = String::from_utf8_lossy(&buffer);
            
            let mut fps_val = 0.0;
            if let Some(idx) = line.find("fps=") {
                let fps_substr = &line[idx + 4..];
                let fps_str = fps_substr.trim_start();
                let end_idx = fps_str.find(' ').unwrap_or(fps_str.len());
                let fps_token = &fps_str[..end_idx];
                fps_val = fps_token.parse().unwrap_or(0.0);
            }

            if let Some(idx) = line.find("time=") {
                let time_substr = &line[idx + 5..];
                let end_idx = time_substr.find(' ').unwrap_or(time_substr.len());
                let time_str = &time_substr[..end_idx];
                
                let current_sec = time_to_seconds(time_str);

                let mut percent = 0.0;
                if config_clone.duration_seconds > 0.0 {
                    percent = (current_sec / config_clone.duration_seconds) * 100.0;
                    if percent > 100.0 {
                        percent = 100.0;
                    }
                }
                
                let _ = app_clone.emit(
                    "encoding-progress",
                    EncodeProgress {
                        id: config_clone.id.clone(),
                        percent,
                        time_str: time_str.to_string(),
                        fps: fps_val,
                        status: "encoding".to_string(),
                    },
                );
            }
            
            buffer.clear();
        }

        let mut proc_guard = ACTIVE_ENCODE_PROC.lock().unwrap();
        if let Some(mut child) = proc_guard.take() {
            let status = child.wait().unwrap();
            if status.success() {
                let _ = app_clone.emit(
                    "encoding-done",
                    EncodeProgress {
                        id: config_clone.id.clone(),
                        percent: 100.0,
                        time_str: "Done".to_string(),
                        fps: 0.0,
                        status: "success".to_string(),
                    },
                );
            } else {
                let _ = app_clone.emit(
                    "encoding-done",
                    EncodeProgress {
                        id: config_clone.id.clone(),
                        percent: 0.0,
                        time_str: "Error".to_string(),
                        fps: 0.0,
                        status: "error".to_string(),
                    },
                );
            }
        }
    });

    Ok(())
}

pub fn cancel_encoding() -> Result<(), String> {
    let mut proc_guard = ACTIVE_ENCODE_PROC.lock().unwrap();
    if let Some(mut child) = proc_guard.take() {
        let _ = child.kill();
        let _ = child.wait();
        Ok(())
    } else {
        Err("No active encoding process".to_string())
    }
}
