import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface AIModel {
  id: string
  name: string
  size_mb: number
  is_installed: boolean
  disk_usage_mb: number
  download_url: string
}

export interface DownloadProgressEvent {
  id: string
  percent: number
  speed_mbps: number
  eta_seconds: number
  status: string
}

export const useModelsStore = defineStore('models', () => {
  const models = ref<AIModel[]>([])
  const downloadProgress = ref<Record<string, DownloadProgressEvent>>({})

  const fetchModels = async () => {
    try {
      models.value = await invoke('get_available_models')
    } catch (e) {
      console.error(e)
    }
  }

  const startDownload = async (model: AIModel) => {
    try {
      downloadProgress.value[model.id] = {
        id: model.id,
        percent: 0,
        speed_mbps: 0,
        eta_seconds: 0,
        status: 'starting'
      }
      await invoke('start_model_download', { id: model.id, url: model.download_url })
    } catch (e) {
      console.error(e)
    }
  }

  const pauseDownload = async (id: string) => {
    await invoke('pause_model_download', { id })
  }

  const resumeDownload = async (id: string) => {
    await invoke('resume_model_download', { id })
  }

  const cancelDownload = async (id: string) => {
    await invoke('cancel_model_download', { id })
    delete downloadProgress.value[id]
  }

  const deleteModel = async (id: string) => {
    await invoke('delete_model', { id })
    await fetchModels()
  }

  let listenerSetup = false
  const initListener = async () => {
    if (listenerSetup) return
    listenerSetup = true
    await listen<DownloadProgressEvent>('model-download-progress', (event) => {
      downloadProgress.value[event.payload.id] = event.payload
      if (event.payload.status === 'completed') {
        fetchModels()
      }
    })
  }

  initListener()

  return {
    models,
    downloadProgress,
    fetchModels,
    startDownload,
    pauseDownload,
    resumeDownload,
    cancelDownload,
    deleteModel
  }
})
