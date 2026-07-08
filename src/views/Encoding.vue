<template>
  <div class="encoding-view">
    <div class="content-grid">
      <n-card class="config-card" title="Export Settings">
        <n-form :model="config" label-placement="top">
          <div class="settings-grid">
            <n-form-item label="Export Action" class="full-width">
              <n-radio-group v-model:value="config.exportMode" name="exportMode">
                <n-space>
                  <n-radio value="video">Encode Video</n-radio>
                  <n-radio value="subtitle">Export Subtitles Only</n-radio>
                </n-space>
              </n-radio-group>
            </n-form-item>

            <template v-if="config.exportMode === 'video'">
              <n-form-item label="Container Format">
                <n-radio-group v-model:value="config.format" name="format">
                  <n-space>
                    <n-radio value="mp4">MP4</n-radio>
                    <n-radio value="mkv">MKV</n-radio>
                  </n-space>
                </n-radio-group>
              </n-form-item>

              <n-form-item label="Subtitle Mode">
                <n-radio-group v-model:value="config.subMode" name="subMode">
                  <n-space>
                    <n-radio value="soft" :disabled="config.format !== 'mkv'">Soft Sub</n-radio>
                    <n-radio value="hard">Hard Sub</n-radio>
                  </n-space>
                </n-radio-group>
              </n-form-item>

              <n-form-item label="Video Codec">
                <n-select v-model:value="config.videoCodec" :options="videoCodecOptions" />
              </n-form-item>

              <n-form-item label="Audio Codec">
                <n-select v-model:value="config.audioCodec" :options="audioCodecOptions" />
              </n-form-item>

              <template v-if="config.videoCodec !== 'copy'">
                <n-form-item label="Resolution">
                  <n-space style="width: 100%" align="center">
                    <n-select v-model:value="config.resolution" :options="resolutionOptions" style="min-width: 120px;" />
                    <template v-if="config.resolution === 'custom'">
                      <n-input v-model:value="config.customResWidth" placeholder="W (e.g. -2)" style="width: 100px" />
                      <span style="padding: 0 4px; font-weight: bold;">x</span>
                      <n-input v-model:value="config.customResHeight" placeholder="H (e.g. 480)" style="width: 100px" />
                    </template>
                  </n-space>
                </n-form-item>

                <n-form-item label="Rate Control">
                  <n-radio-group v-model:value="config.rateControl" name="rateControl">
                    <n-space>
                      <n-radio value="crf">CRF (Constant Quality)</n-radio>
                      <n-radio value="bitrate">Manual Bitrate</n-radio>
                    </n-space>
                  </n-radio-group>
                </n-form-item>

                <n-form-item label="Frame Rate (FPS)">
                  <n-select v-model:value="config.outputFps" :options="outputFpsOptions" />
                </n-form-item>

                <n-form-item label="Preset">
                  <n-select v-model:value="config.preset" :options="presetOptions" />
                </n-form-item>

                <n-form-item v-if="config.rateControl === 'crf'" label="CRF (Quality: 0-51, lower is better)">
                  <n-slider v-model:value="config.crf" :step="1" :max="51" :min="0" />
                  <span style="margin-left: 12px; min-width: 30px">{{ config.crf }}</span>
                </n-form-item>

                <n-form-item v-else label="Video Bitrate (kbps)">
                  <n-input-number v-model:value="config.videoBitrate" placeholder="e.g. 5000" :min="100" :step="500" />
                </n-form-item>
              </template>

              <template v-if="config.audioCodec === 'aac'">
                <n-form-item label="Audio Bitrate">
                  <n-select v-model:value="config.audioBitrate" :options="audioBitrateOptions" />
                </n-form-item>
              </template>
            </template>

            <n-form-item label="Select Subtitle Version" class="full-width">
              <n-select v-model:value="selectedSubtitlePath" :options="subtitleOptions"
                placeholder="Select a subtitle version..." />
            </n-form-item>
          </div>

          <div class="actions">
            <template v-if="config.exportMode === 'video'">
              <n-button type="primary" size="large" block
                :disabled="!projectStore.currentProject || (!canCopy && config.videoCodec === 'copy')"
                @click="addToQueue">
                Add to Queue
              </n-button>
              <div v-if="!canCopy && config.videoCodec === 'copy'" class="warning-text">
                Hard Subs require re-encoding (cannot use Copy codec).
              </div>
            </template>
            <template v-else>
              <n-button type="primary" size="large" block
                :disabled="!projectStore.currentProject || !selectedSubtitlePath" @click="exportSubtitles">
                Export Subtitle File
              </n-button>
            </template>
          </div>
        </n-form>
      </n-card>

      <n-card class="queue-card" title="Job Queue">
        <template #header-extra>
          <n-button size="small" @click="encoderStore.clearDone">Clear Done</n-button>
        </template>

        <div v-if="encoderStore.queue.length === 0" class="empty-state">
          <n-empty description="Queue is empty" />
        </div>

        <div v-else class="queue-list">
          <div v-for="job in encoderStore.queue" :key="job.id" class="job-item" :class="job.status">
            <div class="job-header">
              <span class="job-name">{{ job.projectName }}</span>
              <n-tag size="small" :type="getStatusType(job.status)">
                {{ job.status.toUpperCase() }}
              </n-tag>
            </div>

            <div class="job-details">
              {{ job.subMode === 'hard' ? 'Hard Sub' : 'Soft Sub' }} • {{ job.format.toUpperCase() }} • {{
                job.videoCodec === 'copy' ? 'Copy' : (job.rateControl === 'bitrate' ? `Bitrate ${job.videoBitrate}` : `CRF ${job.crf}`)
              }} • {{ job.outputFps === 'original' ? 'Original FPS' : (job.outputFps.includes('/') ? (parseInt(job.outputFps.split('/')[0]) / parseInt(job.outputFps.split('/')[1])).toFixed(3) : job.outputFps) + ' FPS' }}
            </div>

            <div class="progress-section" v-if="job.status === 'encoding'">
              <n-progress type="line" :percentage="Math.round(job.percent)" />
              <div class="time-label">{{ job.timeStr }} • {{ job.fps }} fps</div>
            </div>

            <div class="job-actions" v-if="job.status === 'pending'">
              <n-button size="tiny" @click="encoderStore.removeJob(job.id)">Remove</n-button>
              <n-button size="tiny" type="primary" @click="startJob(job)"
                :disabled="!!encoderStore.activeJobId">Start</n-button>
            </div>
            <div class="job-actions" v-if="job.status === 'encoding'">
              <n-button size="tiny" type="error" @click="cancelJob">Cancel</n-button>
            </div>
          </div>
        </div>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import {
  NCard, NForm, NFormItem, NSelect, NRadioGroup, NRadio, NSpace,
  NButton, NProgress, NEmpty, NTag, useMessage, NSlider, NInput, NInputNumber
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useProjectStore } from '../stores/project'
import { useEncoderStore, type EncodeJob } from '../stores/encoder'
import { MediaService } from '../services/mediaService'
import { save } from '@tauri-apps/plugin-dialog'
import { SubtitleManager } from '../services/subtitleManager'
import type { SubtitleVersion } from '../services/subtitleManager'

const message = useMessage()
const projectStore = useProjectStore()
const encoderStore = useEncoderStore()

const availableSubtitles = ref<SubtitleVersion[]>([])
const selectedSubtitlePath = ref<string | null>(null)

const subtitleOptions = computed(() => {
  return availableSubtitles.value.map(v => ({
    label: `${v.category.toUpperCase()} v${v.version} (${v.filename})`,
    value: v.path
  }))
})

let unlisten: (() => void) | null = null

const config = ref({
  exportMode: 'video',
  format: 'mp4',
  subMode: 'hard',
  videoCodec: 'libx264',
  resolution: 'original',
  customResWidth: '848',
  customResHeight: '480',
  rateControl: 'crf',
  crf: 23,
  videoBitrate: 5000,
  preset: 'medium',
  audioCodec: 'copy',
  audioBitrate: '192k',
  outputFps: 'original'
})

const canCopy = computed(() => config.value.subMode !== 'hard')

import { watch } from 'vue'
watch(() => config.value.format, (newFormat) => {
  if (newFormat !== 'mkv' && config.value.subMode === 'soft') {
    config.value.subMode = 'hard'
  }
})

const loadSubtitles = async () => {
  if (projectStore.currentProject) {
    availableSubtitles.value = await SubtitleManager.getProjectSubtitles(projectStore.currentProject.id)
    if (availableSubtitles.value.length > 0) {
      selectedSubtitlePath.value = availableSubtitles.value[0].path
    } else {
      selectedSubtitlePath.value = null
    }
  } else {
    availableSubtitles.value = []
    selectedSubtitlePath.value = null
  }
}

watch(() => projectStore.currentProject?.id, () => {
  loadSubtitles()
}, { immediate: true })

const videoCodecOptions = computed(() => {
  const options = [
    { label: 'H.264 (libx264)', value: 'libx264' },
    { label: 'H.265 (libx265)', value: 'libx265' },
    { label: 'NVENC H.264 (NVIDIA)', value: 'h264_nvenc' }
  ]
  if (canCopy.value) {
    options.unshift({ label: 'Copy (No Re-encoding)', value: 'copy' })
  } else if (config.value.videoCodec === 'copy') {
    config.value.videoCodec = 'libx264'
  }
  return options
})

const audioCodecOptions = [
  { label: 'Copy', value: 'copy' },
  { label: 'AAC', value: 'aac' }
]

const audioBitrateOptions = [
  { label: '64 kbps', value: '64k' },
  { label: '96 kbps', value: '96k' },
  { label: '128 kbps', value: '128k' },
  { label: '192 kbps', value: '192k' },
  { label: '256 kbps', value: '256k' }
]

const resolutionOptions = [
  { label: 'Original', value: 'original' },
  { label: '1080p', value: '1080p' },
  { label: '720p', value: '720p' },
  { label: '480p', value: '480p' },
  { label: '360p', value: '360p' },
  { label: 'Custom', value: 'custom' }
]

const outputFpsOptions = [
  { label: 'Same as Source', value: 'original' },
  { label: '23.976 fps', value: '24000/1001' },
  { label: '24 fps', value: '24' },
  { label: '25 fps', value: '25' },
  { label: '29.97 fps', value: '30000/1001' },
  { label: '30 fps', value: '30' },
  { label: '50 fps', value: '50' },
  { label: '59.94 fps', value: '60000/1001' },
  { label: '60 fps', value: '60' }
]

const presetOptions = [
  { label: 'Ultrafast', value: 'ultrafast' },
  { label: 'Superfast', value: 'superfast' },
  { label: 'Veryfast', value: 'veryfast' },
  { label: 'Faster', value: 'faster' },
  { label: 'Fast', value: 'fast' },
  { label: 'Medium', value: 'medium' },
  { label: 'Slow', value: 'slow' },
  { label: 'Slower', value: 'slower' },
  { label: 'Veryslow', value: 'veryslow' }
]

onMounted(async () => {
  unlisten = await listen('encoding-progress', (event) => {
    try {
      const data: any = event.payload
      encoderStore.updateProgress(data.id, data.percent, data.time_str, data.fps || 0, 'encoding')
    } catch (e) {
      console.error(e)
    }
  })

  const unlistenDone = await listen('encoding-done', (event) => {
    try {
      const data: any = event.payload
      encoderStore.updateProgress(data.id, data.percent, data.time_str, data.fps || 0, data.status)
      systemStore.isEncoding = false
      systemStore.setTask('Idle')
      if (data.status === 'success') {
        message.success('Encoding job completed!')
      } else {
        message.error('Encoding job failed!')
      }
    } catch (e) {
      console.error(e)
    }
  })

  const oldUnlisten = unlisten
  unlisten = () => {
    if (oldUnlisten) oldUnlisten()
    unlistenDone()
  }
})

onUnmounted(() => {
  if (unlisten) unlisten()
})

const addToQueue = async () => {
  if (!projectStore.currentProject) {
    message.error("Please select a project first")
    return
  }

  const project = projectStore.currentProject
  if (!project || !project.source_video) {
    message.error("Project has no source video")
    return
  }

  if (!selectedSubtitlePath.value) {
    message.error("Please select a subtitle version to burn or encode")
    return
  }

  try {
    const mediaInfo = await MediaService.analyzeMedia(project.source_video)

    const defaultName = project.source_video.split(/[\\/]/).pop()?.replace(/\.[^/.]+$/, `_encoded.${config.value.format}`) || `output.${config.value.format}`
    const filePath = await save({
      filters: [{ name: 'Video', extensions: [config.value.format] }],
      defaultPath: defaultName
    })

    if (!filePath) {
      return
    }

    const job: EncodeJob = {
      id: Math.random().toString(36).substring(2, 9),
      projectId: project.id,
      projectName: project.name,
      inputVideo: project.source_video,
      inputSub: selectedSubtitlePath.value,
      outputPath: filePath,
      videoCodec: config.value.videoCodec,
      resolution: config.value.resolution === 'custom' ? `${config.value.customResWidth}x${config.value.customResHeight}` : config.value.resolution,
      rateControl: config.value.rateControl,
      crf: config.value.crf,
      videoBitrate: config.value.videoBitrate.toString() + 'k',
      preset: config.value.preset,
      audioCodec: config.value.audioCodec,
      audioBitrate: config.value.audioBitrate,
      subMode: config.value.subMode,
      format: config.value.format,
      durationSeconds: mediaInfo.duration,
      outputFps: config.value.outputFps,
      status: 'pending',
      percent: 0,
      timeStr: '00:00:00',
      fps: 0
    }

    encoderStore.addJob(job)
    message.success("Job added to queue")
  } catch (e: any) {
    message.error("Failed to add job: " + e)
  }
}

import { useSystemStore } from '../stores/system'
const systemStore = useSystemStore()

const startJob = async (job: EncodeJob) => {
  if (encoderStore.activeJobId) {
    message.warning("A job is already running")
    return
  }

  job.status = 'encoding'
  encoderStore.activeJobId = job.id
  systemStore.isEncoding = true
  systemStore.setTask('Encoding Video...')

  try {
    await invoke('start_encoding_cmd', {
      config: {
        id: job.id,
        input_video: job.inputVideo,
        input_sub: job.inputSub,
        output_path: job.outputPath,
        video_codec: job.videoCodec,
        resolution: job.resolution,
        rate_control: job.rateControl,
        crf: job.crf,
        video_bitrate: job.videoBitrate,
        preset: job.preset,
        audio_codec: job.audioCodec,
        audio_bitrate: job.audioBitrate,
        sub_mode: job.subMode,
        format: job.format,
        duration_seconds: job.durationSeconds,
        output_fps: job.outputFps
      }
    })
  } catch (e: any) {
    message.error("Failed to start encoder: " + e)
    job.status = 'error'
    encoderStore.activeJobId = null
    systemStore.isEncoding = false
    systemStore.setTask('Idle')
  }
}

const cancelJob = async () => {
  try {
    await invoke('cancel_encoding_cmd')
    systemStore.isEncoding = false
    systemStore.setTask('Idle')
    message.warning("Encoding cancelled")
  } catch (e: any) {
    message.error("Failed to cancel: " + e)
  }
}

const getStatusType = (status: string) => {
  if (status === 'pending') return 'default'
  if (status === 'encoding') return 'info'
  if (status === 'success') return 'success'
  return 'error'
}

const exportSubtitles = async () => {
  if (!selectedSubtitlePath.value) {
    message.warning("Please select a subtitle to export!")
    return
  }

  const selectedSub = availableSubtitles.value.find(v => v.path === selectedSubtitlePath.value)
  if (!selectedSub) return

  try {
    const ext = selectedSub.filename.split('.').pop() || 'srt'
    const defaultName = projectStore.currentProject ? `${projectStore.currentProject.name}_${selectedSub.category}_v${selectedSub.version}.${ext}` : selectedSub.filename
    const filePath = await save({
      filters: [{ name: 'Subtitle', extensions: [ext] }],
      defaultPath: defaultName
    })

    if (filePath) {
      const content = await invoke<string>('read_text_file', { path: selectedSubtitlePath.value })
      await invoke('write_text_file', {
        path: filePath,
        content: content
      })
      message.success("Subtitle exported successfully")
    }
  } catch (e) {
    message.error("Failed to export subtitle: " + e)
  }
}
</script>

<style scoped>
.encoding-view {
  max-width: 100%;
  margin: 0 auto;
}

.header {
  margin-bottom: 24px;
}

.header h2 {
  font-size: 28px;
  margin: 0;
  color: #f8fafc;
}

.subtitle {
  color: #94a3b8;
  margin: 4px 0 0 0;
}

.content-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

@media (max-width: 1000px) {
  .content-grid {
    grid-template-columns: 1fr;
  }
}

.config-card {
  background-color: #1e293b;
}

.settings-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0 24px;
}

.full-width {
  grid-column: span 2;
}

.queue-card {
  background-color: #1e293b;
}

.actions {
  margin-top: 24px;
}

.warning-text {
  color: #ef4444;
  font-size: 13px;
  margin-top: 8px;
  text-align: center;
}

.queue-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.job-item {
  background-color: #0f172a;
  padding: 16px;
  border-radius: 8px;
  border-left: 4px solid #475569;
}

.job-item.encoding {
  border-left-color: #3b82f6;
}

.job-item.success {
  border-left-color: #10b981;
}

.job-item.error {
  border-left-color: #ef4444;
}

.job-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.job-name {
  font-weight: 600;
  color: #f1f5f9;
}

.job-details {
  font-size: 13px;
  color: #94a3b8;
  margin-bottom: 12px;
}

.progress-section {
  margin-bottom: 12px;
}

.time-label {
  font-size: 12px;
  color: #64748b;
  text-align: right;
  margin-top: 4px;
}

.job-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
