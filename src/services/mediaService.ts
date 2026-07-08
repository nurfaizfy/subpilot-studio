import { invoke } from '@tauri-apps/api/core'

export interface AudioStream {
  index: number;
  codec: string;
  language: string;
  channels: number;
}

export interface SubtitleStream {
  index: number;
  codec: string;
  language: string;
}

export interface MediaInfo {
  path: string;
  format_name: string;
  file_size_bytes: number;
  bit_rate: string;
  video_codec: string;
  width: number;
  height: number;
  fps: string;
  duration: number;
  audio_tracks: AudioStream[];
  subtitle_tracks: SubtitleStream[];
}

export class MediaService {
  /**
   * Analyzes a media file using FFprobe and returns its metadata.
   */
  static async analyzeMedia(path: string): Promise<MediaInfo> {
    try {
      const info: MediaInfo = await invoke('analyze_media', { path })
      return info
    } catch (e) {
      console.error("Failed to analyze media:", e)
      throw e
    }
  }

  /**
   * Generates a thumbnail for a given project's video.
   * Returns the absolute path to the generated thumbnail.
   */
  static async generateThumbnail(path: string, projectId: string): Promise<string> {
    try {
      const thumbnailPath: string = await invoke('generate_thumbnail', { path, projectId })
      return thumbnailPath
    } catch (e) {
      console.error("Failed to generate thumbnail:", e)
      throw e
    }
  }

  /**
   * Generates a waveform preview for a given project's video.
   * Returns the absolute path to the generated waveform image.
   */
  static async generateWaveform(path: string, projectId: string): Promise<string> {
    try {
      const waveformPath: string = await invoke('generate_waveform', { path, projectId })
      return waveformPath
    } catch (e) {
      console.error("Failed to generate waveform:", e)
      throw e
    }
  }
}
