<template>
  <header class="topbar">
    <div class="left-section">
      <div v-if="projectStore.currentProject" class="project-info">
        <n-icon size="18" color="#94a3b8"><FilmIcon /></n-icon>
        <span class="project-name">{{ projectStore.currentProject.name }}</span>
      </div>
      <div v-else class="project-info">
        <span class="no-project">No Project Open</span>
      </div>
    </div>
    
    <div class="right-section">
      <n-input placeholder="Search..." round clearable>
        <template #prefix>
          <n-icon><SearchIcon /></n-icon>
        </template>
      </n-input>
      <n-button circle secondary>
        <template #icon>
          <n-icon><MoonIcon /></n-icon>
        </template>
      </n-button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { NIcon, NInput, NButton } from 'naive-ui'
import { 
  SearchOutline as SearchIcon, 
  MoonOutline as MoonIcon,
  FilmOutline as FilmIcon
} from '@vicons/ionicons5'
import { useProjectStore } from '../stores/project'

const projectStore = useProjectStore()
</script>

<style scoped>
.topbar {
  height: 60px;
  background-color: #1e293b;
  border-bottom: 1px solid #334155;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0 24px;
}

.left-section, .right-section {
  display: flex;
  align-items: center;
  gap: 16px;
}

.project-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.project-name {
  font-weight: 600;
  color: #f8fafc;
}

.no-project {
  color: #64748b;
  font-style: italic;
}
</style>
