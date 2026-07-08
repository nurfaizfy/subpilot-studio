<template>
  <div class="ai-view">
    <n-tabs type="segment" animated @before-leave="handleTabLeave">
      <n-tab-pane name="transcription" :tab="$t('aistudio.speechRecognition')">
        <div class="content-grid">
          <n-card class="settings-card" :title="$t('aistudio.speechSettings')">
            <n-form :model="config" label-placement="top">
              <n-form-item :label="$t('aistudio.whisperModel')">
                <n-select v-model:value="config.model" :options="modelOptions" />
              </n-form-item>

              <n-form-item :label="$t('aistudio.language')">
                <n-select v-model:value="config.language" :options="langOptions" />
              </n-form-item>

              <n-form-item :label="$t('aistudio.computeDevice')">
                <n-radio-group v-model:value="config.device" name="device">
                  <n-space>
                    <n-radio value="cpu">{{ $t('aistudio.cpu') }}</n-radio>
                    <n-radio value="cuda" :disabled="!hasCuda">
                      {{ $t('aistudio.gpu') }}
                      <span v-if="!hasCuda" style="font-size: 11px; margin-left: 4px; color: #ef4444;">{{
                        $t('aistudio.notSupported') }}</span>
                    </n-radio>
                  </n-space>
                </n-radio-group>
              </n-form-item>

              <n-form-item :label="$t('aistudio.outputFormat')">
                <n-select v-model:value="config.format" :options="formatOptions" />
              </n-form-item>

              <div class="actions">
                <n-button type="primary" size="large" block :disabled="!projectStore.currentProject || isRunning"
                  @click="startTranscription">
                  {{ $t('aistudio.startTranscription') }}
                </n-button>
              </div>
            </n-form>
          </n-card>

          <n-card class="progress-card" :title="$t('aistudio.status')">
            <template #header-extra>
              <n-tag :type="isRunning ? 'info' : 'default'">
                {{ isRunning ? $t('aistudio.running') : $t('aistudio.idle') }}
              </n-tag>
            </template>

            <div v-if="!hasStarted && !isRunning" class="empty-state">
              <n-empty :description="$t('aistudio.configureAndStart')" />
            </div>

            <div v-else class="progress-content">
              <div class="stats-row">
                <n-statistic :label="$t('aistudio.progress')" :value="progress.percent + '%'" />
                <n-statistic :label="$t('aistudio.speed')" :value="(progress.speed || 0).toFixed(1) + 'x'" />
                <n-statistic :label="$t('aistudio.eta')" :value="formatTime(progress.eta)" />
              </div>

              <n-progress type="line" :percentage="progress.percent" :indicator-placement="'inside'" processing
                class="progress-bar" />

              <div class="live-caption-box" :class="{ 'is-running': isRunning }">
                <span class="caption-label"><i v-if="isRunning"></i>{{ $t('aistudio.currentSegment') }}</span>
                <p class="caption-text">{{ progress.current_segment || $t('aistudio.initializing') }}</p>
              </div>

              <div class="control-actions">
                <n-button v-if="isRunning" type="error" @click="cancelTranscription">
                  {{ $t('aistudio.cancel') }}
                </n-button>
                <n-button v-if="!isRunning && hasStarted" type="primary" @click="startTranscription">
                  {{ $t('aistudio.resumeRetry') }}
                </n-button>
              </div>
            </div>
          </n-card>
        </div>
      </n-tab-pane>

      <n-tab-pane name="translation" :tab="$t('aistudio.aiTranslation')">
        <div class="content-grid">
          <n-card class="settings-card" :title="$t('aistudio.translationSettings')">
            <n-form label-placement="top">
              <n-form-item :label="$t('aistudio.provider')">
                <n-select v-model:value="translationStore.provider" :options="providerOptions" />
              </n-form-item>

              <n-form-item :label="$t('aistudio.model')">
                <n-input v-model:value="translationStore.model" :placeholder="$t('aistudio.modelPlaceholder')"
                  readonly />
              </n-form-item>

              <div class="lang-row">
                <n-form-item :label="$t('aistudio.sourceSubtitle')">
                  <n-select v-model:value="selectedSourcePath" :options="sourceOptions"
                    :placeholder="$t('aistudio.selectSource')" />
                </n-form-item>

                <n-form-item :label="$t('aistudio.sourceLanguage')">
                  <n-select v-model:value="translationStore.sourceLang" :options="langOptions" />
                </n-form-item>
                <div class="arrow">→</div>
                <n-form-item :label="$t('aistudio.target')">
                  <n-select v-model:value="translationStore.targetLang" :options="targetLangOptions" />
                </n-form-item>
              </div>

              <n-form-item :label="$t('aistudio.batchSize')">
                <n-input-number v-model:value="transConfig.batchSize" :min="10" :max="100" />
              </n-form-item>

              <n-form-item :label="$t('aistudio.customPrompt')">
                <n-input type="textarea" v-model:value="translationStore.customPrompt"
                  :placeholder="$t('aistudio.promptPlaceholder')" :autosize="{ minRows: 2, maxRows: 4 }" />
              </n-form-item>

              <div class="actions">
                <n-button v-if="!isTranslating" type="primary" size="large" block
                  :disabled="!projectStore.currentProject || !selectedSourcePath" @click="startTranslation">
                  {{ $t('aistudio.startTranslation') }}
                </n-button>
                <n-button v-else type="error" size="large" block @click="cancelTranslation">
                  {{ $t('aistudio.cancel') }}
                </n-button>
                <div v-if="!selectedSourcePath" style="color: #f59e0b; margin-top: 8px; text-align: center;">
                  {{ $t('aistudio.generateFirst') }}
                </div>
              </div>
            </n-form>
          </n-card>

          <n-card class="progress-card" :title="$t('aistudio.translationQueue')">
            <template #header-extra>
              <n-tag :type="isTranslating ? 'info' : 'default'">
                {{ isTranslating ? $t('aistudio.translating') : $t('aistudio.idle') }}
              </n-tag>
            </template>

            <div v-if="chunks.length === 0" class="empty-state">
              <n-empty :description="$t('aistudio.readyToTranslate')" />
            </div>

            <div v-else class="queue-list">
              <div class="stats-row" style="margin-bottom: 16px;">
                <n-statistic :label="$t('aistudio.totalChunks')" :value="chunks.length" />
                <n-statistic :label="$t('aistudio.completed')"
                  :value="chunks.filter(c => c.status === 'success').length" />
                <n-statistic :label="$t('aistudio.failed')" :value="chunks.filter(c => c.status === 'error').length" />
              </div>

              <div class="chunk-list-container">
                <div v-for="chunk in chunks" :key="chunk.id" class="chunk-item" :class="chunk.status">
                  <div class="chunk-info">
                    <strong>{{ $t('aistudio.chunk') }} {{ chunk.id + 1 }}</strong> ({{ $t('aistudio.lines') }} {{
                      chunk.id *
                    transConfig.batchSize + 1 }} - {{
                      Math.min((chunk.id + 1) * transConfig.batchSize, totalTranslationLines) }})
                  </div>
                  <div class="chunk-status">
                    <span v-if="chunk.status === 'pending'">{{ $t('aistudio.pending') }}</span>
                    <span v-if="chunk.status === 'processing'" class="processing">{{ $t('aistudio.translatingStatus')
                      }}</span>
                    <span v-if="chunk.status === 'success'" class="success">{{ $t('aistudio.success') }}</span>
                    <span v-if="chunk.status === 'error'" class="error">{{ $t('aistudio.errorStatus', {
                      error:
                      chunk.errorMessage }) }}</span>
                    <n-button v-if="chunk.status === 'error'" size="tiny" type="warning" @click="retryChunk(chunk)"
                      style="margin-left: 8px;">
                      {{ $t('aistudio.retry') }}
                    </n-button>
                  </div>
                </div>
              </div>
            </div>
          </n-card>
        </div>
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import {
  NCard, NForm, NFormItem, NSelect, NRadioGroup, NRadio, NSpace,
  NButton, NProgress, NStatistic, NEmpty, NTag, useMessage,
  NTabs, NTabPane, NInput, NInputNumber
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'
import { useProjectStore } from '../stores/project'
import { useModelsStore } from '../stores/models'
import { MediaService } from '../services/mediaService'
import { useTranslationStore } from '../stores/translation'
import { useProvidersStore } from '../stores/providers'
import { useSystemStore } from '../stores/system'
import { TranslationService, type TranslationChunk } from '../services/translationService'
import { SubtitleParser, type SubtitleLine } from '../utils/subtitleParser'
import { SubtitleManager } from '../services/subtitleManager'
import type { SubtitleVersion } from '../services/subtitleManager'
import { useSubtitleStore } from '../stores/subtitle'
import { useStylesStore } from '../stores/styles'

const message = useMessage()
const projectStore = useProjectStore()
const modelsStore = useModelsStore()
const translationStore = useTranslationStore()
const providersStore = useProvidersStore()
const systemStore = useSystemStore()
const subtitleStore = useSubtitleStore()
const stylesStore = useStylesStore()
const { t } = useI18n()

const config = ref({
  model: 'base',
  language: 'auto',
  device: 'cpu',
  format: 'srt'
})
const isRunning = ref(false)
const hasStarted = ref(false)
const progress = ref({ percent: 0, speed: 0, eta: 0, current_segment: '' })
const hasCuda = ref(false)
let unlisten: (() => void) | null = null

const availableVersions = ref<SubtitleVersion[]>([])
const isTranslating = ref(false)
const cancelTranslationFlag = ref(false)
const selectedSourcePath = ref<string | null>(null)

const sourceOptions = computed(() => {
  return availableVersions.value
    .filter(v => v.category === 'imported' || v.category === 'transcripted' || v.category === 'translated')
    .map(v => ({
      label: `${v.category.toUpperCase()} v${v.version} (${v.filename})`,
      value: v.path
    }))
})

const loadSubtitleVersions = async () => {
  if (!projectStore.currentProject) return
  availableVersions.value = await SubtitleManager.getProjectSubtitles(projectStore.currentProject.id)

  if (availableVersions.value.length > 0) {
    if (!selectedSourcePath.value || !availableVersions.value.some(v => v.path === selectedSourcePath.value)) {
      const valid = availableVersions.value.find(v => v.category === 'imported' || v.category === 'transcripted')
      selectedSourcePath.value = valid ? valid.path : null
    }
  } else {
    selectedSourcePath.value = null
  }
}

watch(() => projectStore.currentProject?.id, (newId) => {
  if (newId) {
    loadSubtitleVersions()
  }
}, { immediate: true })

const transConfig = ref({
  batchSize: 50
})
const chunks = ref<TranslationChunk[]>([])
const originalLines = ref<SubtitleLine[]>([])
const totalTranslationLines = ref(0)
const currentTranscriptionPrefix = ref('')

const modelOptions = [
  { label: 'Tiny (Fastest)', value: 'tiny' },
  { label: 'Base', value: 'base' },
  { label: 'Small', value: 'small' },
  { label: 'Medium', value: 'medium' },
  { label: 'Large-v3', value: 'large-v3' },
  { label: 'Large V3 Turbo', value: 'large-v3-turbo' }
]

const providerOptions = computed(() => {
  return providersStore.providers.map(p => ({
    label: p.has_key ? p.name : `${p.name} (No API Key)`,
    value: p.id,
    disabled: !p.has_key
  }))
})

watch(() => translationStore.provider, (newProviderId) => {
  const p = providersStore.providers.find(x => x.id === newProviderId)
  if (p && p.default_model) {
    translationStore.model = p.default_model
  }
})

const langOptions = [
  { label: 'Auto Detect', value: 'auto' },
  { label: 'Japanese', value: 'ja' },
  { label: 'English', value: 'en' },
  { label: 'Chinese', value: 'zh' },
  { label: 'Korean', value: 'ko' }
]

const targetLangOptions = [
  { label: 'English', value: 'en' },
  { label: 'Indonesian', value: 'id' }
]

const formatOptions = [
  { label: 'SRT', value: 'srt' },
  { label: 'ASS', value: 'ass' }
]

onMounted(async () => {
  await projectStore.fetchProjects()
  await modelsStore.fetchModels()
  await providersStore.fetchProviders()
  try {
    hasCuda.value = await invoke('check_cuda')
  } catch (e) {
    hasCuda.value = false
  }
  unlisten = await listen('transcription-event', async (event) => {
    try {
      const data = JSON.parse(event.payload as string)
      if (data.type === 'progress') {
        progress.value.percent = data.percent
        progress.value.speed = data.speed
        progress.value.eta = data.eta
        progress.value.current_segment = data.current_segment
      } else if (data.type === 'done') {
        isRunning.value = false
        systemStore.isTranscribing = false
        systemStore.setTask('Idle')
        message.success(t('aistudio.messages.transcriptionComplete'))
        progress.value.percent = 100
        progress.value.current_segment = 'Done.'

        if (projectStore.currentProject) {
          try {
            if (config.value.format === 'ass' && currentTranscriptionPrefix.value) {
              const srtPath = currentTranscriptionPrefix.value + '.srt'
              const assPath = currentTranscriptionPrefix.value + '.ass'

              const content = await invoke<string>('read_text_file', { path: srtPath })
              const parsedLines = SubtitleParser.parseSrt(content)

              if (stylesStore.styles.length === 0) {
                await stylesStore.loadStyles()
              }

              const assContent = SubtitleParser.serializeAss({
                scriptInfo: ['[Script Info]'],
                styles: {},
                styleFormat: ['Name', 'Fontname', 'Fontsize', 'PrimaryColour', 'SecondaryColour', 'OutlineColour', 'BackColour', 'Bold', 'Italic', 'Underline', 'StrikeOut', 'ScaleX', 'ScaleY', 'Spacing', 'Angle', 'BorderStyle', 'Outline', 'Shadow', 'Alignment', 'MarginL', 'MarginR', 'MarginV', 'Encoding'],
                eventsFormat: ['Layer', 'Start', 'End', 'Style', 'Name', 'MarginL', 'MarginR', 'MarginV', 'Effect', 'Text'],
                lines: parsedLines
              }, stylesStore.styles)

              await invoke('write_text_file', { path: assPath, content: assContent })
              await invoke('delete_file', { path: srtPath })
            }

            await loadSubtitleVersions()
            message.success(t('aistudio.messages.newTranscriptionReady'))
          } catch (e) {
            console.error('Failed to load subtitles:', e)
          }
        }
      } else if (data.type === 'error') {
        isRunning.value = false
        systemStore.isTranscribing = false
        systemStore.setTask('Idle')
        message.error(t('aistudio.messages.error') + data.message)
      } else if (data.type === 'info') {
        progress.value.current_segment = data.message
      }
    } catch (e) {
      console.error("Failed to parse event", e)
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})

const startTranscription = async () => {
  if (!projectStore.currentProject) {
    message.error(t('aistudio.messages.selectProject'))
    return
  }

  const selectedModel = modelsStore.models.find(m => m.id === config.value.model)
  if (!selectedModel || !selectedModel.is_installed) {
    message.error(t('aistudio.messages.modelNotInstalled', { model: config.value.model }))
    return
  }

  const project = projectStore.currentProject
  if (!project || !project.source_video) {
    message.error(t('aistudio.messages.noSourceVideo'))
    return
  }
  hasStarted.value = true
  isRunning.value = true
  systemStore.isTranscribing = true
  systemStore.setTask(t('aistudio.messages.processingTranscription'))
  progress.value = { percent: 0, speed: 0, eta: 0, current_segment: t('aistudio.messages.analyzingMedia') }
  try {
    const mediaInfo = await MediaService.analyzeMedia(project.source_video)

    const outPrefix = await SubtitleManager.getNextTranscriptionPrefix(project.id)
    currentTranscriptionPrefix.value = outPrefix

    await invoke('start_transcription_cmd', {
      projectId: project.id,
      inputPath: project.source_video,
      duration: mediaInfo.duration,
      model: config.value.model,
      language: config.value.language,
      device: config.value.device,
      format: config.value.format,
      customOutputPrefix: outPrefix
    })
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    isRunning.value = false
    systemStore.isTranscribing = false
    message.error(t('aistudio.messages.transcriptionFailed') + msg)
  }
}

const cancelTranscription = async () => {
  try {
    await invoke('cancel_transcription_cmd')
    isRunning.value = false
    systemStore.isTranscribing = false
    systemStore.setTask('Idle')
    progress.value.current_segment = t('aistudio.messages.cancelledByUser')
    message.warning(t('aistudio.messages.transcriptionCancelled'))
  } catch (e: any) {
    message.error(`Cancel failed: ${e}`)
  }
}

const formatTime = (seconds: number) => {
  if (seconds <= 0) return '0s'
  const m = Math.floor(seconds / 60)
  const s = Math.floor(seconds % 60)
  if (m > 0) return `${m}m ${s}s`
  return `${s}s`
}

const startTranslation = async () => {
  const project = projectStore.currentProject
  if (!project || !selectedSourcePath.value) {
    message.error(t('aistudio.messages.selectTarget'))
    return
  }

  isTranslating.value = true
  cancelTranslationFlag.value = false
  systemStore.isTranslating = true
  systemStore.setTask(t('aistudio.messages.translatingSubtitles'))
  try {
    const content = await invoke<string>('read_text_file', { path: selectedSourcePath.value })
    const isAss = selectedSourcePath.value.toLowerCase().endsWith('.ass')
    const parsedLines = isAss ? SubtitleParser.parseAss(content).lines : SubtitleParser.parseSrt(content)

    originalLines.value = JSON.parse(JSON.stringify(parsedLines))
    totalTranslationLines.value = originalLines.value.length

    chunks.value = TranslationService.createChunks(originalLines.value, transConfig.value.batchSize)

    for (const chunk of chunks.value) {
      if (cancelTranslationFlag.value) {
        message.warning(t('aistudio.messages.cancelledByUser') || 'Cancelled')
        break
      }
      await processChunk(chunk)
    }

    if (chunks.value.every(c => c.status === 'success')) {
      await saveTranslation()
    }

  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    message.error(t('aistudio.messages.translationStartFailed') + msg)
  } finally {
    isTranslating.value = false
    cancelTranslationFlag.value = false
    systemStore.isTranslating = false
    systemStore.setTask('Idle')
  }
}

const cancelTranslation = () => {
  cancelTranslationFlag.value = true
}

const handleTabLeave = () => {
  if (systemStore.isTranscribing || systemStore.isTranslating) {
    message.warning(t('aistudio.messages.waitProcess'))
    return false
  }
  return true
}

const processChunk = async (chunk: TranslationChunk) => {
  chunk.status = 'processing'
  try {
    const translatePromise = TranslationService.translateChunk(chunk)
    
    const checkCancel = new Promise<any>((_, reject) => {
      const interval = setInterval(() => {
        if (cancelTranslationFlag.value) {
          clearInterval(interval)
          reject(new Error(t('aistudio.messages.cancelledByUser') || 'Cancelled by user'))
        }
      }, 300)
      translatePromise.then(() => clearInterval(interval)).catch(() => clearInterval(interval))
    })

    const translated = await Promise.race([translatePromise, checkCancel])
    chunk.lines = translated
    chunk.status = 'success'
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    if (cancelTranslationFlag.value) {
      chunk.status = 'pending'
    } else {
      chunk.status = 'error'
      chunk.errorMessage = msg || 'Unknown error'
    }
  }
}

const retryChunk = async (chunk: TranslationChunk) => {
  isTranslating.value = true
  systemStore.isTranslating = true
  systemStore.setTask(t('aistudio.messages.translatingSubtitles'))

  await processChunk(chunk)

  if (chunks.value.every(c => c.status !== 'processing')) {
    isTranslating.value = false
    systemStore.isTranslating = false
    systemStore.setTask('Idle')

    if (chunks.value.every(c => c.status === 'success')) {
      await saveTranslation()
    }
  }
}

const saveTranslation = async () => {
  const project = projectStore.currentProject
  if (!project) return

  const finalLines = chunks.value.flatMap(c => c.lines)

  const ext = selectedSourcePath.value?.toLowerCase().endsWith('.ass') ? 'ass' : 'srt'
  let content = ''
  if (ext === 'ass') {
    if (stylesStore.styles.length === 0) {
      await stylesStore.loadStyles()
    }
    content = SubtitleParser.serializeAss({
      scriptInfo: ['[Script Info]'],
      styles: {},
      styleFormat: ['Name', 'Fontname', 'Fontsize', 'PrimaryColour', 'SecondaryColour', 'OutlineColour', 'BackColour', 'Bold', 'Italic', 'Underline', 'StrikeOut', 'ScaleX', 'ScaleY', 'Spacing', 'Angle', 'BorderStyle', 'Outline', 'Shadow', 'Alignment', 'MarginL', 'MarginR', 'MarginV', 'Encoding'],
      eventsFormat: ['Layer', 'Start', 'End', 'Style', 'Name', 'MarginL', 'MarginR', 'MarginV', 'Effect', 'Text'],
      lines: finalLines
    }, stylesStore.styles)
  } else {
    content = SubtitleParser.serializeSrt(finalLines)
  }

  try {
    const outPath = await SubtitleManager.saveNewSubtitle(project.id, 'translated', content, ext)
    await loadSubtitleVersions()

    await subtitleStore.loadFromFile(outPath)

    message.success(t('aistudio.messages.translationSaved'))
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e)
    message.error(t('aistudio.messages.translationSaveFailed') + msg)
  }
}

</script>

<style scoped>
.ai-view {
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
  grid-template-columns: 1fr 2fr;
  gap: 24px;
  margin-top: 16px;
}

@media (max-width: 1000px) {
  .content-grid {
    grid-template-columns: 1fr;
  }
}

.settings-card {
  background-color: #1e293b;
}

.progress-card {
  background-color: #1e293b;
}

.actions {
  margin-top: 24px;
}

.lang-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.lang-row .n-form-item {
  flex: 1;
}

.arrow {
  color: #64748b;
  font-weight: bold;
  margin-top: 34px;
  /* Align precisely with the input box below the label */
}

/* Queue Styles */
.queue-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.chunk-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background-color: #0f172a;
  border-radius: 6px;
  border-left: 4px solid #475569;
}

.chunk-item.processing {
  border-left-color: #3b82f6;
}

.chunk-item.success {
  border-left-color: #10b981;
}

.chunk-item.error {
  border-left-color: #ef4444;
}

.chunk-info {
  color: #e2e8f0;
}

.chunk-status span {
  font-size: 13px;
  font-weight: 600;
}

.processing {
  color: #3b82f6;
}

.success {
  color: #10b981;
}

.error {
  color: #ef4444;
}

/* Transcription styles from before */
.empty-state {
  padding: 60px 0;
}

.progress-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.stats-row {
  display: flex;
  justify-content: space-around;
  padding: 24px;
  background: linear-gradient(145deg, #1e293b, #0f172a);
  border: 1px solid rgba(99, 102, 241, 0.2);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  border-radius: 12px;
  position: relative;
  overflow: hidden;
}

.stats-row::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: linear-gradient(90deg, transparent, #6366f1, transparent);
}

.stats-row :deep(.n-statistic-value__content) {
  font-size: 28px;
  font-weight: 700;
  color: #f8fafc;
  text-shadow: 0 2px 10px rgba(99, 102, 241, 0.4);
}

.stats-row :deep(.n-statistic__label) {
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #94a3b8;
  font-weight: 600;
}

.progress-bar {
  margin: 4px 0;
}

.progress-bar :deep(.n-progress-graph-line-fill) {
  background: linear-gradient(90deg, #3b82f6, #8b5cf6) !important;
}

.live-caption-box {
  background: rgba(15, 23, 42, 0.6);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 20px;
  min-height: 110px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.live-caption-box.is-running {
  box-shadow: 0 0 20px rgba(99, 102, 241, 0.15);
  border-color: rgba(99, 102, 241, 0.3);
}

.live-caption-box.is-running::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  background: #6366f1;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% {
    opacity: 0.4;
  }

  50% {
    opacity: 1;
  }

  100% {
    opacity: 0.4;
  }
}

.caption-label {
  font-size: 12px;
  color: #64748b;
  text-transform: uppercase;
  font-weight: 700;
  letter-spacing: 1.5px;
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.caption-label i {
  display: inline-block;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #10b981;
}

.live-caption-box.is-running .caption-label i {
  animation: pulse 1s infinite;
}

.caption-text {
  font-size: 18px;
  color: #f8fafc;
  margin: 0;
  line-height: 1.6;
  font-weight: 500;
  font-family: monospace;
}

.control-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.chunk-list-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 370px;
  overflow-y: auto;
  padding-right: 8px;
}

/* Custom Scrollbar for container */
.chunk-list-container::-webkit-scrollbar {
  width: 6px;
}

.chunk-list-container::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.1);
  border-radius: 4px;
}

.chunk-list-container::-webkit-scrollbar-thumb {
  background: rgba(99, 102, 241, 0.4);
  border-radius: 4px;
}

.chunk-list-container::-webkit-scrollbar-thumb:hover {
  background: rgba(99, 102, 241, 0.8);
}
</style>
