use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Command;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioStream {
    pub index: usize,
    pub codec: String,
    pub language: String,
    pub channels: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SubtitleStream {
    pub index: usize,
    pub codec: String,
    pub language: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MediaInfo {
    pub path: String,
    pub format_name: String,
    pub file_size_bytes: u64,
    pub bit_rate: String,
    pub video_codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: String,
    pub duration: f64,
    pub audio_tracks: Vec<AudioStream>,
    pub subtitle_tracks: Vec<SubtitleStream>,
}

pub fn analyze_media(path: &str) -> Result<MediaInfo, String> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe returned error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let raw_json = String::from_utf8_lossy(&output.stdout);
    let v: Value = serde_json::from_str(&raw_json)
        .map_err(|e| format!("Failed to parse ffprobe json: {}", e))?;

    let format_duration: f64 = v["format"]["duration"]
        .as_str()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0.0);

    let format_name = v["format"]["format_name"].as_str().unwrap_or("unknown").to_string();
    let file_size_bytes: u64 = v["format"]["size"].as_str().unwrap_or("0").parse().unwrap_or(0);
    let bit_rate = v["format"]["bit_rate"].as_str().unwrap_or("0").to_string();

    let mut video_codec = String::new();
    let mut width = 0;
    let mut height = 0;
    let mut fps = String::new();
    let mut audio_tracks = Vec::new();
    let mut subtitle_tracks = Vec::new();

    if let Some(streams) = v["streams"].as_array() {
        for stream in streams {
            let codec_type = stream["codec_type"].as_str().unwrap_or("");
            let index = stream["index"].as_u64().unwrap_or(0) as usize;
            let codec = stream["codec_name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            let tags = &stream["tags"];
            let language = tags["language"].as_str().unwrap_or("und").to_string();

            if codec_type == "video" && video_codec.is_empty() {
                video_codec = codec.clone();
                width = stream["width"].as_u64().unwrap_or(0) as u32;
                height = stream["height"].as_u64().unwrap_or(0) as u32;
                fps = stream["r_frame_rate"].as_str().unwrap_or("0/0").to_string();
            } else if codec_type == "audio" {
                audio_tracks.push(AudioStream {
                    index,
                    codec,
                    language,
                    channels: stream["channels"].as_u64().unwrap_or(2) as u32,
                });
            } else if codec_type == "subtitle" {
                subtitle_tracks.push(SubtitleStream {
                    index,
                    codec,
                    language,
                });
            }
        }
    }

    Ok(MediaInfo {
        path: path.to_string(),
        format_name,
        file_size_bytes,
        bit_rate,
        video_codec,
        width,
        height,
        fps,
        duration: format_duration,
        audio_tracks,
        subtitle_tracks,
    })
}

pub fn generate_thumbnail(path: &str, output_path: &str) -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i",
            path,
            "-ss",
            "00:00:10.000",
            "-vframes",
            "1",
            "-q:v",
            "2",
            output_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg for thumbnail: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffmpeg thumbnail error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output_path.to_string())
}

pub fn generate_waveform(path: &str, output_path: &str) -> Result<String, String> {
    let output = Command::new("ffmpeg")
        .args(&[
            "-y",
            "-i",
            path,
            "-filter_complex",
            "showwavespic=s=800x150:colors=white",
            "-frames:v",
            "1",
            output_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg for waveform: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "ffmpeg waveform error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(output_path.to_string())
}
