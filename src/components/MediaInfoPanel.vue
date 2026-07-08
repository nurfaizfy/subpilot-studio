<template>
  <div class="media-info-panel">
    <div v-if="!info" class="empty-state">
      <n-empty description="No media analyzed yet" />
    </div>

    <div v-else class="info-content">
      <div class="panel-header">
        <n-icon size="20" class="header-icon">
          <CodeSlashIcon />
        </n-icon>
        <h3 class="header-title">Media Information</h3>
      </div>

      <div class="info-grid">
        <div class="info-card">
          <span class="label">FORMAT / CONTAINER</span>
          <span class="value font-mono">{{ info.format_name.toUpperCase() }}</span>
        </div>
        <div class="info-card">
          <span class="label">FILE SIZE</span>
          <span class="value font-mono">{{ formatBytes(info.file_size_bytes) }}</span>
        </div>
        <div class="info-card">
          <span class="label">VIDEO CODEC</span>
          <span class="value font-mono">{{ info.video_codec.toUpperCase() }}</span>
        </div>
        <div class="info-card">
          <span class="label">RESOLUTION</span>
          <span class="value font-mono">{{ info.width }}x{{ info.height }}</span>
        </div>
        <div class="info-card">
          <span class="label">ASPECT RATIO</span>
          <span class="value font-mono">{{ aspectRatio }}</span>
        </div>
        <div class="info-card">
          <span class="label">FRAME RATE</span>
          <span class="value font-mono">{{ formattedFps }} fps</span>
        </div>
        <div class="info-card">
          <span class="label">BITRATE</span>
          <span class="value font-mono">{{ formatBitrate(info.bit_rate) }}</span>
        </div>
        <div class="info-card">
          <span class="label">DURATION</span>
          <span class="value font-mono">{{ formattedDuration }}</span>
        </div>
      </div>

      <n-divider style="margin: 24px 0; background-color: #334155;" />

      <div class="tracks-section">
        <h4 class="track-header">Audio Tracks ({{ info.audio_tracks.length }})</h4>
        <div v-if="info.audio_tracks.length > 0" class="track-list">
          <div v-for="track in info.audio_tracks" :key="track.index" class="track-badge">
            <n-icon size="14" class="track-icon">
              <PulseIcon />
            </n-icon>
            <span class="font-mono">[{{ track.index }}] {{ track.codec }} - {{ track.language || 'und' }} ({{
              track.channels }}ch)</span>
          </div>
        </div>
        <div v-else class="no-tracks">
          <span>No audio tracks</span>
        </div>

        <h4 class="track-header mt-4">Subtitle Tracks ({{ info.subtitle_tracks.length }})</h4>
        <div v-if="info.subtitle_tracks.length > 0" class="track-list">
          <div v-for="track in info.subtitle_tracks" :key="track.index" class="track-badge">
            <n-icon size="14" class="track-icon">
              <TextIcon />
            </n-icon>
            <span class="font-mono">[{{ track.index }}] {{ track.codec }} - {{ track.language || 'und' }}</span>
          </div>
        </div>
        <div v-else class="no-tracks">
          <span>No subtitle tracks</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { NEmpty, NDivider, NIcon } from 'naive-ui'
import { CodeSlashOutline as CodeSlashIcon, PulseOutline as PulseIcon, DocumentTextOutline as TextIcon } from '@vicons/ionicons5'
import type { MediaInfo } from '../services/mediaService'

const props = defineProps<{
  info: MediaInfo | null
}>()

const formatBytes = (bytes: number) => {
  if (bytes === 0 || !bytes) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatBitrate = (bitrateStr: string) => {
  const bitrate = parseInt(bitrateStr, 10)
  if (isNaN(bitrate) || bitrate === 0) return 'Unknown'
  return formatBytes(bitrate / 8) + '/s'
}

const gcd = (a: number, b: number): number => b === 0 ? a : gcd(b, a % b)

const aspectRatio = computed(() => {
  if (!props.info?.width || !props.info?.height) return 'Unknown'
  // Common aspect ratios to snap to instead of precise math which can look ugly (e.g. 683:384)
  const ratio = props.info.width / props.info.height
  if (Math.abs(ratio - 16 / 9) < 0.05) return '16:9'
  if (Math.abs(ratio - 4 / 3) < 0.05) return '4:3'
  if (Math.abs(ratio - 21 / 9) < 0.05) return '21:9'
  if (Math.abs(ratio - 9 / 16) < 0.05) return '9:16'

  const divisor = gcd(props.info.width, props.info.height)
  return `${props.info.width / divisor}:${props.info.height / divisor}`
})

const formattedFps = computed(() => {
  if (!props.info?.fps) return 'Unknown'
  if (props.info.fps.includes('/')) {
    const [numStr, denStr] = props.info.fps.split('/')
    const num = parseInt(numStr, 10)
    const den = parseInt(denStr, 10)
    if (!isNaN(num) && !isNaN(den) && den !== 0) {
      // 24000/1001 -> 23.976
      return (num / den).toFixed(3).replace(/\.?0+$/, '')
    }
  }
  return props.info.fps
})

const formattedDuration = computed(() => {
  if (!props.info) return '00:00:00'
  const date = new Date(0)
  date.setSeconds(props.info.duration)
  return date.toISOString().substring(11, 19)
})
</script>

<style scoped>
.media-info-panel {
  background-color: #1e293b;
  border-radius: 8px;
  border: 1px solid #334155;
  padding: 24px;
  color: #e2e8f0;
}

.empty-state {
  padding: 40px 0;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.header-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
}

.info-card {
  background-color: #0f172a;
  border: 1px solid #334155;
  border-radius: 6px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.label {
  font-size: 11px;
  color: #cbd5e1;
  letter-spacing: 0.5px;
  font-weight: 600;
}

.value {
  font-size: 12px;
  color: #e2e8f0;
}

.font-mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
}

.tracks-section h4.track-header {
  margin: 0 0 12px 0;
  font-size: 16px;
  font-weight: 600;
  color: #f8fafc;
}

.mt-4 {
  margin-top: 24px !important;
}

.track-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.track-badge {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background-color: #0f172a;
  border: 1px solid #334155;
  border-radius: 4px;
  padding: 6px 10px;
  font-size: 12px;
  color: #e2e8f0;
  width: fit-content;
}

.track-icon {
  color: #94a3b8;
}

.no-tracks {
  display: inline-flex;
  background-color: transparent;
  border: 1px solid #334155;
  border-radius: 4px;
  padding: 6px 12px;
  font-size: 13px;
  color: #94a3b8;
  font-style: italic;
  width: fit-content;
}

@media (max-width: 1200px) {
  .info-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 800px) {
  .info-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
