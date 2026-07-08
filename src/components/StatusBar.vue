<template>
  <footer class="statusbar">
    <div class="status-left">
      <n-icon size="16" :class="{ 'spin': isBusy }">
        <SyncIcon v-if="isBusy" />
        <CheckmarkIcon v-else />
      </n-icon>
      <span class="task-text">{{ displayTask }}</span>
    </div>
    
    <div class="status-right">
      <div class="stat-item">
        <n-icon size="16"><HardwareIcon /></n-icon>
        <span>CPU: {{ systemStore.cpuUsage }}%</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <n-icon size="16"><HardwareIcon /></n-icon>
        <span>RAM: {{ systemStore.ramUsage }}MB</span>
      </div>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { NIcon } from 'naive-ui'
import { 
  SyncOutline as SyncIcon, 
  CheckmarkCircleOutline as CheckmarkIcon,
  HardwareChipOutline as HardwareIcon
} from '@vicons/ionicons5'
import { useSystemStore } from '../stores/system'
import { useEncoderStore } from '../stores/encoder'
import { invoke } from '@tauri-apps/api/core'

const systemStore = useSystemStore()
const encoderStore = useEncoderStore()
const { t } = useI18n()
let intervalId: number;

const isBusy = computed(() => {
  return systemStore.isImporting || systemStore.isTranscribing || systemStore.isTranslating || systemStore.isEncoding || systemStore.currentTask !== 'Idle'
})

const displayTask = computed(() => {
  if (systemStore.isImporting) return t('statusbar.importing')
  if (systemStore.isTranscribing) return t('statusbar.transcribing')
  if (systemStore.isTranslating) return t('statusbar.translating')
  if (systemStore.isEncoding) {
    if (encoderStore.activeJobId) {
      const job = encoderStore.queue.find(j => j.id === encoderStore.activeJobId)
      if (job) return t('statusbar.encodingPercent', { percent: Math.round(job.percent) })
    }
    return t('statusbar.encoding')
  }
  return systemStore.currentTask === 'Idle' ? 'Idle' : systemStore.currentTask
})

onMounted(() => {
  intervalId = window.setInterval(async () => {
    try {
      const stats: any = await invoke('get_system_stats')
      systemStore.updateStats(stats.cpu, stats.ram)
    } catch (e) {
      console.error('Failed to fetch system stats', e)
    }
  }, 2000)
})

onUnmounted(() => {
  clearInterval(intervalId)
})
</script>

<style scoped>
.statusbar {
  height: 32px;
  background-color: #0f172a;
  border-top: 1px solid #334155;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 16px;
  font-size: 12px;
  color: #94a3b8;
}

.status-left, .status-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
}

.stat-divider {
  width: 1px;
  height: 12px;
  background-color: #334155;
  margin: 0 4px;
}

.spin {
  animation: spin 2s linear infinite;
}

@keyframes spin {
  100% {
    transform: rotate(360deg);
  }
}
</style>
