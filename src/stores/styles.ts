import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AssStyle } from '../utils/subtitleParser'

export const useStylesStore = defineStore('styles', () => {
  const styles = ref<AssStyle[]>([])
  
  const loadStyles = async () => {
    try {
      styles.value = await invoke<AssStyle[]>('get_ass_styles')
    } catch (e) {
      console.error("Failed to load ASS styles:", e)
    }
  }

  const saveStyle = async (style: AssStyle) => {
    try {
      await invoke('save_ass_style', { style })
      await loadStyles()
    } catch (e) {
      console.error("Failed to save ASS style:", e)
      throw e
    }
  }

  const deleteStyle = async (name: string) => {
    try {
      await invoke('delete_ass_style', { name })
      await loadStyles()
    } catch (e) {
      console.error("Failed to delete ASS style:", e)
      throw e
    }
  }

  return {
    styles,
    loadStyles,
    saveStyle,
    deleteStyle
  }
})
