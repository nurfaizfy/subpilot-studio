import { defineStore } from 'pinia'
import { ref, version } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useProvidersStore } from './providers'

export interface DiagnosticItem {
  name: string
  status: string
  version?: string | null
  path?: string | null
  health: string
  error_msg?: string | null
  fix_suggestion?: string | null
}

export const useDiagnosticsStore = defineStore('diagnostics', () => {
  const isRunning = ref(false)
  const results = ref<DiagnosticItem[]>([])
  
  const runDiagnostics = async () => {
    isRunning.value = true
    results.value = []
    
    const vueItem: DiagnosticItem = {
      name: 'Vue Frontend',
      status: 'OK',
      version: version,
      health: 'healthy'
    }
    results.value.push(vueItem)
    
    let ipcOk = false
    try {
      const backendResults: DiagnosticItem[] = await invoke('run_diagnostics')
      ipcOk = true
      
      const ipcItem: DiagnosticItem = {
        name: 'IPC Connection',
        status: 'OK',
        health: 'healthy'
      }
      results.value.push(ipcItem)
      
      results.value.push(...backendResults)
    } catch (e) {
      const ipcItem: DiagnosticItem = {
        name: 'IPC Connection',
        status: 'ERROR',
        health: 'error',
        error_msg: String(e),
        fix_suggestion: 'Check if Tauri backend is running and properly initialized.'
      }
      results.value.push(ipcItem)
    }
    
    if (ipcOk) {
      const providersStore = useProvidersStore()
      await providersStore.fetchProviders()
      
      if (providersStore.providers.length > 0) {
        for (const provider of providersStore.providers) {
          try {
            const result = await providersStore.testConnection(provider.id)
            results.value.push({
              name: `AI Provider: ${provider.name}`,
              status: result.success ? 'OK' : (provider.has_key ? 'ERROR' : 'WARNING'),
              version: provider.default_model,
              health: result.success ? 'healthy' : (provider.has_key ? 'error' : 'warning'),
              error_msg: result.success ? null : `API Connection test failed: ${result.message}`,
              fix_suggestion: result.success ? null : 'Check your API Key and network settings.'
            })
          } catch (e) {
            results.value.push({
              name: `AI Provider: ${provider.name}`,
              status: 'ERROR',
              version: provider.default_model,
              health: 'error',
              error_msg: String(e),
              fix_suggestion: 'Check your API Key and network.'
            })
          }
        }
      } else {
        results.value.push({
          name: 'AI Provider API',
          status: 'WARNING',
          health: 'warning',
          error_msg: 'No AI Providers found in database.',
          fix_suggestion: 'Restart the application to initialize default providers.'
        })
      }
    }
    
    isRunning.value = false
  }

  const generateReport = () => {
    let report = '# SubPilot Studio Diagnostic Report\n\n'
    report += `Generated at: ${new Date().toISOString()}\n\n`
    
    results.value.forEach(item => {
      report += `## ${item.name}\n`
      report += `- **Status**: ${item.status}\n`
      report += `- **Health**: ${item.health}\n`
      if (item.version) report += `- **Version/Info**: ${item.version}\n`
      if (item.path) report += `- **Path**: ${item.path}\n`
      if (item.error_msg) report += `- **Error**: ${item.error_msg}\n`
      if (item.fix_suggestion) report += `- **Fix Suggestion**: ${item.fix_suggestion}\n`
      report += '\n'
    })
    
    return report
  }

  return {
    isRunning,
    results,
    runDiagnostics,
    generateReport
  }
})
