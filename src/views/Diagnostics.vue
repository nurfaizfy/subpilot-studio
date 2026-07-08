<template>
  <div class="diagnostics-page p-6">
    <div class="header">
      <h2>{{ $t('diagnostics.title') }}</h2>
      <n-space>
        <n-button type="primary" @click="runDiagnostics" :loading="store.isRunning">
          {{ $t('diagnostics.runBtn') }}
        </n-button>
        <n-button @click="copyReport" :disabled="store.results.length === 0">
          {{ $t('diagnostics.copyBtn') }}
        </n-button>
      </n-space>
    </div>

    <n-card class="mt-4" style="background-color: #1e293b;">
      <template v-if="store.results.length === 0">
        <n-empty :description="$t('diagnostics.emptyState')" />
      </template>
      <template v-else>
        <div class="diagnostic-grid">
          <n-card v-for="item in store.results" :key="item.name" class="diag-card" :class="`health-${item.health}`">
            <div class="diag-header">
              <span class="diag-title">{{ item.name }}</span>
              <n-tag :type="getTagType(item.health)" size="small">
                {{ item.status }}
              </n-tag>
            </div>
            <div class="diag-body">
              <div v-if="item.version" class="diag-row">
                <span class="diag-label">{{ $t('diagnostics.labels.info') }}</span>
                <span class="diag-value">{{ item.version }}</span>
              </div>
              <div v-if="item.path" class="diag-row">
                <span class="diag-label">{{ $t('diagnostics.labels.path') }}</span>
                <span class="diag-value path">{{ item.path }}</span>
              </div>
              <div v-if="item.error_msg" class="diag-row error-msg">
                <span class="diag-label">{{ $t('diagnostics.labels.error') }}</span>
                <span class="diag-value">{{ item.error_msg }}</span>
              </div>
              <div v-if="item.fix_suggestion" class="diag-row fix-msg">
                <span class="diag-label">{{ $t('diagnostics.labels.fix') }}</span>
                <span class="diag-value">{{ item.fix_suggestion }}</span>
              </div>
            </div>
          </n-card>
        </div>
      </template>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { NCard, NButton, NSpace, NEmpty, NTag, useMessage } from 'naive-ui'
import { useDiagnosticsStore } from '../stores/diagnostics'
import { useI18n } from 'vue-i18n'

const store = useDiagnosticsStore()
const message = useMessage()
const { t } = useI18n()

const runDiagnostics = async () => {
  await store.runDiagnostics()
}

const copyReport = async () => {
  const report = store.generateReport()
  try {
    await navigator.clipboard.writeText(report)
    message.success(t('diagnostics.messages.copySuccess'))
  } catch (e) {
    message.error(t('diagnostics.messages.copyFail') + String(e))
  }
}

const getTagType = (health: string) => {
  if (health === 'healthy') return 'success'
  if (health === 'warning') return 'warning'
  return 'error'
}
</script>

<style scoped>
.diagnostics-page {
  padding: 24px;
}
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}
.header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: #e2e8f0;
}
.mt-4 {
  margin-top: 16px;
}
.diagnostic-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 16px;
}
.diag-card {
  background-color: #0f172a;
  border-left: 4px solid transparent;
}
.diag-card.health-healthy {
  border-left-color: #18a058;
}
.diag-card.health-warning {
  border-left-color: #f0a020;
}
.diag-card.health-error {
  border-left-color: #d03050;
}
.diag-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  border-bottom: 1px solid #334155;
  padding-bottom: 8px;
}
.diag-title {
  font-weight: bold;
  font-size: 16px;
  color: #f1f5f9;
}
.diag-row {
  margin-bottom: 8px;
  display: flex;
  flex-direction: column;
}
.diag-label {
  font-size: 12px;
  color: #94a3b8;
  margin-bottom: 2px;
}
.diag-value {
  font-size: 14px;
  color: #cbd5e1;
  word-break: break-word;
}
.diag-value.path {
  font-family: monospace;
  font-size: 12px;
  background: #1e293b;
  padding: 4px;
  border-radius: 4px;
}
.error-msg .diag-value {
  color: #f87171;
}
.fix-msg .diag-value {
  color: #fbbf24;
}
</style>
