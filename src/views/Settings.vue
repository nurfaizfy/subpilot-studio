<template>
  <div class="settings-view">
    <div class="header">
      <h2>Settings</h2>
      <p class="subtitle">Configure application paths and integrations</p>
    </div>

    <n-tabs type="line" class="settings-tabs">
      <n-tab-pane name="general" tab="General Preferences">
        <div class="content-grid">
          <div class="column">
            <n-card class="settings-card" title="General Preferences" style="margin-top: 24px;">
              <n-form :model="settingsStore" label-placement="top">
                <n-form-item label="Language">
                  <n-select v-model:value="settingsStore.language" :options="languageOptions" />
                </n-form-item>
                <n-form-item label="Theme">
                  <n-select v-model:value="settingsStore.theme" :options="themeOptions" />
                </n-form-item>
                <n-form-item label="Default Workspace Directory">
                  <n-input v-model:value="settingsStore.workspaceDir" placeholder="C:/Projects/Subtitles" />
                </n-form-item>
                <n-form-item>
                  <n-checkbox v-model:checked="settingsStore.autoSave">Enable Auto-Save in Editor</n-checkbox>
                </n-form-item>
                <n-form-item>
                  <n-checkbox v-model:checked="settingsStore.autoBackup">Enable Automatic Backups</n-checkbox>
                </n-form-item>
              </n-form>
            </n-card>
          </div>
        </div>
      </n-tab-pane>

      <n-tab-pane name="models" tab="Speech Models">
        <Models />
      </n-tab-pane>

      <n-tab-pane name="providers" tab="AI Providers">
        <Providers />
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<script setup lang="ts">
import {
  NCard, NForm, NFormItem, NInput, NSelect, NCheckbox, NTabs, NTabPane
} from 'naive-ui'
import { useSettingsStore } from '../stores/settings'
import Models from './Models.vue'
import Providers from './Providers.vue'

const settingsStore = useSettingsStore()

const languageOptions = [
  { label: 'English (US)', value: 'en-US' },
  { label: 'Indonesian (ID)', value: 'id-ID' }
]

const themeOptions = [
  { label: 'Dark (SubPilot Default)', value: 'dark' },
  { label: 'Light', value: 'light' },
  { label: 'System', value: 'system' }
]
</script>

<style scoped>
.settings-view {
  max-width: 100%;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  height: 100%;
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

.settings-tabs {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.settings-tabs :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  overflow: auto;
  padding-top: 16px;
  padding-bottom: 24px;
}

.content-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

.column {
  display: flex;
  flex-direction: column;
}

.settings-card {
  background-color: #1e293b;
  border-radius: 8px;
}
</style>
