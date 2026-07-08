import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export interface AutoLog {
  time: string;
  message: string;
  type: 'info' | 'success' | 'error' | 'warning';
}

export const useAutomationStore = defineStore('automation', () => {
  const enabled = ref(false)
  const watchFolder = ref('')
  const whisperModel = ref('base')
  const translateTarget = ref('en')
  
  const isProcessing = ref(false)
  const history = ref<AutoLog[]>([])

  const addLog = (message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') => {
    const time = new Date().toLocaleTimeString()
    history.value.unshift({ time, message, type })
    if (history.value.length > 100) {
      history.value.pop()
    }
  }

  const loadSettings = () => {
    const saved = localStorage.getItem('subpilot-auto-settings')
    if (saved) {
      const parsed = JSON.parse(saved)
      if (parsed.enabled !== undefined) enabled.value = parsed.enabled
      if (parsed.watchFolder !== undefined) watchFolder.value = parsed.watchFolder
      if (parsed.whisperModel !== undefined) whisperModel.value = parsed.whisperModel
      if (parsed.translateTarget !== undefined) translateTarget.value = parsed.translateTarget
    }
  }

  const saveSettings = () => {
    localStorage.setItem('subpilot-auto-settings', JSON.stringify({
      enabled: enabled.value,
      watchFolder: watchFolder.value,
      whisperModel: whisperModel.value,
      translateTarget: translateTarget.value,
    }))
  }

  watch([enabled, watchFolder, whisperModel, translateTarget], () => {
    saveSettings()
  }, { deep: true })

  loadSettings()

  return {
    enabled,
    watchFolder,
    whisperModel,
    translateTarget,
    isProcessing,
    history,
    addLog
  }
})
