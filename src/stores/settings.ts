import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  const language = ref('en-US')
  const theme = ref('dark')
  const workspaceDir = ref('')
  const autoSave = ref(true)
  const autoBackup = ref(false)

  const loadSettings = () => {
    const saved = localStorage.getItem('subpilot-settings')
    if (saved) {
      const parsed = JSON.parse(saved)
      if (parsed.language !== undefined) language.value = parsed.language
      if (parsed.theme !== undefined) theme.value = parsed.theme
      if (parsed.workspaceDir !== undefined) workspaceDir.value = parsed.workspaceDir
      if (parsed.autoSave !== undefined) autoSave.value = parsed.autoSave
      if (parsed.autoBackup !== undefined) autoBackup.value = parsed.autoBackup
    }
  }

  const saveSettings = () => {
    localStorage.setItem('subpilot-settings', JSON.stringify({
      language: language.value,
      theme: theme.value,
      workspaceDir: workspaceDir.value,
      autoSave: autoSave.value,
      autoBackup: autoBackup.value
    }))
  }

  watch([
    language, theme, workspaceDir, autoSave, autoBackup
  ], () => {
    saveSettings()
  }, { deep: true })

  loadSettings()

  return {
    language,
    theme,
    workspaceDir,
    autoSave,
    autoBackup
  }
})
