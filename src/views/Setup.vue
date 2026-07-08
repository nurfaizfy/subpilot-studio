<template>
  <div class="setup-view">
    <div class="setup-container">
      <div class="header">
        <div class="logo-circle">
          <n-icon size="40">
            <HardwareChipOutline />
          </n-icon>
        </div>
        <h2>Welcome to SubPilot Studio</h2>
        <p>Let's get your environment ready before we start.</p>
      </div>

      <div class="checklist">
        <div class="check-item" :class="{ ok: runtimeStore.status.ffmpeg_ok }">
          <div class="info">
            <h4>FFmpeg Engine</h4>
            <p>Required for encoding and video processing</p>
            <div v-if="activeRepairs['ffmpeg']" class="progress-info">
              <span class="status-text">{{ activeRepairs['ffmpeg'] === 'downloading' ? 'Downloading...' : 'Extracting...' }}</span>
              <n-progress type="line" indeterminate size="small" status="info" :show-indicator="false" style="width: 100%" />
            </div>
          </div>
          <n-icon size="24" :color="runtimeStore.status.ffmpeg_ok ? '#10b981' : '#ef4444'">
            <CheckmarkCircle v-if="runtimeStore.status.ffmpeg_ok" />
            <CloseCircle v-else />
          </n-icon>
        </div>

        <div class="check-item" :class="{ ok: runtimeStore.status.whisper_ok }">
          <div class="info">
            <h4>Whisper Engine</h4>
            <p>Required for AI transcription</p>
            <div v-if="activeRepairs['whisper']" class="progress-info">
              <span class="status-text">{{ activeRepairs['whisper'] === 'downloading' ? 'Downloading...' : 'Extracting...' }}</span>
              <n-progress type="line" indeterminate size="small" status="info" :show-indicator="false" style="width: 100%" />
            </div>
          </div>
          <n-icon size="24" :color="runtimeStore.status.whisper_ok ? '#10b981' : '#ef4444'">
            <CheckmarkCircle v-if="runtimeStore.status.whisper_ok" />
            <CloseCircle v-else />
          </n-icon>
        </div>
      </div>

      <div class="actions">
        <n-button v-if="!isReady" type="primary" size="large" block :loading="isRepairing" @click="handleRepair">
          <template #icon>
            <n-icon>
              <DownloadOutline />
            </n-icon>
          </template>
          Download Missing
        </n-button>

        <n-button v-if="isReady" type="primary" size="large" block @click="startApp">
          Launch SubPilot
        </n-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NIcon, NProgress } from 'naive-ui'
import {
  HardwareChipOutline, CheckmarkCircle, CloseCircle, DownloadOutline
} from '@vicons/ionicons5'
import { useRuntimeStore } from '../stores/runtime'
import { listen } from '@tauri-apps/api/event'

const router = useRouter()
const runtimeStore = useRuntimeStore()

const isRepairing = ref(false)
const activeRepairs = ref<Record<string, string>>({})

const isDependenciesOk = computed(() => {
  return runtimeStore.status.ffmpeg_ok && runtimeStore.status.whisper_ok
})

const isReady = computed(() => {
  return isDependenciesOk.value
})

onMounted(async () => {
  await runtimeStore.checkRuntime()
  
  await listen('repair-progress', (event: any) => {
    const { component, status } = event.payload
    if (status === 'done' || status === 'error') {
      delete activeRepairs.value[component]
    } else {
      activeRepairs.value[component] = status
    }
  })
})

const handleRepair = async () => {
  isRepairing.value = true
  try {
    await runtimeStore.repairRuntime()
  } finally {
    isRepairing.value = false
  }
}

const startApp = () => {
  router.push('/')
}
</script>

<style scoped>
.setup-view {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  background-color: #0f172a;
}

.setup-container {
  width: 100%;
  max-width: 500px;
  background-color: #1e293b;
  padding: 40px;
  border-radius: 12px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
  border: 1px solid #334155;
}

.header {
  text-align: center;
  margin-bottom: 32px;
}

.logo-circle {
  width: 80px;
  height: 80px;
  background: linear-gradient(135deg, #3b82f6, #2dd4bf);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 0 auto 16px;
  color: white;
}

.header h2 {
  font-size: 24px;
  color: #f8fafc;
  margin: 0 0 8px 0;
}

.header p {
  color: #94a3b8;
  margin: 0;
}

.checklist {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 32px;
}

.check-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  background-color: #0f172a;
  border-radius: 8px;
  border: 1px solid #334155;
  transition: all 0.3s;
}

.check-item.ok {
  border-color: #10b981;
  background-color: rgba(16, 185, 129, 0.05);
}

.check-item .info {
  flex: 1;
  padding-right: 16px;
}

.check-item .info h4 {
  margin: 0 0 4px 0;
  color: #e2e8f0;
}

.check-item .info p {
  margin: 0;
  font-size: 12px;
  color: #64748b;
}

.actions {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-info {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.status-text {
  font-size: 11px;
  color: #3b82f6;
  font-weight: 500;
}
</style>
