<template>
  <div class="workspace-view">
    <div v-if="!projectStore.currentProject">
      <n-empty :description="$t('workspace.noProject')" />
    </div>

    <div v-else-if="!projectStore.currentProject.source_video" class="empty-state">
      <div class="empty-content">
        <n-icon size="64" color="#6366f1">
          <FilmIcon />
        </n-icon>
        <h2>{{ $t('workspace.importVideo') }}</h2>
        <p>{{ $t('workspace.importDesc') }}</p>
        <n-button type="primary" size="large" @click="handleImport">
          {{ $t('workspace.selectVideo') }}
        </n-button>
        <p class="drop-hint">{{ $t('workspace.dropHint') }}</p>
      </div>
    </div>

    <div v-else class="workspace-tabs">
      <div class="workspace-header">
        <div class="header-left">
          <h2>{{ projectStore.currentProject.name }}</h2>
          <n-tag type="info">{{ projectStore.currentProject.project_type }}</n-tag>
        </div>
        <n-button @click="handleImport" :loading="systemStore.isImporting" :disabled="systemStore.isAppBusy" secondary>
          <template #icon>
            <n-icon>
              <FilmIcon />
            </n-icon>
          </template>
          {{ $t('workspace.changeMedia') }}
        </n-button>
      </div>

      <n-tabs type="line" animated class="flex-tabs" @before-leave="handleTabLeave" v-model:value="activeTab">
        <n-tab-pane name="overview" :tab="$t('workspace.tabs.overview')">
          <div class="overview-tab">
            <div v-if="systemStore.isImporting" class="importing-overlay">
              <div class="processing-content">
                <div class="waveform-animation">
                  <div class="bar bar1"></div>
                  <div class="bar bar2"></div>
                  <div class="bar bar3"></div>
                  <div class="bar bar4"></div>
                  <div class="bar bar5"></div>
                  <div class="bar bar6"></div>
                  <div class="bar bar7"></div>
                </div>
                <h3 class="shimmer-text">{{ $t('workspace.preparing') }}</h3>
                <p>{{ $t('workspace.generatingPreviews') }}</p>
                <n-progress type="line" :percentage="100" :show-indicator="false" status="info" processing />
              </div>
            </div>
            <MediaInfoPanel v-else-if="mediaInfo" :info="mediaInfo" />
            <n-empty v-else :description="$t('workspace.loadingMedia')" />
          </div>
        </n-tab-pane>

        <n-tab-pane name="ai" :tab="$t('workspace.tabs.aiStudio')">
          <AiStudio />
        </n-tab-pane>

        <n-tab-pane name="editor" :tab="$t('workspace.tabs.editor')">
          <Editor />
        </n-tab-pane>

        <n-tab-pane name="encoding" :tab="$t('workspace.tabs.encoding')">
          <Encoding />
        </n-tab-pane>
      </n-tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { NEmpty, NButton, NIcon, NTabs, NTabPane, NTag, useMessage, NProgress } from 'naive-ui'
import { FilmOutline as FilmIcon } from '@vicons/ionicons5'
import { useProjectStore } from '../stores/project'
import { useSystemStore } from '../stores/system'
import { open } from '@tauri-apps/plugin-dialog'
import { MediaService, type MediaInfo } from '../services/mediaService'
import MediaInfoPanel from '../components/MediaInfoPanel.vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { appDataDir, join } from '@tauri-apps/api/path'
import { invoke } from '@tauri-apps/api/core'
import { useSubtitleStore } from '../stores/subtitle'

import Editor from './Editor.vue'
import AiStudio from './AiStudio.vue'
import Encoding from './Encoding.vue'
import { useI18n } from 'vue-i18n'

const route = useRoute()
const projectStore = useProjectStore()
const systemStore = useSystemStore()
const subtitleStore = useSubtitleStore()
const message = useMessage()
const { t } = useI18n()
const mediaInfo = ref<MediaInfo | null>(null)
const activeTab = ref('overview')
let unlistenDrop: UnlistenFn | null = null

onMounted(async () => {
  if (!projectStore.currentProject && route.params.id) {
    if (projectStore.recentProjects.length === 0) {
      await projectStore.fetchProjects()
    }
    const proj = projectStore.recentProjects.find(p => p.id === route.params.id)
    if (proj) {
      projectStore.setProject(proj)
    }
  }

  if (projectStore.currentProject?.source_video) {
    loadMediaInfo(projectStore.currentProject.source_video)
  }

  const handleDrop = (event: any) => {
    if (projectStore.currentProject && !projectStore.currentProject.source_video) {
      const payload = event.payload as { paths: string[] };
      if (payload.paths && payload.paths.length > 0) {
        processImportFile(payload.paths[0]);
      }
    }
  };

  unlistenDrop = await listen('tauri://file-drop', handleDrop);
  const unlistenDragDrop = await listen('tauri://drag-drop', handleDrop);

  const origUnlisten = unlistenDrop;
  unlistenDrop = () => {
    origUnlisten();
    unlistenDragDrop();
  };
})

onUnmounted(() => {
  if (unlistenDrop) unlistenDrop()
})

watch(() => projectStore.currentProject?.source_video, (newVal) => {
  if (newVal) loadMediaInfo(newVal)
})

watch(() => projectStore.currentProject?.id, async (newId) => {
  if (newId) {
    try {
      const { SubtitleManager } = await import('../services/subtitleManager')
      const versions = await SubtitleManager.getProjectSubtitles(newId)
      if (versions.length > 0) {
        await subtitleStore.loadFromFile(versions[0].path)
      } else {
        const dir = await appDataDir()
        const srtPath = await join(dir, 'subtitles', `${newId}.srt`)
        try {
          await invoke('read_text_file', { path: srtPath })
          await subtitleStore.loadFromFile(srtPath)
        } catch (err) {
          subtitleStore.lines = []
        }
      }
    } catch (e) {
      console.warn("Failed to check project subtitle path", e)
    }
  } else {
    subtitleStore.lines = []
  }
}, { immediate: true })

const loadMediaInfo = async (path: string) => {
  try {
    mediaInfo.value = await MediaService.analyzeMedia(path)
  } catch (e) {
    console.error("Failed to load media info", e)
  }
}

const handleTabLeave = () => {
  if (systemStore.isAppBusy) {
    message.warning(t('workspace.messages.waitProcess'))
    return false // Block tab switch
  }
  return true
}

const handleImport = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Video',
        extensions: ['mp4', 'mkv', 'avi', 'mov', 'ts']
      }]
    })

    if (selected && !Array.isArray(selected)) {
      await processImportFile(selected)
    }
  } catch (e) {
    message.error(t('workspace.messages.failPicker'))
    console.error(e)
  }
}

const processImportFile = async (filePath: string) => {
  if (!projectStore.currentProject) return

  const ext = filePath.split('.').pop()?.toLowerCase()
  if (!['mp4', 'mkv', 'avi', 'mov', 'ts'].includes(ext || '')) {
    message.error(t('workspace.messages.unsupportedFormat'))
    return
  }

  const loadingMsg = message.loading(t('workspace.messages.analyzing'), { duration: 0 })
  try {
    const updatedProject = { ...projectStore.currentProject, source_video: filePath }
    await projectStore.updateProject(updatedProject)

    loadingMsg.destroy()
    message.success(t('workspace.messages.importSuccess'))

    activeTab.value = 'overview'
    systemStore.isImporting = true

    // Process async tasks
    await Promise.allSettled([
      MediaService.generateThumbnail(filePath, updatedProject.id),
      MediaService.generateWaveform(filePath, updatedProject.id)
    ])

    systemStore.isImporting = false
    message.success(t('workspace.messages.processingDone'))
  } catch (e) {
    systemStore.isImporting = false
    loadingMsg.destroy()
    message.error(t('workspace.messages.updateFail'))
  }
}
</script>

<style scoped>
.workspace-view {
  max-width: 100%;
  margin: 0 auto;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 60vh;
  background-color: #1e293b;
  border: 2px dashed #475569;
  border-radius: 12px;
}

.empty-content {
  text-align: center;
  color: #94a3b8;
}

.empty-content h2 {
  color: #f8fafc;
  margin: 16px 0 8px 0;
}

.empty-content p {
  margin-bottom: 24px;
}

.workspace-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 24px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.workspace-header h2 {
  margin: 0;
  font-size: 24px;
  color: #f8fafc;
}

.workspace-tabs {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.flex-tabs {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.flex-tabs :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  overflow: auto;
  height: 100%;
}

.flex-tabs :deep(.n-tabs-pane-wrapper > div) {
  height: 100%;
}

.flex-tabs :deep(.n-tab-pane) {
  height: 100%;
  width: 100%;
  box-sizing: border-box;
}

.overview-tab {
  padding: 24px 0;
}

.importing-overlay {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 400px;
  background-color: #1e293b;
  border-radius: 12px;
  border: 1px solid #334155;
  box-shadow: 0 10px 30px -10px rgba(0, 0, 0, 0.5);
  color: #f8fafc;
  text-align: center;
  position: relative;
  overflow: hidden;
}

@keyframes scanline {
  0% {
    left: -100%;
  }

  100% {
    left: 200%;
  }
}

.processing-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  max-width: 400px;
  width: 100%;
}

.waveform-animation {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  height: 60px;
  margin-bottom: 24px;
}

.waveform-animation .bar {
  width: 6px;
  border-radius: 4px;
  animation: wave 1.2s ease-in-out infinite;
}

.waveform-animation .bar1 {
  height: 20px;
  animation-delay: 0.0s;
  background-color: #818cf8;
}

.waveform-animation .bar2 {
  height: 35px;
  animation-delay: 0.1s;
  background-color: #6366f1;
}

.waveform-animation .bar3 {
  height: 50px;
  animation-delay: 0.2s;
  background-color: #4f46e5;
}

.waveform-animation .bar4 {
  height: 60px;
  animation-delay: 0.3s;
  background-color: #4338ca;
}

.waveform-animation .bar5 {
  height: 50px;
  animation-delay: 0.4s;
  background-color: #4f46e5;
}

.waveform-animation .bar6 {
  height: 35px;
  animation-delay: 0.5s;
  background-color: #6366f1;
}

.waveform-animation .bar7 {
  height: 20px;
  animation-delay: 0.6s;
  background-color: #818cf8;
}

@keyframes wave {

  0%,
  100% {
    transform: scaleY(0.4);
    opacity: 0.6;
  }

  50% {
    transform: scaleY(1);
    opacity: 1;
  }
}

.shimmer-text {
  margin: 0 0 12px 0;
  font-size: 24px;
  font-weight: 700;
  color: #f8fafc;
  background: linear-gradient(90deg,
      #f8fafc 0%,
      #f8fafc 40%,
      #818cf8 50%,
      #f8fafc 60%,
      #f8fafc 100%);
  background-size: 200% auto;
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  animation: shimmer 2.5s linear infinite;
}

@keyframes shimmer {
  to {
    background-position: 200% center;
  }
}

.processing-content p {
  color: #94a3b8;
  font-size: 14px;
  margin: 0 0 24px 0;
  line-height: 1.5;
}
</style>
