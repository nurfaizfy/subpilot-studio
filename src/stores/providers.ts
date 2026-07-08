import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ProviderConfig {
  id: string
  name: string
  api_url: string
  default_model: string
  is_fallback: boolean
  priority: number
  has_key: boolean
}

export interface TestResult {
  success: boolean
  latency_ms: number
  message: string
}

export const useProvidersStore = defineStore('providers', () => {
  const providers = ref<ProviderConfig[]>([])
  
  const fetchProviders = async () => {
    try {
      providers.value = await invoke('get_providers')
    } catch (e) {
      console.error(e)
    }
  }

  const saveProvider = async (config: ProviderConfig, api_key: string) => {
    try {
      await invoke('save_provider', { config, apiKey: api_key, api_key: api_key })
      await fetchProviders()
    } catch (e) {
      console.error("Failed to save provider:", e)
      throw e
    }
  }


  const testConnection = async (providerId: string): Promise<TestResult> => {
    try {
      return await invoke('test_connection', { providerId })
    } catch (e) {
      console.error(e)
      return { success: false, latency_ms: 0, message: String(e) }
    }
  }

  return {
    providers,
    fetchProviders,
    saveProvider,
    testConnection
  }
})
