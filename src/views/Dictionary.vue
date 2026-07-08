<template>
  <div class="dictionary-view">
    <div class="header">
      <h2>Global Dictionary</h2>
      <p class="subtitle">Manage unified translation rules (Overrides AI Translation)</p>
    </div>

    <n-card class="dict-card">
      <div class="toolbar">
        <n-button type="primary" @click="showAddModal = true">
          Add Entry
        </n-button>
        
        <div class="right-tools">
          <n-select 
            v-model:value="filterType" 
            :options="typeFilterOptions" 
            placeholder="Filter by Type"
            clearable
            style="width: 200px"
          />
          <n-input v-model:value="searchQuery" placeholder="Search original or translation..." clearable style="width: 300px" />
          <n-button @click="importDict">Import JSON</n-button>
          <n-button @click="exportDict">Export JSON</n-button>
        </div>
      </div>

      <n-data-table
        :columns="columns"
        :data="filteredEntries"
        :bordered="false"
        :loading="loading"
        class="dict-table"
      />
    </n-card>

    <n-modal v-model:show="showAddModal" preset="card" :title="isEditing ? 'Edit Entry' : 'Add Entry'" style="width: 500px">
      <n-form :model="form" ref="formRef" :rules="rules">
        <n-form-item label="Original Text" path="original">
          <n-input v-model:value="form.original" placeholder="e.g. 先生" />
        </n-form-item>
        
        <n-form-item label="Translation" path="translation">
          <n-input v-model:value="form.translation" placeholder="e.g. Sensei" />
        </n-form-item>

        <n-form-item label="Type" path="entry_type">
          <n-select v-model:value="form.entry_type" :options="typeOptions" />
        </n-form-item>

        <n-form-item label="Priority (Higher overrides lower)" path="priority">
          <n-input-number v-model:value="form.priority" :min="1" :max="100" />
        </n-form-item>

        <div class="modal-actions">
          <n-button @click="showAddModal = false">Cancel</n-button>
          <n-button type="primary" @click="saveEntry" :loading="saving">Save</n-button>
        </div>
      </n-form>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, h, onMounted } from 'vue'
import { 
  NCard, NButton, NDataTable, NModal, NForm, NFormItem, 
  NInput, NSelect, NInputNumber, useMessage, NTag, NSpace, NPopconfirm
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'

const message = useMessage()

interface DictEntry {
  id: string;
  original: string;
  translation: string;
  entry_type: string;
  priority: number;
}

const entries = ref<DictEntry[]>([])
const loading = ref(false)
const showAddModal = ref(false)
const isEditing = ref(false)
const saving = ref(false)
const filterType = ref<string | null>(null)
const searchQuery = ref('')

const form = ref({
  id: '',
  original: '',
  translation: '',
  entry_type: 'General',
  priority: 1
})

const rules = {
  original: { required: true, message: 'Original text is required', trigger: 'blur' },
  translation: { required: true, message: 'Translation is required', trigger: 'blur' }
}

const typeOptions = [
  { label: 'General', value: 'General' },
  { label: 'Honorific', value: 'Honorific' },
  { label: 'Medical', value: 'Medical' },
  { label: 'Police', value: 'Police' },
  { label: 'School', value: 'School' },
  { label: 'Business', value: 'Business' }
]

const typeFilterOptions = [
  ...typeOptions
]

const fetchEntries = async () => {
  loading.value = true
  try {
    entries.value = await invoke('get_dictionary')
  } catch (e: any) {
    message.error("Failed to load dictionary: " + e)
  }
  loading.value = false
}

onMounted(() => {
  fetchEntries()
})

const filteredEntries = computed(() => {
  let result = entries.value
  if (filterType.value) {
    result = result.filter(e => e.entry_type === filterType.value)
  }
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(e => 
      e.original.toLowerCase().includes(q) || 
      e.translation.toLowerCase().includes(q)
    )
  }
  return result
})

const saveEntry = async () => {
  if (!form.value.original || !form.value.translation) {
    message.warning("Please fill all required fields")
    return
  }
  saving.value = true
  try {
    if (isEditing.value) {
      await invoke('update_dictionary_entry', { entry: form.value })
      message.success("Entry updated")
    } else {
      await invoke('add_dictionary_entry', { entry: form.value })
      message.success("Entry added")
    }
    showAddModal.value = false
    fetchEntries()
  } catch (e: any) {
    message.error("Failed to save entry: " + e)
  }
  saving.value = false
}

const editEntry = (row: DictEntry) => {
  form.value = { ...row }
  isEditing.value = true
  showAddModal.value = true
}

const deleteEntry = async (id: string) => {
  try {
    await invoke('delete_dictionary_entry', { id })
    message.success("Entry deleted")
    fetchEntries()
  } catch (e: any) {
    message.error("Failed to delete entry: " + e)
  }
}

import { watch } from 'vue'
watch(showAddModal, (val) => {
  if (!val) {
    setTimeout(() => {
      isEditing.value = false
      form.value = { id: '', original: '', translation: '', entry_type: 'General', priority: 1 }
    }, 200)
  }
})

const columns: any[] = [
  { title: 'Original', key: 'original', sorter: 'default' },
  { title: 'Translation', key: 'translation', sorter: 'default' },
  { 
    title: 'Type', 
    key: 'entry_type',
    render(row: DictEntry) {
      let type: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'error' = 'default'
      if (row.entry_type === 'Honorific') type = 'warning'
      if (row.entry_type === 'Medical') type = 'error'
      if (row.entry_type === 'Police') type = 'info'
      if (row.entry_type === 'School') type = 'success'
      if (row.entry_type === 'Business') type = 'primary'
      
      return h(NTag, { type, bordered: false, size: 'small' }, { default: () => row.entry_type })
    }
  },
  { title: 'Priority', key: 'priority', sorter: (a: DictEntry, b: DictEntry) => a.priority - b.priority },
  {
    title: 'Actions',
    key: 'actions',
    render(row: DictEntry) {
      return h(NSpace, {}, {
        default: () => [
          h(NButton, { size: 'small', onClick: () => editEntry(row) }, { default: () => 'Edit' }),
          h(
            NPopconfirm,
            { onPositiveClick: () => deleteEntry(row.id) },
            {
              trigger: () => h(NButton, { size: 'small', type: 'error', tertiary: true }, { default: () => 'Delete' }),
              default: () => 'Are you sure you want to delete this entry?'
            }
          )
        ]
      })
    }
  }
]

const exportDict = () => {
  try {
    const data = JSON.stringify(entries.value, null, 2)
    const blob = new Blob([data], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'subpilot_dictionary.json'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    message.success("Dictionary exported successfully")
  } catch (e: any) {
    message.error("Export failed: " + e)
  }
}

const importDict = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = async (e: any) => {
    const file = e.target.files?.[0]
    if (!file) return

    const reader = new FileReader()
    reader.onload = async (e) => {
      try {
        const content = e.target?.result as string
        const importedEntries = JSON.parse(content) as DictEntry[]
        
        if (!Array.isArray(importedEntries)) {
          throw new Error("Invalid format")
        }

        loading.value = true
        let successCount = 0
        for (const item of importedEntries) {
          if (item.original && item.translation && item.entry_type) {
            await invoke('add_dictionary_entry', { 
              entry: {
                id: '',
                original: item.original,
                translation: item.translation,
                entry_type: item.entry_type,
                priority: item.priority || 1
              }
            })
            successCount++
          }
        }
        
        message.success(`Imported ${successCount} entries successfully`)
        fetchEntries()
      } catch (err: any) {
        message.error("Import failed: " + err)
        loading.value = false
      }
    }
    reader.readAsText(file)
  }
  input.click()
}

</script>

<style scoped>
.dictionary-view {
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

.dict-card {
  background-color: #1e293b;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  margin-bottom: 24px;
}

.right-tools {
  display: flex;
  gap: 12px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}
</style>
