<template>
  <div class="subtitle-styling-view">
    <div class="header-section">
      <div class="title">
        <n-icon size="28" color="#6366f1">
          <ColorPaletteIcon />
        </n-icon>
        <h2>Global Subtitle Styles</h2>
      </div>
      <n-button type="primary" @click="openAddModal">
        <template #icon>
          <n-icon>
            <AddIcon />
          </n-icon>
        </template>
        New Style
      </n-button>
    </div>

    <div class="content-section">
      <n-card class="styles-card">
        <n-data-table :columns="columns" :data="stylesStore.styles" :bordered="false" :pagination="{ pageSize: 10 }" />
      </n-card>
    </div>

    <StyleEditorModal v-model:show="showModal" :style-to-edit="styleToEdit" @saved="onStyleSaved" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue'
import {
  NIcon, NButton, NCard, NDataTable, useMessage, useDialog
} from 'naive-ui'
import {
  ColorPaletteOutline as ColorPaletteIcon,
  AddOutline as AddIcon,
  CreateOutline as EditIcon,
  TrashOutline as DeleteIcon,
  CopyOutline as CopyIcon
} from '@vicons/ionicons5'
import { useStylesStore } from '../stores/styles'
import type { AssStyle } from '../utils/subtitleParser'
import StyleEditorModal from '../components/StyleEditorModal.vue'

const stylesStore = useStylesStore()
const message = useMessage()
const dialog = useDialog()

const showModal = ref(false)
const styleToEdit = ref<AssStyle | null>(null)

const onStyleSaved = () => {
  showModal.value = false
  stylesStore.loadStyles()
}

onMounted(async () => {
  await stylesStore.loadStyles()
})

const columns = [
  { title: 'Name', key: 'Name' },
  { title: 'Font', key: 'Fontname' },
  { title: 'Size', key: 'Fontsize' },
  {
    title: 'Actions',
    key: 'actions',
    render(row: AssStyle) {
      return h('div', { style: 'display: flex; gap: 8px;' }, [
        h(NButton, {
          size: 'small',
          onClick: () => openEditModal(row)
        }, { icon: () => h(NIcon, null, { default: () => h(EditIcon) }) }),
        h(NButton, {
          size: 'small',
          onClick: () => duplicateStyle(row)
        }, { icon: () => h(NIcon, null, { default: () => h(CopyIcon) }) }),
        h(NButton, {
          size: 'small',
          type: 'error',
          disabled: row.Name === 'Default',
          onClick: () => deleteStyle(row.Name)
        }, { icon: () => h(NIcon, null, { default: () => h(DeleteIcon) }) })
      ])
    }
  }
]

const openAddModal = () => {
  styleToEdit.value = null
  showModal.value = true
}

const openEditModal = (row: AssStyle) => {
  styleToEdit.value = row
  showModal.value = true
}

const duplicateStyle = async (row: AssStyle) => {
  try {
    const newStyle = JSON.parse(JSON.stringify(row))
    newStyle.Name = `${row.Name}_copy`
    await stylesStore.saveStyle(newStyle)
    message.success(`Style ${row.Name} duplicated`)
    await stylesStore.loadStyles()
  } catch (e: any) {
    message.error(`Failed to duplicate style: ${e.message}`)
  }
}

const deleteStyle = (name: string) => {
  dialog.warning({
    title: 'Confirm Delete',
    content: `Are you sure you want to delete style "${name}"?`,
    positiveText: 'Delete',
    negativeText: 'Cancel',
    onPositiveClick: async () => {
      try {
        await stylesStore.deleteStyle(name)
        message.success(`Style ${name} deleted`)
        await stylesStore.loadStyles()
      } catch (e: any) {
        message.error(`Failed to delete style: ${e.message}`)
      }
    }
  })
}
</script>

<style scoped>
.subtitle-styling-view {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 600;
  color: white;
}

.styles-card {
  background-color: #1e293b;
  border-radius: 8px;
  border: 1px solid #334155;
}
</style>
