<template>
  <div class="automation-view">
    <div class="header">
      <h2>Watch Folder Automation</h2>
      <p class="subtitle">Automatically process videos dropped into a folder</p>
    </div>

    <div class="content-grid">
      <n-card class="settings-card" title="Automation Rules">
        <n-form :model="autoStore" label-placement="top">
          
          <div class="master-switch">
            <n-switch v-model:value="autoStore.enabled" size="large" />
            <div class="switch-label">
              <div class="title" :class="{ active: autoStore.enabled }">
                {{ autoStore.enabled ? 'Automation is ACTIVE' : 'Automation is STOPPED' }}
              </div>
              <div class="desc">
                When active, SubPilot will check the Watch Folder every 10 seconds.
              </div>
            </div>
          </div>

          <n-form-item label="Watch Folder Path">
            <n-input v-model:value="autoStore.watchFolder" placeholder="C:/Downloads/Auto_Subs" :disabled="autoStore.enabled" />
          </n-form-item>

          <n-form-item label="Speech Recognition Model">
            <n-select v-model:value="autoStore.whisperModel" :options="whisperOptions" :disabled="autoStore.enabled" />
          </n-form-item>

          <n-form-item label="Auto Translate Target">
            <n-select v-model:value="autoStore.translateTarget" :options="languageOptions" :disabled="autoStore.enabled" />
          </n-form-item>

        </n-form>
      </n-card>

      <n-card class="log-card" title="Activity Log">
        <template #header-extra>
          <n-tag v-if="autoStore.isProcessing" type="warning" size="small">
            Processing...
          </n-tag>
          <n-tag v-else type="success" size="small">
            Idle
          </n-tag>
        </template>
        
        <div class="log-window">
          <div v-if="autoStore.history.length === 0" class="empty-log">
            No activity yet.
          </div>
          <div 
            v-for="(log, index) in autoStore.history" 
            :key="index"
            class="log-entry"
            :class="log.type"
          >
            <span class="time">[{{ log.time }}]</span>
            <span class="msg">{{ log.message }}</span>
          </div>
        </div>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { 
  NCard, NForm, NFormItem, NInput, NSelect, NSwitch, NTag
} from 'naive-ui'
import { useAutomationStore } from '../stores/automation'

const autoStore = useAutomationStore()

const whisperOptions = [
  { label: 'Tiny', value: 'tiny' },
  { label: 'Base', value: 'base' },
  { label: 'Small', value: 'small' },
  { label: 'Medium', value: 'medium' }
]

const languageOptions = [
  { label: 'Indonesian', value: 'id' },
  { label: 'English', value: 'en' }
]
</script>

<style scoped>
.automation-view {
  max-width: 100%;
  margin: 0 auto;
}

.header {
  margin-bottom: 24px;
}

.header h2 {
  font-size: 28px;
  margin: 0;
  color: #f8fafc;
}

.subtitle {
  color: #94a3b8;
  margin: 4px 0 0 0;
}

.content-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.settings-card, .log-card {
  background-color: #1e293b;
  border-radius: 8px;
}

.master-switch {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background-color: #0f172a;
  border-radius: 8px;
  margin-bottom: 24px;
  border: 1px solid #334155;
}

.switch-label .title {
  font-weight: 600;
  color: #94a3b8;
  font-size: 16px;
}

.switch-label .title.active {
  color: #10b981;
}

.switch-label .desc {
  font-size: 12px;
  color: #64748b;
  margin-top: 2px;
}

.log-window {
  background-color: #020617;
  border-radius: 6px;
  padding: 12px;
  height: 400px;
  overflow-y: auto;
  font-family: monospace;
}

.empty-log {
  color: #475569;
  text-align: center;
  margin-top: 40px;
}

.log-entry {
  margin-bottom: 6px;
  font-size: 13px;
  line-height: 1.4;
}

.log-entry .time {
  color: #64748b;
  margin-right: 8px;
}

.log-entry.info .msg { color: #e2e8f0; }
.log-entry.success .msg { color: #10b981; }
.log-entry.error .msg { color: #ef4444; }
.log-entry.warning .msg { color: #f59e0b; }
</style>
