import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface RuntimeStatus {
  ffmpeg_ok: boolean;
  ffprobe_ok: boolean;
  models_ok: boolean;
  whisper_ok: boolean;
}

export const useRuntimeStore = defineStore('runtime', () => {
  const status = ref<RuntimeStatus>({
    ffmpeg_ok: false,
    ffprobe_ok: false,
    models_ok: false,
    whisper_ok: false
  })
  
  const isChecked = ref(false)
  const isAllGood = ref(false)
  
  const checkRuntime = async () => {
    try {
      const res: RuntimeStatus = await invoke('check_runtime')
      status.value = res
      
      isAllGood.value = res.ffmpeg_ok && res.ffprobe_ok && res.whisper_ok
      isChecked.value = true
    } catch (e) {
      console.error("Failed to check runtime", e)
    }
  }
  
  const repairRuntime = async () => {
    try {
      await invoke('repair_runtime')
      await checkRuntime()
    } catch (e) {
      console.error("Failed to repair runtime", e)
      throw e
    }
  }

  return { status, isChecked, isAllGood, checkRuntime, repairRuntime }
})
