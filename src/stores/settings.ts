import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import i18n from '../i18n'

export const useSettingsStore = defineStore('settings', () => {
  const language = ref('en-US')
  const autoSave = ref(true)
  const autoBackup = ref(false)

  const loadSettings = () => {
    const saved = localStorage.getItem('subpilot-settings')
    if (saved) {
      const parsed = JSON.parse(saved)
      if (parsed.language !== undefined) {
        language.value = parsed.language
        ;(i18n.global.locale as any).value = parsed.language
      }
      if (parsed.autoSave !== undefined) autoSave.value = parsed.autoSave
      if (parsed.autoBackup !== undefined) autoBackup.value = parsed.autoBackup
    }
  }

  const saveSettings = () => {
    localStorage.setItem('subpilot-settings', JSON.stringify({
      language: language.value,
      autoSave: autoSave.value,
      autoBackup: autoBackup.value
    }))
  }

  watch([
    language, autoSave, autoBackup
  ], () => {
    saveSettings()
    ;(i18n.global.locale as any).value = language.value
  }, { deep: true })

  loadSettings()

  return {
    language,
    autoSave,
    autoBackup
  }
})
