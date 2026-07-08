<template>
  <div class="models-view">
    <div class="header">
      <h2>AI Model Manager</h2>
      <p>Download and manage Faster-Whisper models for local speech recognition.</p>
    </div>

    <div class="model-grid">
      <n-card v-for="model in modelsStore.models" :key="model.id" class="model-card">
        <template #header>
          <div class="card-title">
            <span class="name">{{ model.name }}</span>
            <n-tag :type="model.is_installed ? 'success' : 'default'" size="small">
              {{ model.is_installed ? 'Installed' : 'Not Installed' }}
            </n-tag>
          </div>
        </template>
        
        <div class="card-content">
          <div class="info-row">
            <span class="label">Download Size:</span>
            <span class="value">{{ model.size_mb }} MB</span>
          </div>
          <div class="info-row" v-if="model.is_installed">
            <span class="label">Disk Usage:</span>
            <span class="value">{{ model.disk_usage_mb.toFixed(1) }} MB</span>
          </div>
          
          <div class="progress-area" v-if="getProgress(model.id) && getProgress(model.id)!.status !== 'completed'">
            <div class="status-text">
              {{ getStatusText(getProgress(model.id)!.status) }}
              <span v-if="getProgress(model.id)!.status === 'downloading'">
                - {{ getProgress(model.id)!.speed_mbps.toFixed(1) }} MB/s (ETA: {{ formatETA(getProgress(model.id)!.eta_seconds) }})
              </span>
            </div>
            <n-progress 
              type="line" 
              :percentage="getProgress(model.id)!.percent" 
              :status="getProgress(model.id)!.status === 'error' ? 'error' : getProgress(model.id)!.status === 'paused' ? 'warning' : 'success'"
              :processing="getProgress(model.id)!.status === 'downloading'"
            />
            
            <div class="progress-actions">
              <n-button 
                v-if="getProgress(model.id)!.status === 'downloading'" 
                size="small" 
                @click="modelsStore.pauseDownload(model.id)"
              >
                Pause
              </n-button>
              <n-button 
                v-if="getProgress(model.id)!.status === 'paused'" 
                size="small" 
                type="primary"
                @click="modelsStore.resumeDownload(model.id)"
              >
                Resume
              </n-button>
              <n-button 
                size="small" 
                type="error" ghost
                @click="modelsStore.cancelDownload(model.id)"
              >
                Cancel
              </n-button>
            </div>
          </div>
        </div>

        <template #footer>
          <div class="actions">
            <n-button 
              v-if="!model.is_installed && !getProgress(model.id)" 
              type="primary" 
              block 
              @click="modelsStore.startDownload(model)"
            >
              <template #icon>
                <n-icon><DownloadOutline /></n-icon>
              </template>
              Download
            </n-button>
            
            <n-button 
              v-if="model.is_installed" 
              type="error" 
              ghost 
              block 
              @click="modelsStore.deleteModel(model.id)"
            >
              <template #icon>
                <n-icon><TrashOutline /></n-icon>
              </template>
              Delete
            </n-button>
          </div>
        </template>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { NCard, NTag, NButton, NProgress, NIcon } from 'naive-ui'
import { DownloadOutline, TrashOutline } from '@vicons/ionicons5'
import { useModelsStore, type DownloadProgressEvent } from '../stores/models'

const modelsStore = useModelsStore()

onMounted(() => {
  modelsStore.fetchModels()
})

const getProgress = (id: string): DownloadProgressEvent | undefined => {
  return modelsStore.downloadProgress[id]
}

const getStatusText = (status: string) => {
  switch (status) {
    case 'starting': return 'Starting download...'
    case 'downloading': return 'Downloading'
    case 'paused': return 'Paused'
    case 'verifying': return 'Verifying checksum...'
    case 'extracting': return 'Extracting files...'
    case 'completed': return 'Completed'
    case 'error': return 'Error downloading'
    default: return status
  }
}

const formatETA = (seconds: number) => {
  if (!seconds || seconds === Infinity) return '0s'
  if (seconds < 60) return `${Math.round(seconds)}s`
  const m = Math.floor(seconds / 60)
  const s = Math.round(seconds % 60)
  return `${m}m ${s}s`
}
</script>

<style scoped>
.models-view {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.header h2 {
  margin: 0 0 8px 0;
  color: #f8fafc;
}

.header p {
  margin: 0;
  color: #94a3b8;
}

.model-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.model-card {
  background-color: #1e293b;
  border-color: #334155;
}

.card-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-title .name {
  font-weight: 600;
  font-size: 16px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 14px;
}

.info-row .label {
  color: #94a3b8;
}

.progress-area {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #334155;
}

.status-text {
  font-size: 12px;
  color: #cbd5e1;
  margin-bottom: 8px;
}

.progress-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}
</style>
