<template>
  <div class="settings-view">
    <div class="header">
      <h2>{{ $t('settings.title') }}</h2>
      <p class="subtitle">{{ $t('settings.subtitle') }}</p>
    </div>

    <n-tabs type="line" class="settings-tabs">
      <n-tab-pane name="general" :tab="$t('settings.tabs.general')">
        <div class="content-grid">
          <div class="column">
            <n-card class="settings-card" :title="$t('settings.tabs.general')" style="margin-top: 24px;">
              <n-form :model="settingsStore" label-placement="top">
                <n-form-item :label="$t('settings.general.language')">
                  <n-select v-model:value="settingsStore.language" :options="languageOptions" />
                </n-form-item>
                <n-form-item :label="$t('settings.general.theme')">
                  <n-select v-model:value="settingsStore.theme" :options="themeOptions" />
                </n-form-item>
                <n-form-item :label="$t('settings.general.defaultWorkspace')">
                  <n-input v-model:value="settingsStore.workspaceDir" placeholder="C:/Projects/Subtitles" />
                </n-form-item>
                <n-form-item>
                  <n-checkbox v-model:checked="settingsStore.autoSave">{{ $t('settings.general.autoSave') }}</n-checkbox>
                </n-form-item>
                <n-form-item>
                  <n-checkbox v-model:checked="settingsStore.autoBackup">{{ $t('settings.general.autoBackup') }}</n-checkbox>
                </n-form-item>
              </n-form>
            </n-card>
          </div>
        </div>
      </n-tab-pane>

      <n-tab-pane name="models" :tab="$t('settings.tabs.models')">
        <Models />
      </n-tab-pane>

      <n-tab-pane name="providers" :tab="$t('settings.tabs.providers')">
        <Providers />
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  NCard, NForm, NFormItem, NInput, NSelect, NCheckbox, NTabs, NTabPane
} from 'naive-ui'
import { useSettingsStore } from '../stores/settings'
import Models from './Models.vue'
import Providers from './Providers.vue'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const languageOptions = computed(() => [
  { label: 'English (US)', value: 'en-US' },
  { label: 'Bahasa Indonesia (ID)', value: 'id-ID' }
])

const themeOptions = computed(() => [
  { label: t('settings.themes.dark'), value: 'dark' },
  { label: t('settings.themes.light'), value: 'light' },
  { label: t('settings.themes.system'), value: 'system' }
])
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
