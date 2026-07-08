import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useSystemStore = defineStore('system', () => {
  const cpuUsage = ref(0)
  const ramUsage = ref(0)
  const currentTask = ref('Idle')

  // Process States
  const isImporting = ref(false)
  const isTranscribing = ref(false)
  const isTranslating = ref(false)
  const isEncoding = ref(false)

  const isAppBusy = computed(() => {
    return isImporting.value || isTranscribing.value || isTranslating.value || isEncoding.value
  })

  const updateStats = (cpu: number, ram: number) => {
    cpuUsage.value = cpu
    ramUsage.value = ram
  }

  const setTask = (task: string) => {
    currentTask.value = task
  }

  return { 
    cpuUsage, ramUsage, currentTask, updateStats, setTask,
    isImporting, isTranscribing, isTranslating, isEncoding, isAppBusy
  }
})
