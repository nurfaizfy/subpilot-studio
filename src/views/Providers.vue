<template>
  <div class="providers-view">
    <div class="header">
      <h2>AI Provider Manager</h2>
      <p>Configure Language Models for translation. Enable Fallback to automatically switch providers if the primary one fails.</p>
    </div>

    <div class="provider-grid">
      <n-card v-for="provider in providersStore.providers" :key="provider.id" class="provider-card">
        <template #header>
          <div class="card-title">
            <span class="name">{{ provider.name }}</span>
            <n-tag :type="provider.has_key ? 'success' : 'error'" size="small">
              {{ provider.has_key ? 'Key Set' : 'Missing Key' }}
            </n-tag>
          </div>
        </template>
        
        <div class="card-content">
          <div class="info-row">
            <span class="label">Default Model:</span>
            <span class="value">{{ provider.default_model }}</span>
          </div>
          <div class="info-row">
            <span class="label">Fallback Enabled:</span>
            <n-switch :value="provider.is_fallback" @update:value="val => toggleFallback(provider, val)" size="small" />
          </div>
          
          <div class="test-area" v-if="testResults[provider.id]">
            <n-tag :type="testResults[provider.id].success ? 'success' : 'error'" size="small">
              {{ testResults[provider.id].message }}
              <span v-if="testResults[provider.id].success">({{ testResults[provider.id].latency_ms }}ms)</span>
            </n-tag>
          </div>
        </div>

        <template #footer>
          <div class="actions">
            <n-button type="default" block @click="openConfig(provider)">
              Configure
            </n-button>
            <n-button 
              type="primary" 
              ghost 
              block 
              :loading="isTesting[provider.id]"
              @click="testProvider(provider.id)"
            >
              Test Connection
            </n-button>
          </div>
        </template>
      </n-card>
    </div>

    <n-modal v-model:show="showConfig">
      <n-card
        style="width: 500px"
        :title="`Configure ${editingProvider?.name}`"
        :bordered="false"
        size="huge"
        role="dialog"
        aria-modal="true"
      >
        <n-form v-if="editingProvider">
          <n-form-item label="API URL">
            <n-input v-model:value="editForm.api_url" />
          </n-form-item>
          <n-form-item label="Default Model">
            <n-input v-model:value="editForm.default_model" />
          </n-form-item>
          <n-form-item label="API Key">
            <n-input 
              v-model:value="apiKeyInput" 
              type="password" 
              show-password-on="click" 
              placeholder="Leave blank to keep current key"
            />
          </n-form-item>
        </n-form>
        
        <template #footer>
          <div style="display: flex; justify-content: flex-end; gap: 12px;">
            <n-button @click="showConfig = false">Cancel</n-button>
            <n-button type="primary" @click="saveConfig" :loading="isSaving">Save Securely</n-button>
          </div>
        </template>
      </n-card>
    </n-modal>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NCard, NTag, NButton, NSwitch, NModal, NForm, NFormItem, NInput, useMessage } from 'naive-ui'
import { useProvidersStore, type ProviderConfig, type TestResult } from '../stores/providers'

const providersStore = useProvidersStore()
const message = useMessage()

const testResults = ref<Record<string, TestResult>>({})
const isTesting = ref<Record<string, boolean>>({})

const showConfig = ref(false)
const isSaving = ref(false)
const editingProvider = ref<ProviderConfig | null>(null)
const editForm = ref({
  api_url: '',
  default_model: '',
  is_fallback: false
})
const apiKeyInput = ref('')

onMounted(async () => {
  await providersStore.fetchProviders()
})

const testProvider = async (id: string) => {
  isTesting.value[id] = true
  const res = await providersStore.testConnection(id)
  testResults.value[id] = res
  isTesting.value[id] = false
}

const toggleFallback = async (provider: ProviderConfig, val: boolean) => {
  const newConfig = { ...provider, is_fallback: val }
  await providersStore.saveProvider(newConfig, "")
}

const openConfig = (provider: ProviderConfig) => {
  editingProvider.value = provider
  editForm.value = {
    api_url: provider.api_url,
    default_model: provider.default_model,
    is_fallback: provider.is_fallback
  }
  apiKeyInput.value = ''
  showConfig.value = true
}

const saveConfig = async () => {
  if (!editingProvider.value) return
  isSaving.value = true
  
  const updatedConfig: ProviderConfig = {
    ...editingProvider.value,
    ...editForm.value
  }
  
  try {
    await providersStore.saveProvider(updatedConfig, apiKeyInput.value)
    showConfig.value = false
    message.success('Provider settings saved successfully')
  } catch(e) {
    message.error('Failed to save. Error: ' + e)
  }
  
  isSaving.value = false
}
</script>

<style scoped>
.providers-view {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.header h2 {
  margin: 0 0 8px 0;
  color: #f8fafc;
}

.header p {
  margin: 0;
  color: #94a3b8;
}

.provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 20px;
}

.provider-card {
  background-color: #1e293b;
  border-color: #334155;
}

.card-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-title .name {
  font-weight: 600;
  font-size: 16px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  font-size: 14px;
}

.info-row .label {
  color: #94a3b8;
}

.info-row .value {
  color: #cbd5e1;
  font-family: monospace;
}

.test-area {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid #334155;
  text-align: center;
}

.actions {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}
</style>
