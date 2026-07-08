<template>
  <div class="dashboard">
    <div class="header">
      <h2>{{ $t('dashboard.welcomeTitle') }}</h2>
      <p class="subtitle">{{ $t('dashboard.welcomeSubtitle') }}</p>
    </div>

    <div class="quick-actions">
      <n-card class="action-card" hoverable @click="handleCreate">
        <div class="action-content">
          <n-icon size="48" color="#6366f1">
            <AddIcon />
          </n-icon>
          <h3>{{ $t('dashboard.createProject') }}</h3>
          <p>{{ $t('dashboard.createProjectDesc') }}</p>
        </div>
      </n-card>

      <n-card class="action-card" hoverable @click="handleOpen">
        <div class="action-content">
          <n-icon size="48" color="#10b981">
            <FolderOpenIcon />
          </n-icon>
          <h3>{{ $t('dashboard.openProject') }}</h3>
          <p>{{ $t('dashboard.openProjectDesc') }}</p>
        </div>
      </n-card>
    </div>

    <div class="recent-projects" style="grid-column: 1 / -1;">
      <div class="section-header">
        <h3>{{ $t('dashboard.recentProjects') }}</h3>
        <n-button text type="primary" @click="$router.push('/projects')">{{ $t('dashboard.viewAll') }}</n-button>
      </div>

      <n-data-table :columns="columns" :data="projectStore.recentProjects" :bordered="false" :row-props="rowProps"
        class="recent-table" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed, h } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { NCard, NIcon, NButton, NDataTable } from 'naive-ui'
import {
  AddCircleOutline as AddIcon,
  FolderOpenOutline as FolderOpenIcon,
  CheckmarkCircleOutline as CheckIcon,
  CloseCircleOutline as CrossIcon
} from '@vicons/ionicons5'
import { useProjectStore } from '../stores/project'
import { SubtitleManager } from '../services/subtitleManager'

const router = useRouter()
const projectStore = useProjectStore()
const { t } = useI18n()
const projectStatuses = ref<Record<string, string[]>>({})

const renderStatusIcon = (condition: boolean) => {
  if (condition) {
    return h(NIcon, { color: '#10b981', size: 20 }, { default: () => h(CheckIcon) })
  }
  return h(NIcon, { color: '#ef4444', size: 20 }, { default: () => h(CrossIcon) })
}

const columns = computed(() => [
  {
    title: t('dashboard.columns.projectName'), key: 'name',
    render(row: any) {
      return h('strong', { style: 'color: #f1f5f9' }, row.name)
    }
  },
  { title: t('dashboard.columns.type'), key: 'project_type' },
  {
    title: t('dashboard.columns.transcript'),
    key: 'transcript',
    align: 'center' as const,
    render(row: any) {
      const statuses = projectStatuses.value[row.id] || []
      return renderStatusIcon(statuses.includes('Transcribed'))
    }
  },
  {
    title: t('dashboard.columns.translate'),
    key: 'translate',
    align: 'center' as const,
    render(row: any) {
      const statuses = projectStatuses.value[row.id] || []
      return renderStatusIcon(statuses.includes('Translated'))
    }
  },
  {
    title: t('dashboard.columns.readyToExport'),
    key: 'export',
    align: 'center' as const,
    render(row: any) {
      const statuses = projectStatuses.value[row.id] || []
      const hasSubtitle = statuses.includes('Transcribed') || statuses.includes('Translated') || statuses.includes('Edited')
      return renderStatusIcon(hasSubtitle)
    }
  },
  {
    title: t('dashboard.columns.lastModified'),
    key: 'last_modified',
    render(row: any) {
      return new Date(row.last_modified).toLocaleDateString()
    }
  }
])

const rowProps = (row: any) => {
  return {
    style: 'cursor: pointer; transition: background-color 0.2s',
    onClick: () => {
      router.push(`/project/${row.id}`)
    }
  }
}

onMounted(async () => {
  await projectStore.fetchProjects()

  // Fetch statuses for each project
  for (const p of projectStore.recentProjects) {
    const statuses = []
    if (p.source_video) statuses.push('Video')

    try {
      const subs = await SubtitleManager.getProjectSubtitles(p.id)
      if (subs.some(s => s.category === 'transcripted')) statuses.push('Transcribed')
      if (subs.some(s => s.category === 'translated')) statuses.push('Translated')
    } catch (e) {
      // Ignore subtitle fetch errors
    }

    projectStatuses.value[p.id] = statuses
  }
})

const handleCreate = () => {
  router.push('/projects')
}

const handleOpen = () => {
  router.push('/projects')
}
</script>

<style scoped>
.dashboard {
  max-width: 100%;
  margin: 0 auto;
  position: relative;
  min-height: 80vh;
}

.header {
  margin-bottom: 40px;
}

.header h2 {
  font-size: 32px;
  margin: 0 0 8px 0;
  color: #f8fafc;
}

.subtitle {
  color: #94a3b8;
  font-size: 16px;
  margin: 0;
}

.quick-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 24px;
  margin-bottom: 48px;
}

.action-card {
  cursor: pointer;
  background-color: #1e293b;
  border-radius: 12px;
  border: 1px solid #334155;
  transition: all 0.3s ease;
}

.action-card:hover {
  transform: translateY(-4px);
  border-color: #6366f1;
  box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.5);
}

.action-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding: 24px 0;
}

.action-content h3 {
  margin: 16px 0 8px 0;
  color: #f1f5f9;
  font-size: 20px;
}

.action-content p {
  margin: 0;
  color: #64748b;
  font-size: 14px;
}

.content-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 24px;
}

.recent-projects {
  background-color: #1e293b;
  border-radius: 12px;
  padding: 24px;
  border: 1px solid #334155;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.section-header h3 {
  margin: 0;
  font-size: 20px;
  color: #f8fafc;
}

.recent-table {
  background-color: transparent;
}
</style>
