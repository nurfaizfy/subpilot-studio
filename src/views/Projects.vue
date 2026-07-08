<template>
  <div class="projects-view">
    <div class="header">
      <h2>All Projects</h2>
      <n-button type="primary" @click="showCreateModal = true">
        <template #icon>
          <n-icon>
            <AddIcon />
          </n-icon>
        </template>
        New Project
      </n-button>
    </div>

    <div class="projects-container">
      <n-empty v-if="projectStore.recentProjects.length === 0" description="No projects found">
        <template #extra>
          <n-button size="small" @click="showCreateModal = true">Create your first project</n-button>
        </template>
      </n-empty>

      <div v-else class="project-grid">
        <n-card v-for="proj in projectStore.recentProjects" :key="proj.id" class="project-card" hoverable>
          <template #cover>
            <div class="thumbnail-placeholder">
              <n-icon size="48" color="#64748b">
                <FilmIcon />
              </n-icon>
              <span class="type-badge">{{ proj.project_type }}</span>
            </div>
          </template>

          <h3 class="proj-name">{{ proj.name }}</h3>
          <p class="proj-meta">Source: {{ proj.source_video || 'No source file' }}</p>
          <p class="proj-meta">Last Modified: {{ new Date(proj.last_modified).toLocaleDateString() }}</p>

          <template #action>
            <div class="card-actions">
              <n-button size="small" type="primary" ghost @click="openProject(proj)">Open</n-button>
              <n-dropdown :options="dropdownOptions" @select="(key) => handleDropdownSelect(key, proj.id)">
                <n-button size="small" circle ghost>
                  <template #icon><n-icon>
                      <EllipsisIcon />
                    </n-icon></template>
                </n-button>
              </n-dropdown>
            </div>
          </template>
        </n-card>
      </div>
    </div>

    <n-modal v-model:show="showCreateModal" preset="card" style="width: 600px" title="Create New Project">
      <n-form :model="formData" label-placement="left" label-width="120">
        <n-form-item label="Project Name" path="name">
          <n-input v-model:value="formData.name" placeholder="Enter project name" />
        </n-form-item>

        <n-form-item label="Project Type" path="project_type">
          <n-select v-model:value="formData.project_type" :options="typeOptions" />
        </n-form-item>

        <n-form-item label="Original Lang" path="original_language">
          <n-select filterable v-model:value="formData.original_language" :options="originalLanguageOptions"
            placeholder="Select Language" />
        </n-form-item>

        <n-form-item label="Target Lang" path="target_language">
          <n-select filterable v-model:value="formData.target_language" :options="targetLanguageOptions"
            placeholder="Select Language" />
        </n-form-item>

        <div v-if="formData.project_type === 'Anime' || formData.project_type === 'Drama'" class="season-episode">
          <n-form-item label="Season">
            <n-input-number v-model:value="formData.season" :min="1" />
          </n-form-item>
          <n-form-item label="Episode">
            <n-input-number v-model:value="formData.episode" :min="1" />
          </n-form-item>
        </div>
      </n-form>

      <template #footer>
        <div style="display: flex; justify-content: flex-end; gap: 12px;">
          <n-button @click="showCreateModal = false">Cancel</n-button>
          <n-button type="primary" @click="submitCreate">Create Project</n-button>
        </div>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  NButton, NIcon, NEmpty, NCard, NDropdown,
  NModal, NForm, NFormItem, NInput, NSelect, NInputNumber,
  useMessage, useDialog
} from 'naive-ui'
import {
  AddOutline as AddIcon,
  FilmOutline as FilmIcon,
  EllipsisHorizontal as EllipsisIcon
} from '@vicons/ionicons5'
import { useProjectStore } from '../stores/project'
import { useRouter } from 'vue-router'

const message = useMessage()
const dialog = useDialog()
const router = useRouter()
const projectStore = useProjectStore()

const showCreateModal = ref(false)

const formData = ref({
  name: '',
  project_type: 'Other',
  original_language: 'Japanese',
  target_language: 'Indonesian',
  season: null,
  episode: null,
  source_video: ''
})

const typeOptions = [
  { label: 'Anime', value: 'Anime' },
  { label: 'Drama', value: 'Drama' },
  { label: 'Movie', value: 'Movie' },
  { label: 'Other', value: 'Other' }
]

const originalLanguageOptions = [
  { label: 'Japanese', value: 'Japanese' },
  { label: 'Chinese', value: 'Chinese' },
  { label: 'Korean', value: 'Korean' }
]

const targetLanguageOptions = [
  { label: 'English', value: 'English' },
  { label: 'Indonesian', value: 'Indonesian' }
]

const dropdownOptions = [
  { label: 'Duplicate', key: 'duplicate' },
  { label: 'Delete', key: 'delete' }
]

onMounted(async () => {
  await projectStore.fetchProjects()
})

const openProject = (proj: any) => {
  projectStore.setProject(proj)
  message.success(`Opened project: ${proj.name}`)
  router.push(`/project/${proj.id}`)
}

const handleDropdownSelect = async (key: string, id: string) => {
  if (key === 'duplicate') {
    try {
      await projectStore.copyProject(id)
      message.success("Project duplicated successfully")
    } catch (e) {
      message.error("Failed to duplicate project")
    }
  } else if (key === 'delete') {
    dialog.warning({
      title: 'Confirm',
      content: 'Are you sure you want to delete this project?',
      positiveText: 'Delete',
      negativeText: 'Cancel',
      onPositiveClick: async () => {
        try {
          await projectStore.removeProject(id)
          message.success("Project deleted")
        } catch (e) {
          message.error("Failed to delete project")
        }
      }
    })
  }
}


const submitCreate = async () => {
  if (!formData.value.name) {
    message.error("Project name is required")
    return
  }

  try {
    await projectStore.createNewProject(formData.value)
    showCreateModal.value = false
    message.success("Project created successfully")

    formData.value = {
      name: '',
      project_type: 'Other',
      original_language: 'Japanese',
      target_language: 'Indonesian',
      season: null,
      episode: null,
      source_video: ''
    }
  } catch (e) {
    message.error("Failed to create project")
  }
}
</script>

<style scoped>
.projects-view {
  max-width: 100%;
  margin: 0 auto;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}

.header h2 {
  font-size: 28px;
  margin: 0;
  color: #f8fafc;
}

.projects-container {
  min-height: 400px;
  display: flex;
  flex-direction: column;
}

.project-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
}

.project-card {
  background-color: #1e293b;
  border-radius: 12px;
  border: 1px solid #334155;
  transition: all 0.3s ease;
  overflow: hidden;
}

.project-card:hover {
  border-color: #6366f1;
}

.thumbnail-placeholder {
  height: 160px;
  background-color: #0f172a;
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
  border-bottom: 1px solid #334155;
  border-radius: 11px 11px 0 0;
}

.type-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  background-color: #4f46e5;
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
}

.proj-name {
  margin: 0 0 8px 0;
  font-size: 18px;
  color: #f1f5f9;
}

.proj-meta {
  margin: 0 0 4px 0;
  font-size: 13px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.season-episode {
  display: flex;
  gap: 16px;
}
</style>
