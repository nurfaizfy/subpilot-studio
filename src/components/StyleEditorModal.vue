<template>
  <n-modal v-model:show="internalShow" preset="card" :title="isEdit ? $t('styleEditor.editStyle') : $t('styleEditor.newStyle')" style="width: 800px; max-width: 95vw;">
    <n-form ref="formRef" :model="formData" :rules="rules" label-placement="left" label-width="120" require-mark-placement="right-hanging">
      <n-tabs type="line" animated>
        <n-tab-pane name="basics" :tab="$t('styleEditor.tabs.basics')">
          <n-form-item :label="$t('styleEditor.fields.styleName')" path="Name">
            <n-input v-model:value="formData.Name" :placeholder="$t('styleEditor.placeholders.styleName')" :disabled="isEdit && formData.Name === 'Default'" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.fontName')" path="Fontname">
            <n-select filterable tag v-model:value="formData.Fontname" :options="systemFonts" :placeholder="$t('styleEditor.placeholders.fontName')" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.fontSize')" path="Fontsize">
            <n-input-number v-model:value="formData.Fontsize" :min="1" />
          </n-form-item>
        </n-tab-pane>

        <n-tab-pane name="colors" :tab="$t('styleEditor.tabs.colors')">
          <n-form-item :label="$t('styleEditor.fields.primary')" path="PrimaryColour">
            <n-color-picker show-alpha :modes="['hex']" v-model:value="primaryColorHex" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.secondary')" path="SecondaryColour">
            <n-color-picker show-alpha :modes="['hex']" v-model:value="secondaryColorHex" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.outline')" path="OutlineColour">
            <n-color-picker show-alpha :modes="['hex']" v-model:value="outlineColorHex" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.background')" path="BackColour">
            <n-color-picker show-alpha :modes="['hex']" v-model:value="backColorHex" />
          </n-form-item>
        </n-tab-pane>

        <n-tab-pane name="layout" :tab="$t('styleEditor.tabs.layout')">
          <n-form-item :label="$t('styleEditor.fields.alignment')" path="Alignment">
            <n-select v-model:value="formData.Alignment" :options="alignmentOptions" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.marginL')" path="MarginL">
            <n-input-number v-model:value="formData.MarginL" :min="0" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.marginR')" path="MarginR">
            <n-input-number v-model:value="formData.MarginR" :min="0" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.marginV')" path="MarginV">
            <n-input-number v-model:value="formData.MarginV" :min="0" />
          </n-form-item>
        </n-tab-pane>

        <n-tab-pane name="effects" :tab="$t('styleEditor.tabs.effects')">
          <n-form-item :label="$t('styleEditor.fields.outline')" path="Outline">
            <n-input-number v-model:value="formData.Outline" :min="0" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.shadow')" path="Shadow">
            <n-input-number v-model:value="formData.Shadow" :min="0" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.bold')" path="Bold">
            <n-switch v-model:value="formData.Bold" :checked-value="-1" :unchecked-value="0" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.italic')" path="Italic">
            <n-switch v-model:value="formData.Italic" :checked-value="-1" :unchecked-value="0" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.underline')" path="Underline">
            <n-switch v-model:value="formData.Underline" :checked-value="-1" :unchecked-value="0" />
          </n-form-item>
          <n-form-item :label="$t('styleEditor.fields.strikeout')" path="StrikeOut">
            <n-switch v-model:value="formData.StrikeOut" :checked-value="-1" :unchecked-value="0" />
          </n-form-item>
        </n-tab-pane>
      </n-tabs>

      <div style="display: flex; justify-content: flex-end; gap: 12px; margin-top: 24px;">
        <n-button @click="internalShow = false">{{ $t('styleEditor.buttons.cancel') }}</n-button>
        <n-button type="primary" @click="handleSave">{{ $t('styleEditor.buttons.save') }}</n-button>
      </div>
    </n-form>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import {
  NModal, NForm, NFormItem, NInput, NInputNumber, NSelect, NSwitch, NTabs, NTabPane, NColorPicker, NButton, useMessage
} from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { useI18n } from 'vue-i18n'
import { useStylesStore } from '../stores/styles'
import { assColorToHex, hexToAssColor } from '../utils/subtitleParser'
import type { AssStyle } from '../utils/subtitleParser'

const props = defineProps<{
  show: boolean
  styleToEdit?: AssStyle | null
}>()

const emit = defineEmits<{
  (e: 'update:show', val: boolean): void
  (e: 'saved'): void
}>()

const stylesStore = useStylesStore()
const message = useMessage()
const { t } = useI18n()

const internalShow = computed({
  get: () => props.show,
  set: (val) => emit('update:show', val)
})

const isEdit = computed(() => !!props.styleToEdit)

const formRef = ref<any>(null)
const formData = ref<any>({
  Name: 'Default',
  Fontname: 'Arial',
  Fontsize: 48,
  PrimaryColour: '&H00FFFFFF',
  SecondaryColour: '&H000000FF',
  OutlineColour: '&H00000000',
  BackColour: '&HFF000000',
  Bold: 0,
  Italic: 0,
  Underline: 0,
  StrikeOut: 0,
  ScaleX: 100,
  ScaleY: 100,
  Spacing: 0,
  Angle: 0,
  BorderStyle: 1,
  Outline: 2,
  Shadow: 2,
  Alignment: 2,
  MarginL: 10,
  MarginR: 10,
  MarginV: 10,
  Encoding: 1
})

const primaryColorHex = computed({
  get: () => assColorToHex(formData.value.PrimaryColour),
  set: (val) => { formData.value.PrimaryColour = hexToAssColor(val) }
})
const secondaryColorHex = computed({
  get: () => assColorToHex(formData.value.SecondaryColour),
  set: (val) => { formData.value.SecondaryColour = hexToAssColor(val) }
})
const outlineColorHex = computed({
  get: () => assColorToHex(formData.value.OutlineColour),
  set: (val) => { formData.value.OutlineColour = hexToAssColor(val) }
})
const backColorHex = computed({
  get: () => assColorToHex(formData.value.BackColour),
  set: (val) => { formData.value.BackColour = hexToAssColor(val) }
})

const alignmentOptions = [
  { label: 'Bottom Left (1)', value: 1 },
  { label: 'Bottom Center (2)', value: 2 },
  { label: 'Bottom Right (3)', value: 3 },
  { label: 'Middle Left (4)', value: 4 },
  { label: 'Middle Center (5)', value: 5 },
  { label: 'Middle Right (6)', value: 6 },
  { label: 'Top Left (7)', value: 7 },
  { label: 'Top Center (8)', value: 8 },
  { label: 'Top Right (9)', value: 9 },
]

const systemFonts = ref<{label: string, value: string}[]>([
  { label: 'Arial', value: 'Arial' },
  { label: 'Verdana', value: 'Verdana' },
  { label: 'Tahoma', value: 'Tahoma' },
  { label: 'Trebuchet MS', value: 'Trebuchet MS' }
])

const loadFonts = async () => {
  try {
    const fonts = await invoke<string[]>('get_system_fonts')
    if (fonts && fonts.length > 0) {
      systemFonts.value = fonts.map(f => ({ label: f, value: f }))
    }
  } catch (e) {
    if ('queryLocalFonts' in window) {
      try {
        const fonts = await (window as any).queryLocalFonts()
        const uniqueFonts = new Set<string>()
        fonts.forEach((f: any) => uniqueFonts.add(f.family))
        const fontList = Array.from(uniqueFonts).sort()
        if (fontList.length > 0) {
          systemFonts.value = fontList.map(f => ({ label: f, value: f }))
        }
      } catch (e2) {}
    }
  }
}

const rules = computed(() => ({
  Name: [{ required: true, message: t('styleEditor.messages.nameRequired'), trigger: 'blur' }],
  Fontname: [{ required: true, message: t('styleEditor.messages.fontRequired'), trigger: 'blur' }]
}))

watch(() => props.show, (val) => {
  if (val) {
    if (props.styleToEdit) {
      const parsed: any = { ...JSON.parse(JSON.stringify(props.styleToEdit)) }
      const numFields = ['Fontsize', 'MarginL', 'MarginR', 'MarginV', 'Outline', 'Shadow', 'Bold', 'Italic', 'Underline', 'StrikeOut', 'Alignment', 'Encoding', 'ScaleX', 'ScaleY', 'Spacing', 'Angle', 'BorderStyle']
      for (const field of numFields) {
        if (parsed[field] !== undefined) parsed[field] = Number(parsed[field])
      }
      formData.value = parsed
    } else {
      formData.value = {
        Name: 'NewStyle',
        Fontname: 'Arial',
        Fontsize: 48,
        PrimaryColour: '&H00FFFFFF',
        SecondaryColour: '&H000000FF',
        OutlineColour: '&H00000000',
        BackColour: '&HFF000000',
        Bold: 0,
        Italic: 0,
        Underline: 0,
        StrikeOut: 0,
        ScaleX: 100,
        ScaleY: 100,
        Spacing: 0,
        Angle: 0,
        BorderStyle: 1,
        Outline: 2,
        Shadow: 2,
        Alignment: 2,
        MarginL: 10,
        MarginR: 10,
        MarginV: 10,
        Encoding: 1
      }
    }
  }
})

onMounted(() => {
  loadFonts()
})

const handleSave = () => {
  formRef.value?.validate(async (errors: any) => {
    if (!errors) {
      try {
        const payload: any = {}
        for (const [key, val] of Object.entries(formData.value)) {
          payload[key] = String(val)
        }
        
        await stylesStore.saveStyle(payload as AssStyle)
        message.success(t('styleEditor.messages.saveSuccess', { action: isEdit.value ? 'updated' : 'created' }))
        internalShow.value = false
        emit('saved')
      } catch (e: any) {
        message.error(t('styleEditor.messages.saveFailed') + e.message)
      }
    }
  })
}
</script>
