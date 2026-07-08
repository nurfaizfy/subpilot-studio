import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export interface GlossaryEntry {
  source: string;
  target: string;
}

export const useTranslationStore = defineStore('translation', () => {
  const provider = ref('gemini')
  const model = ref('gemini-1.5-flash')
  const apiKey = ref('')
  
  const sourceLang = ref('ja')
  const targetLang = ref('en')
  const customPrompt = ref('')
  
  const glossary = ref<GlossaryEntry[]>([])

  const loadSettings = () => {
    const saved = localStorage.getItem('subpilot-translation-settings')
    if (saved) {
      const parsed = JSON.parse(saved)
      if (parsed.provider) provider.value = parsed.provider
      if (parsed.model) model.value = parsed.model
      if (parsed.apiKey) apiKey.value = parsed.apiKey
      if (parsed.sourceLang) sourceLang.value = parsed.sourceLang
      if (parsed.targetLang) targetLang.value = parsed.targetLang
      if (parsed.customPrompt !== undefined) customPrompt.value = parsed.customPrompt
      if (parsed.glossary) glossary.value = parsed.glossary
    }
  }

  const saveSettings = () => {
    localStorage.setItem('subpilot-translation-settings', JSON.stringify({
      provider: provider.value,
      model: model.value,
      apiKey: apiKey.value,
      sourceLang: sourceLang.value,
      targetLang: targetLang.value,
      customPrompt: customPrompt.value,
      glossary: glossary.value
    }))
  }

  watch([provider, model, apiKey, sourceLang, targetLang, customPrompt, glossary], () => {
    saveSettings()
  }, { deep: true })

  loadSettings()

  return {
    provider,
    model,
    apiKey,
    sourceLang,
    targetLang,
    customPrompt,
    glossary,
    loadSettings
  }
})
