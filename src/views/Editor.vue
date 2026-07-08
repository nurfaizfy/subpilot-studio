<template>
  <div class="editor-view">
    <div v-if="!projectStore.currentProject" class="empty-state-wrapper">
      <n-empty :description="$t('editor.noProject')">
        <template #extra>
          <n-button type="primary" @click="$router.push('/projects')">{{ $t('editor.goToProjects') }}</n-button>
        </template>
      </n-empty>
    </div>

    <template v-else>
      <div class="editor-layout">
        <div class="left-panel">
          <div class="toolbar-section">
            <div class="tool-group">
              <n-button size="small" type="primary" ghost @click="importSubtitle" :loading="isSaving"
                :title="$t('editor.import')">
                <template #icon><n-icon>
                    <ImportIcon />
                  </n-icon></template>
              </n-button>
              <n-button size="small" @click="subtitleStore.undo" :disabled="subtitleStore.historyIndex <= 0"
                :title="$t('editor.undo')">
                <template #icon><n-icon>
                    <UndoIcon />
                  </n-icon></template>
              </n-button>
              <n-button size="small" @click="subtitleStore.redo"
                :disabled="subtitleStore.historyIndex >= subtitleStore.historyLength - 1" :title="$t('editor.redo')">
                <template #icon><n-icon>
                    <RedoIcon />
                  </n-icon></template>
              </n-button>
              <n-button size="small" type="primary" @click="manualSave" :loading="isSaving" :title="$t('editor.save')">
                <template #icon><n-icon>
                    <SaveIcon />
                  </n-icon></template>
              </n-button>
            </div>

            <n-divider vertical />

            <div class="tool-group">
              <n-select size="small" style="width: 200px" v-model:value="selectedVersionPath" :options="versionOptions"
                :placeholder="$t('editor.selectVersion')" @update:value="onVersionSelected" />
              <n-button size="small" type="error" ghost @click="deleteSubtitleFile" :disabled="!selectedVersionPath"
                :title="$t('editor.deleteVersion')">
                <template #icon><n-icon>
                    <TrashIcon />
                  </n-icon></template>
              </n-button>
            </div>

            <n-divider vertical />

            <div class="tool-group">
              <n-input-number size="small" v-model:value="shiftAmount" :step="0.5" placeholder="0.0"
                style="width: 100px" />
              <n-button size="small" type="primary" secondary @click="applyShift">{{ $t('editor.shift') }}</n-button>
            </div>

            <div class="tool-group" style="margin-left: auto;">
              <n-input size="small" v-model:value="searchQuery" :placeholder="$t('editor.search')" clearable
                style="width: 200px" />
            </div>
          </div>

          <div class="video-section">
            <div class="video-wrapper" @mouseenter="showControls = true" @mouseleave="showControls = false">
              <video ref="videoRef" class="video-player" :poster="posterSrc" @timeupdate="onTimeUpdate"
                @loadedmetadata="onVideoLoaded" @click="togglePlay" @play="isPlaying = true" @pause="isPlaying = false">
                <source v-if="projectStore.currentProject?.source_video" :src="videoSrc" type="video/mp4" />
              </video>

              <div class="subtitle-overlay" v-if="activeSubtitleText"
                :style="subtitleStore.format === 'ass' ? assContainerStyle : {}">
                <span class="subtitle-text" :style="subtitleStore.format === 'ass' ? assOverlayStyle : {}"
                  v-html="formattedOverlayText"></span>
              </div>

              <div class="custom-controls" :class="{ 'controls-visible': showControls || !isPlaying }">
                <div class="controls-progress" @click="seekVideo">
                  <div class="progress-filled" :style="{ width: playheadPos + '%' }"></div>
                </div>
                <div class="controls-row">
                  <div class="controls-left">
                    <n-button text class="control-btn" @click="togglePlay">
                      <n-icon size="24">
                        <PauseIcon v-if="isPlaying" />
                        <PlayIcon v-else />
                      </n-icon>
                    </n-button>
                    <span class="time-display">{{ formatTimeSmall(currentTime) }} / {{ formatTimeSmall(videoDuration)
                    }}</span>
                  </div>
                  <div class="controls-right">
                    <n-button text class="control-btn" @click="toggleMute">
                      <n-icon size="20">
                        <MuteIcon v-if="isMuted" />
                        <VolumeIcon v-else />
                      </n-icon>
                    </n-button>
                  </div>
                </div>
              </div>
            </div>

            <div class="timeline-container"
              :style="{ backgroundImage: waveformSrc ? `url(${waveformSrc})` : 'none', backgroundSize: '100% 100%', backgroundRepeat: 'no-repeat' }">
              <div class="timeline-ruler" ref="timelineRef" @click="seekToTimeline">
                <div v-for="line in subtitleStore.lines" :key="line.id" class="timeline-block"
                  v-memo="[line.start, line.end, isLineActive(line), videoDuration]" :style="getBlockStyle(line)"
                  :class="{ active: isLineActive(line) }" @click.stop="focusLine(line.id)">
                </div>
                <div class="timeline-playhead" :style="{ left: playheadPos + '%' }"></div>
              </div>
            </div>

            <div class="zoom-timeline-header">
              <span class="zoom-label">{{ $t('editor.detailedTimeline') }}</span>
              <div class="zoom-controls">
                <n-button size="tiny" tertiary @click="zoomScale = Math.max(10, zoomScale - 50)">-</n-button>
                <span class="zoom-value">{{ Math.round(zoomScale) }}px/s</span>
                <n-button size="tiny" tertiary @click="zoomScale += 50">+</n-button>
              </div>
            </div>

            <div class="zoom-timeline-container" ref="zoomTimelineContainerRef" @wheel.prevent="handleTimelineScroll">
              <div class="zoom-timeline-content" :style="{ width: zoomedTimelineWidth + 'px', backgroundImage: waveformSrc ? `url(${waveformSrc})` : 'none', backgroundSize: '100% 100%', backgroundRepeat: 'no-repeat' }" @click="seekToZoomTimeline">
                <div v-for="line in subtitleStore.lines" :key="'z_'+line.id" class="zoom-timeline-block"
                  v-memo="[line.start, line.end, isLineActive(line), selectedLineId === line.id, zoomScale]"
                  :style="getZoomBlockStyle(line)"
                  :class="{ active: isLineActive(line), selected: selectedLineId === line.id }"
                  @click.stop="focusLine(line.id)"
                  @mousedown.stop="startDrag($event, line, 'move')">
                  
                  <div class="drag-handle left" @mousedown.stop="startDrag($event, line, 'start')"></div>
                  <div class="drag-handle right" @mousedown.stop="startDrag($event, line, 'end')"></div>
                  
                  <span class="zoom-block-text">{{ line.text }}</span>
                </div>
                <div class="timeline-playhead" :style="{ left: zoomedPlayheadPos + 'px' }"></div>
              </div>
            </div>
          </div>


        </div>

        <div class="right-panel">

          <div v-if="subtitleStore.format === 'ass'" class="ass-toolbar"
            style="flex-direction: column; align-items: flex-start; gap: 12px;">
            <div class="ass-toolbar-group">
              <span class="ass-toolbar-label">{{ $t('editor.globalStyle') }}</span>
              <n-select size="small" :options="globalStyleOptions" v-model:value="selectedGlobalStyle"
                style="width: 160px" placeholder="Select style..." />
              <n-button size="small" type="primary" @click="applyGlobalStyle" :disabled="!selectedGlobalStyle">{{ $t('editor.applyStyle') }}</n-button>
              <n-button size="small" tertiary @click="openStyleEditor" :disabled="!selectedGlobalStyle">
                <template #icon><n-icon>
                    <EditIcon />
                  </n-icon></template>
                {{ $t('editor.editStyle') }}
              </n-button>
              <n-button size="small" tertiary @click="openNewStyleEditor">
                <template #icon><n-icon>
                    <AddIcon />
                  </n-icon></template>
                {{ $t('editor.newStyle') }}
              </n-button>
            </div>

            <div class="ass-toolbar-group">
              <span class="ass-toolbar-label">{{ $t('editor.format') }}</span>
              <n-button size="small" strong secondary @mousedown.prevent="applyInlineTag('b')" title="Bold (Ctrl+B)">
                <strong style="font-family: serif; font-size: 14px">B</strong>
              </n-button>
              <n-button size="small" strong secondary @mousedown.prevent="applyInlineTag('i')" title="Italic (Ctrl+I)">
                <em style="font-family: serif; font-size: 14px">I</em>
              </n-button>
              <n-button size="small" strong secondary @mousedown.prevent="applyInlineTag('u')" title="Underline (Ctrl+U)">
                <span style="font-family: serif; font-size: 14px; text-decoration: underline;">U</span>
              </n-button>
              <n-button size="small" strong secondary @mousedown.prevent="applyInlineTag('s')" title="Strikeout">
                <span style="font-family: serif; font-size: 14px; text-decoration: line-through;">S</span>
              </n-button>
            </div>
          </div>

          <div class="table-section">
            <div class="table-header">
              <div class="col-id">#</div>
              <div class="col-time">{{ $t('editor.table.start') }}</div>
              <div class="col-time">{{ $t('editor.table.end') }}</div>
              <div class="col-style" v-if="subtitleStore.format === 'ass'">{{ $t('editor.table.style') }}</div>
              <div class="col-text">{{ $t('editor.table.text') }}</div>
              <div class="col-actions">{{ $t('editor.table.actions') }}</div>
            </div>

            <div class="table-body">
              <n-virtual-list style="height: 100%; width: 100%; max-height: 100%;" :items="filteredLines" :item-size="52" key-field="id">
                <template #default="{ item: line, index }">
                  <div class="table-row"
                    :class="{ active: isLineActive(line), selected: selectedLineId === line.id }"
                    @click="selectLine(line.id, line.start)" :id="'row-' + line.id">
                    <div class="col-id">{{ index + 1 }}</div>

                    <div class="col-time">
                      <input class="time-input" type="text" :value="formatTime(line.start)"
                        @blur="e => updateTime(line.id, 'start', (e.target as HTMLInputElement).value)" />
                    </div>

                    <div class="col-time">
                      <input class="time-input" type="text" :value="formatTime(line.end)"
                        @blur="e => updateTime(line.id, 'end', (e.target as HTMLInputElement).value)" />
                    </div>

                    <div class="col-style" v-if="subtitleStore.format === 'ass'">
                      <n-select size="tiny" v-model:value="line.style" :options="globalStyleOptions"
                        @update:value="triggerAutoSave" />
                    </div>

                    <div class="col-text">
                      <textarea class="text-input" :value="line.text"
                        @input="e => { line.text = (e.target as HTMLTextAreaElement).value; triggerAutoSave() }"
                        @blur="subtitleStore.saveState()" rows="2"></textarea>
                    </div>

                    <div class="col-actions">
                      <n-button size="tiny" tertiary @click.stop="subtitleStore.splitLine(line.id)" :title="$t('editor.actions.split')">
                        <template #icon><n-icon>
                            <CutIcon />
                          </n-icon></template>
                      </n-button>
                      <n-button size="tiny" tertiary @click.stop="subtitleStore.mergeLine(line.id)" :title="$t('editor.actions.mergeNext')">
                        <template #icon><n-icon>
                            <LinkIcon />
                          </n-icon></template>
                      </n-button>
                      <n-button size="tiny" type="error" tertiary @click.stop="deleteLine(line.id)" :title="$t('editor.actions.deleteLine')">
                        <template #icon><n-icon>
                            <TrashIcon />
                          </n-icon></template>
                      </n-button>
                    </div>
                  </div>
                </template>
              </n-virtual-list>
            </div>
          </div>
        </div>
      </div>
    </template>

    <StyleEditorModal v-model:show="showStyleEditor" :style-to-edit="editingStyle" @saved="onStyleSaved" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { appDataDir, join } from '@tauri-apps/api/path'
import { open } from '@tauri-apps/plugin-dialog'
import { NButton, NIcon, NDivider, NInputNumber, NInput, useMessage, NSelect, useDialog, NVirtualList } from 'naive-ui'
import StyleEditorModal from '../components/StyleEditorModal.vue'
import {
  ArrowUndoOutline as UndoIcon,
  ArrowRedoOutline as RedoIcon,
  SaveOutline as SaveIcon,
  PlayOutline as PlayIcon,
  PauseOutline as PauseIcon,
  VolumeHighOutline as VolumeIcon,
  VolumeMuteOutline as MuteIcon,
  DocumentTextOutline as ImportIcon,
  TrashOutline as TrashIcon,
  CutOutline as CutIcon,
  LinkOutline as LinkIcon,
  CreateOutline as EditIcon,
  AddOutline as AddIcon
} from '@vicons/ionicons5'
import { useProjectStore } from '../stores/project'
import { useSubtitleStore } from '../stores/subtitle'
import { secondsToTime, timeToSeconds, assColorToHex } from '../utils/subtitleParser'
import { SubtitleManager } from '../services/subtitleManager'
import type { SubtitleVersion } from '../services/subtitleManager'
import { MediaService } from '../services/mediaService'
import { useStylesStore } from '../stores/styles'
import { useI18n } from 'vue-i18n'

const message = useMessage()
const dialog = useDialog()
const projectStore = useProjectStore()
const subtitleStore = useSubtitleStore()
const stylesStore = useStylesStore()
const { t } = useI18n()

onMounted(async () => {
  await stylesStore.loadStyles()
})

const globalStyleOptions = computed(() => {
  return stylesStore.styles.map(s => ({
    label: s.Name,
    value: s.Name
  }))
})
const selectedGlobalStyle = ref<string | null>(null)

const applyGlobalStyle = () => {
  if (!selectedGlobalStyle.value) return
  for (const line of subtitleStore.lines) {
    line.style = selectedGlobalStyle.value
  }
  triggerAutoSave()
  message.success(t('editor.messages.styleApplied', { name: selectedGlobalStyle.value }))
}

const selectedVersionPath = ref<string | null>(null)

const showStyleEditor = ref(false)
const editingStyle = ref<any>(null)

const openStyleEditor = () => {
  if (selectedGlobalStyle.value) {
    const styleObj = stylesStore.styles.find(s => s.Name === selectedGlobalStyle.value)
    if (styleObj) {
      editingStyle.value = styleObj
      showStyleEditor.value = true
    }
  }
}

const openNewStyleEditor = () => {
  editingStyle.value = null
  showStyleEditor.value = true
}

const onStyleSaved = async () => {
  showStyleEditor.value = false
  await stylesStore.loadStyles()
  triggerAutoSave()
}

const availableVersions = ref<SubtitleVersion[]>([])

const versionOptions = computed(() => {
  return availableVersions.value.map(v => ({
    label: `${v.category.toUpperCase()} v${v.version} (${v.filename})`,
    value: v.path
  }))
})

const loadSubtitleVersions = async () => {
  if (!projectStore.currentProject) return
  availableVersions.value = await SubtitleManager.getProjectSubtitles(projectStore.currentProject.id)

  if (availableVersions.value.length > 0) {
    if (!selectedVersionPath.value || !availableVersions.value.some(v => v.path === selectedVersionPath.value)) {
      selectedVersionPath.value = availableVersions.value[0].path
    }
  } else {
    selectedVersionPath.value = null
  }
}

const onVersionSelected = async (path: string) => {
  if (path) {
    await subtitleStore.loadFromFile(path)
  }
}

const videoRef = ref<HTMLVideoElement | null>(null)
const timelineRef = ref<HTMLDivElement | null>(null)
const currentTime = ref(0)
const videoDuration = ref(0)
const selectedLineId = ref<string | null>(null)
const searchQuery = ref('')
const shiftAmount = ref(0)
const isSaving = ref(false)
const posterSrc = ref('')
const waveformSrc = ref('')

const deleteLine = (id: string) => {
  subtitleStore.deleteLine(id)
  if (selectedLineId.value === id) {
    selectedLineId.value = null
  }
}

const deleteSelected = () => {
  if (selectedLineId.value) {
    deleteLine(selectedLineId.value)
  }
}

const deleteSubtitleFile = () => {
  if (!selectedVersionPath.value) return

  dialog.warning({
    title: t('editor.messages.confirmDeleteTitle'),
    content: t('editor.messages.confirmDeleteText'),
    positiveText: t('editor.messages.deleteBtn'),
    negativeText: t('editor.messages.cancelBtn'),
    onPositiveClick: async () => {
      try {
        if (!selectedVersionPath.value) return
        await invoke('delete_file', { path: selectedVersionPath.value })
        message.success(t('editor.messages.deleteSuccess'))
        await loadSubtitleVersions()
        if (availableVersions.value.length === 0) {
          subtitleStore.lines = []
        }
      } catch (e) {
        message.error(t('editor.messages.deleteFail') + e)
      }
    }
  })
}

const assOverlayStyle = computed(() => {
  if (subtitleStore.format !== 'ass') return {};

  const activeLine = subtitleStore.lines.find(l => l.start <= currentTime.value && l.end >= currentTime.value);
  const styleName = activeLine?.style || 'Default';
  const style = stylesStore.styles.find(s => s.Name === styleName) || stylesStore.styles.find(s => s.Name === 'Default') || {} as any;

  const outlineSize = style.Outline || '2';
  const shadowSize = style.Shadow || '2';
  return {
    fontFamily: style.Fontname ? `"${style.Fontname}", sans-serif` : 'Arial, sans-serif',
    fontSize: (parseInt(style.Fontsize) || 24) + 'px',
    color: assColorToHex(style.PrimaryColour),
    fontWeight: style.Bold === '1' || style.Bold === '-1' ? 'bold' : 'normal',
    fontStyle: style.Italic === '1' || style.Italic === '-1' ? 'italic' : 'normal',
    WebkitTextStroke: `${outlineSize}px ${assColorToHex(style.OutlineColour || '&H00000000')}`,
    textShadow: `${shadowSize}px ${shadowSize}px 0px ${assColorToHex(style.BackColour || '&H00000000')}`,
    backgroundColor: 'transparent',
    padding: '0'
  };
});

const assContainerStyle = computed(() => {
  if (subtitleStore.format !== 'ass') return {};

  const activeLine = subtitleStore.lines.find(l => l.start <= currentTime.value && l.end >= currentTime.value);
  const styleName = activeLine?.style || 'Default';
  const style = stylesStore.styles.find(s => s.Name === styleName) || stylesStore.styles.find(s => s.Name === 'Default') || {} as any;

  const align = parseInt(style.Alignment) || 2;
  const marginV = parseInt(style.MarginV) || 10;
  const marginL = parseInt(style.MarginL) || 10;
  const marginR = parseInt(style.MarginR) || 10;

  const containerStyle: any = {
    top: '0',
    bottom: '0',
    display: 'flex',
    flexDirection: 'column',
    boxSizing: 'border-box',
    paddingLeft: `${marginL}px`,
    paddingRight: `${marginR}px`,
    paddingTop: `${marginV}px`,
    paddingBottom: `${marginV}px`
  };

  if ([7, 8, 9].includes(align)) {
    containerStyle.justifyContent = 'flex-start';
  } else if ([4, 5, 6].includes(align)) {
    containerStyle.justifyContent = 'center';
  } else {
    containerStyle.justifyContent = 'flex-end';
  }

  if ([1, 4, 7].includes(align)) {
    containerStyle.alignItems = 'flex-start';
    containerStyle.textAlign = 'left';
  } else if ([3, 6, 9].includes(align)) {
    containerStyle.alignItems = 'flex-end';
    containerStyle.textAlign = 'right';
  } else {
    containerStyle.alignItems = 'center';
    containerStyle.textAlign = 'center';
  }

  return containerStyle;
});

const isPlaying = ref(false)
const showControls = ref(true)
const isMuted = ref(false)

const videoSrc = computed(() => {
  if (projectStore.currentProject?.source_video) {
    return convertFileSrc(projectStore.currentProject.source_video)
  }
  return ''
})

watch(() => projectStore.currentProject?.id, (newId) => {
  if (newId) {
    loadSubtitleVersions()
    if (projectStore.currentProject?.source_video) {
      MediaService.generateThumbnail(projectStore.currentProject.source_video, newId)
        .then(src => posterSrc.value = src)
        .catch(e => console.warn(e))
    }
  }
}, { immediate: true })

const importSubtitle = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Subtitle',
        extensions: ['srt', 'ass']
      }]
    })

    const project = projectStore.currentProject
    if (selected && !Array.isArray(selected) && project) {
      const content = await invoke<string>('read_text_file', { path: selected })

      const ext = selected.split('.').pop()?.toLowerCase() === 'ass' ? 'ass' : 'srt'
      const outPath = await SubtitleManager.saveNewSubtitle(project.id, 'imported', content, ext)

      await loadSubtitleVersions()
      selectedVersionPath.value = outPath

      const success = await subtitleStore.loadFromFile(outPath)
      if (success) {
        message.success(t('editor.messages.importSuccess'))
      } else {
        message.error(t('editor.messages.importFailParse'))
      }
    }
  } catch (e) {
    message.error(t('editor.messages.importFail'))
    console.error(e)
  }
}

const playheadPos = computed(() => {
  if (videoDuration.value === 0) return 0
  return (currentTime.value / videoDuration.value) * 100
})

const getBlockStyle = (line: any) => {
  if (videoDuration.value === 0) return {}
  const left = (line.start / videoDuration.value) * 100
  const width = ((line.end - line.start) / videoDuration.value) * 100
  return {
    left: `${left}%`,
    width: `${width}%`
  }
}

const zoomTimelineContainerRef = ref<HTMLElement | null>(null)
const zoomScale = ref(100)

const zoomedTimelineWidth = computed(() => videoDuration.value * zoomScale.value)
const zoomedPlayheadPos = computed(() => currentTime.value * zoomScale.value)

const getZoomBlockStyle = (line: any) => {
  return {
    left: `${line.start * zoomScale.value}px`,
    width: `${(line.end - line.start) * zoomScale.value}px`
  }
}

const seekToZoomTimeline = (e: MouseEvent) => {
  if (isDragging.value) return;
  if (!videoRef.value) return;
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
  const clickX = e.clientX - rect.left;
  const targetTime = clickX / zoomScale.value;
  videoRef.value.currentTime = targetTime;
}

const handleTimelineScroll = (e: WheelEvent) => {
  if (zoomTimelineContainerRef.value) {
    const delta = e.deltaY !== 0 ? e.deltaY : e.deltaX;
    zoomTimelineContainerRef.value.scrollLeft += delta;
  }
}

const tableBodyRef = ref<HTMLElement | null>(null)

watch(currentTime, (newTime) => {
  if (zoomTimelineContainerRef.value && !isDragging.value) {
    const container = zoomTimelineContainerRef.value;
    const playheadX = newTime * zoomScale.value;
    const scrollLeft = container.scrollLeft;
    const clientWidth = container.clientWidth;
    
    if (playheadX > scrollLeft + clientWidth * 0.9 || playheadX < scrollLeft + clientWidth * 0.1) {
      container.scrollTo({ left: playheadX - clientWidth / 2, behavior: 'smooth' });
    }
  }

  if (isPlaying.value && tableBodyRef.value) {
    const activeLine = subtitleStore.lines.find(l => newTime >= l.start && newTime <= l.end)
    if (activeLine) {
      const activeEl = document.getElementById('row-' + activeLine.id)
      if (activeEl) {
        const container = tableBodyRef.value
        const topPos = activeEl.offsetTop
        const scrollTarget = topPos - container.clientHeight / 2 + activeEl.clientHeight / 2
        if (Math.abs(container.scrollTop - scrollTarget) > 50) {
          container.scrollTo({ top: scrollTarget, behavior: 'smooth' })
        }
      }
    }
  }
})

const isDragging = ref(false)
const dragType = ref<'start' | 'end' | 'move' | null>(null)
const dragLine = ref<any>(null)
const dragStartX = ref(0)
const dragInitialStart = ref(0)
const dragInitialEnd = ref(0)

const startDrag = (e: MouseEvent, line: any, type: 'start' | 'end' | 'move') => {
  isDragging.value = true;
  dragType.value = type;
  dragLine.value = line;
  dragStartX.value = e.clientX;
  dragInitialStart.value = line.start;
  dragInitialEnd.value = line.end;
  
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
}

const onMouseMove = (e: MouseEvent) => {
  if (!isDragging.value || !dragLine.value) return;
  
  const deltaX = e.clientX - dragStartX.value;
  const deltaTime = deltaX / zoomScale.value;
  
  if (dragType.value === 'start') {
    const newStart = Math.max(0, dragInitialStart.value + deltaTime);
    if (newStart < dragLine.value.end - 0.1) {
      dragLine.value.start = newStart;
    }
  } else if (dragType.value === 'end') {
    const newEnd = Math.min(videoDuration.value, dragInitialEnd.value + deltaTime);
    if (newEnd > dragLine.value.start + 0.1) {
      dragLine.value.end = newEnd;
    }
  } else if (dragType.value === 'move') {
    const duration = dragInitialEnd.value - dragInitialStart.value;
    let newStart = dragInitialStart.value + deltaTime;
    if (newStart < 0) newStart = 0;
    if (newStart + duration > videoDuration.value) newStart = videoDuration.value - duration;
    
    dragLine.value.start = newStart;
    dragLine.value.end = newStart + duration;
  }
}

const onMouseUp = () => {
  if (!isDragging.value) return;
  isDragging.value = false;
  dragType.value = null;
  dragLine.value = null;
  
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('mouseup', onMouseUp);
  
  subtitleStore.saveState();
  triggerAutoSave();
}

const isLineActive = (line: any) => {
  return currentTime.value >= line.start && currentTime.value <= line.end
}

const activeSubtitleText = computed(() => {
  const activeLine = subtitleStore.lines.find((l: any) => isLineActive(l))
  return activeLine ? activeLine.text : ''
})

const formattedOverlayText = computed(() => {
  let text = activeSubtitleText.value
  text = text.replace(/\{\\b1\}/g, '<b style="font-weight:bold">').replace(/\{\\b0\}/g, '</b>')
  text = text.replace(/\{\\i1\}/g, '<i style="font-style:italic">').replace(/\{\\i0\}/g, '</i>')
  text = text.replace(/\{\\u1\}/g, '<u style="text-decoration:underline">').replace(/\{\\u0\}/g, '</u>')
  text = text.replace(/\{\\s1\}/g, '<s style="text-decoration:line-through">').replace(/\{\\s0\}/g, '</s>')
  return text.replace(/\n/g, '<br/>')
})

const filteredLines = computed(() => {
  if (!searchQuery.value) return subtitleStore.lines
  const q = searchQuery.value.toLowerCase()
  return subtitleStore.lines.filter((l: any) => l.text.toLowerCase().includes(q))
})

const formatTime = (sec: number) => secondsToTime(sec, 'srt')

const formatTimeSmall = (sec: number) => {
  const m = Math.floor(sec / 60)
  const s = Math.floor(sec % 60)
  return `${m}:${s.toString().padStart(2, '0')}`
}

const applyInlineTag = (tag: 'b' | 'i' | 'u' | 's') => {
  const activeEl = document.activeElement as HTMLTextAreaElement;
  if (activeEl && activeEl.tagName === 'TEXTAREA') {
    const start = activeEl.selectionStart;
    const end = activeEl.selectionEnd;
    const val = activeEl.value;

    const openTag = `{\\${tag}1}`;
    const closeTag = `{\\${tag}0}`;

    let newVal = val;
    let newCursorPos = start;

    if (start !== end) {
      newVal = val.substring(0, start) + openTag + val.substring(start, end) + closeTag + val.substring(end);
      newCursorPos = end + openTag.length + closeTag.length;
    } else {
      newVal = val.substring(0, start) + openTag + closeTag + val.substring(start);
      newCursorPos = start + openTag.length;
    }

    activeEl.value = newVal;
    activeEl.dispatchEvent(new Event('input', { bubbles: true }));

    setTimeout(() => {
      activeEl.focus();
      activeEl.setSelectionRange(newCursorPos, newCursorPos);
    }, 0);
  }
}

const onVideoLoaded = () => {
  if (videoRef.value) {
    videoDuration.value = videoRef.value.duration
    videoRef.value.volume = isMuted.value ? 0 : 1
  }
}

const togglePlay = () => {
  if (videoRef.value) {
    if (videoRef.value.paused) {
      videoRef.value.play()
      isPlaying.value = true
    } else {
      videoRef.value.pause()
      isPlaying.value = false
    }
  }
}

const toggleMute = () => {
  if (videoRef.value) {
    isMuted.value = !isMuted.value
    videoRef.value.volume = isMuted.value ? 0 : 1
  }
}

const seekVideo = (e: MouseEvent) => {
  if (!videoRef.value || videoDuration.value === 0) return
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const pos = (e.clientX - rect.left) / rect.width
  videoRef.value.currentTime = pos * videoDuration.value
}

const onTimeUpdate = () => {
  if (videoRef.value) {
    currentTime.value = videoRef.value.currentTime
  }
}

const seekToTimeline = (e: MouseEvent) => {
  if (!timelineRef.value || !videoRef.value || videoDuration.value === 0) return
  const rect = timelineRef.value.getBoundingClientRect()
  const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width))
  const ratio = x / rect.width
  videoRef.value.currentTime = ratio * videoDuration.value
}

const selectLine = (id: string, start: number) => {
  selectedLineId.value = id
  if (videoRef.value) {
    videoRef.value.currentTime = start
  }
}

const focusLine = (id: string) => {
  const line = subtitleStore.lines.find((l: any) => l.id === id)
  if (line) selectLine(id, line.start)
  const el = document.getElementById('row-' + id)
  if (el) el.scrollIntoView({ behavior: 'smooth', block: 'center' })
}

const updateTime = (id: string, field: 'start' | 'end', val: string) => {
  const sec = timeToSeconds(val)
  if (field === 'start') {
    subtitleStore.updateLine(id, subtitleStore.lines.find((l: any) => l.id === id)?.text || '', sec, undefined)
  } else {
    subtitleStore.updateLine(id, subtitleStore.lines.find((l: any) => l.id === id)?.text || '', undefined, sec)
  }
  triggerAutoSave()
}

const applyShift = () => {
  if (shiftAmount.value === 0) return
  subtitleStore.shiftTiming(shiftAmount.value)
  message.success(t('editor.messages.shifted', { amount: shiftAmount.value }))
  triggerAutoSave()
}

let saveTimeout: any = null
const triggerAutoSave = () => {
  if (saveTimeout) clearTimeout(saveTimeout)
  saveTimeout = setTimeout(async () => {
    await subtitleStore.saveToFile()
  }, 2000)
}

const manualSave = async () => {
  isSaving.value = true
  const success = await subtitleStore.saveToFile()
  isSaving.value = false
  if (success) message.success(t('editor.messages.saveSuccess'))
  else message.error(t('editor.messages.saveFail'))
}

const handleKeydown = (e: KeyboardEvent) => {
  const target = e.target as HTMLElement
  const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA'

  if ((e.ctrlKey || e.metaKey) && e.key === 'z') {
    e.preventDefault()
    subtitleStore.undo()
    triggerAutoSave()
  }
  if (((e.ctrlKey || e.metaKey) && e.key === 'y') || ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'z')) {
    e.preventDefault()
    subtitleStore.redo()
    triggerAutoSave()
  }

  if (isInput && subtitleStore.format === 'ass') {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'b') {
      e.preventDefault();
      applyInlineTag('b');
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'i') {
      e.preventDefault();
      applyInlineTag('i');
    }
  }

  if (!isInput) {
    if (e.key === 'Tab') {
      e.preventDefault()
      if (videoRef.value) {
        if (videoRef.value.paused) videoRef.value.play()
        else videoRef.value.pause()
      }
    }

    if (e.key === 'Delete' || e.key === 'Backspace') {
      e.preventDefault()
      if (selectedLineId.value) {
        deleteSelected()
      }
    }
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeydown)

  if (projectStore.currentProject) {
    try {
      const dir = await appDataDir()
      const thumbPath = await join(dir, 'thumbnails', `${projectStore.currentProject.id}.jpg`)
      posterSrc.value = convertFileSrc(thumbPath)

      const wavePath = await join(dir, 'waveforms', `${projectStore.currentProject.id}.png`)
      waveformSrc.value = convertFileSrc(wavePath)
    } catch (e) {
      console.warn("Failed to set poster or waveform", e)
    }
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.editor-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 500px;
  gap: 16px;
  max-width: 100%;
  margin: 0 auto;
}

.empty-state-wrapper {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 60vh;
  background-color: #1e293b;
  border-radius: 12px;
  border: 1px dashed #334155;
}

.editor-layout {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: 16px;
  flex: 1;
  min-height: 0;
}

.left-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 0;
  min-width: 0;
}

.right-panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
  min-width: 0;
}

@media (max-width: 1000px) {
  .editor-layout {
    grid-template-columns: 1fr;
    height: auto;
  }

  .left-panel,
  .right-panel {
    min-height: auto;
  }

  .video-section {
    height: 350px;
  }
}

/* Video Section */
.video-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  background-color: #0f172a;
  border-radius: 8px;
  padding: 12px;
  border: 1px solid #334155;
  min-width: 0;
  min-height: 0;
}

.video-wrapper {
  position: relative;
  background-color: #000;
  border-radius: 8px;
  overflow: hidden;
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.video-player {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

/* Custom Controls */
.custom-controls {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  background: linear-gradient(to top, rgba(0, 0, 0, 0.8), transparent);
  padding: 20px 16px 16px;
  opacity: 0;
  transition: opacity 0.3s ease;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.custom-controls.controls-visible {
  opacity: 1;
}

.controls-progress {
  width: 100%;
  height: 4px;
  background-color: rgba(255, 255, 255, 0.3);
  cursor: pointer;
  border-radius: 2px;
  position: relative;
}

.controls-progress:hover {
  height: 6px;
}

.progress-filled {
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  background-color: #6366f1;
  border-radius: 2px;
}

.controls-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.controls-left,
.controls-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.control-btn {
  color: white !important;
}

.control-btn:hover {
  color: #6366f1 !important;
}

.time-display {
  color: white;
  font-size: 13px;
  font-family: monospace;
}

.subtitle-overlay {
  position: absolute;
  bottom: 10%;
  left: 0;
  right: 0;
  text-align: center;
  pointer-events: none;
}

.subtitle-text {
  background-color: rgba(0, 0, 0, 0.7);
  color: #fff;
  font-size: 24px;
  padding: 4px 12px;
  border-radius: 4px;
  font-family: Arial, sans-serif;
  line-height: 1.4;
  display: inline-block;
  text-shadow: 1px 1px 2px #000;
}

/* Timeline */
.timeline-container {
  height: 32px;
  background-color: #1e293b;
  border-radius: 6px;
  position: relative;
  cursor: pointer;
}

.timeline-ruler {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  overflow: hidden;
}

.timeline-block {
  position: absolute;
  height: 16px;
  top: 8px;
  background-color: #3b82f6;
  border-radius: 2px;
  opacity: 0.7;
}

.timeline-block.active {
  background-color: #f59e0b;
  opacity: 1;
}

.timeline-playhead {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 2px;
  background-color: #ef4444;
  z-index: 10;
}

/* Toolbar Section */
.toolbar-section {
  display: flex;
  align-items: center;
  gap: 12px;
  background-color: #1e293b;
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid #334155;
  flex-shrink: 0;
  flex-wrap: wrap;
}

.tool-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.flex-1 {
  flex: 1;
}

/* ASS Toolbar */
.ass-toolbar {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  gap: 16px;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.ass-toolbar-label {
  font-size: 12px;
  color: #94a3b8;
  font-weight: 600;
  text-transform: uppercase;
}

.ass-toolbar-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.color-picker-wrapper {
  width: 28px;
  height: 28px;
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid #334155;
}

.native-color-picker {
  width: 150%;
  height: 150%;
  margin: -25%;
  cursor: pointer;
  border: none;
  padding: 0;
}

.style-btn {
  font-family: serif;
}

.table-section {
  flex: 1;
  background-color: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  overflow-x: auto;
  overflow-y: hidden;
  margin-bottom: 0;
  min-height: 250px;
}

.table-header {
  display: flex;
  flex-shrink: 0;
  min-width: 600px;
  padding: 12px 16px;
  background-color: #0f172a;
  border-bottom: 1px solid #334155;
  font-weight: 600;
  color: #94a3b8;
  font-size: 14px;
}

.table-body {
  flex: 1;
  overflow-y: auto;
  min-width: 600px;
  position: relative;
}

.table-row {
  display: flex;
  padding: 8px 16px;
  border-bottom: 1px solid #334155;
  align-items: center;
  transition: background-color 0.2s;
  cursor: pointer;
}

.table-row:hover {
  background-color: #334155;
}

.table-row.active {
  background-color: rgba(245, 158, 11, 0.1);
  border-left: 3px solid #f59e0b;
  padding-left: 13px;
}

.table-row.selected {
  background-color: rgba(99, 102, 241, 0.1);
}

.col-id {
  width: 50px;
  color: #64748b;
}

.col-time {
  width: 110px;
}

.col-style {
  width: 130px;
  padding: 0 8px;
}

.col-text {
  flex: 1;
  padding: 0 16px;
}

.col-actions {
  width: 100px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.time-input {
  background: transparent;
  border: 1px solid transparent;
  color: #e2e8f0;
  width: 90px;
  padding: 4px;
  border-radius: 4px;
  font-family: monospace;
}

.time-input:focus {
  border-color: #6366f1;
  outline: none;
  background: #0f172a;
}

.text-input {
  width: 100%;
  background: transparent;
  border: 1px solid transparent;
  color: #e2e8f0;
  padding: 6px;
  border-radius: 4px;
  resize: none;
  font-family: inherit;
}

.text-input:focus {
  border-color: #6366f1;
  outline: none;
  background: #0f172a;
}
/* Zoomable Timeline */
.zoom-timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 16px;
  margin-bottom: 8px;
}

.zoom-label {
  font-size: 12px;
  color: #94a3b8;
  font-weight: 600;
  text-transform: uppercase;
}

.zoom-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.zoom-value {
  font-size: 12px;
  color: #fff;
  min-width: 50px;
  text-align: center;
}

.zoom-timeline-container {
  height: 64px;
  width: 100%;
  background-color: #1e293b;
  border-radius: 6px;
  position: relative;
  overflow-x: scroll;
  overflow-y: hidden;
  border: 1px solid #334155;
}

.zoom-timeline-content {
  height: 100%;
  position: relative;
  cursor: text;
}

.zoom-timeline-block {
  position: absolute;
  height: 32px;
  top: 16px;
  background-color: #3b82f6;
  border-radius: 4px;
  opacity: 0.8;
  border: 1px solid #2563eb;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
  user-select: none;
}

.zoom-timeline-block:active {
  cursor: grabbing;
}

.zoom-timeline-block.active {
  background-color: #f59e0b;
  border-color: #d97706;
  opacity: 1;
}

.zoom-timeline-block.selected {
  outline: 2px solid #fff;
  z-index: 5;
}

.zoom-block-text {
  font-size: 11px;
  color: white;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  padding: 0 8px;
  pointer-events: none;
}

.drag-handle {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 8px;
  background-color: rgba(255, 255, 255, 0.2);
  z-index: 10;
}

.drag-handle:hover {
  background-color: rgba(255, 255, 255, 0.8);
}

.drag-handle.left {
  left: 0;
  cursor: ew-resize;
  border-top-left-radius: 4px;
  border-bottom-left-radius: 4px;
}

.drag-handle.right {
  right: 0;
  cursor: ew-resize;
  border-top-right-radius: 4px;
  border-bottom-right-radius: 4px;
}

/* Hide Scrollbar for Zoom Timeline */
.zoom-timeline-container::-webkit-scrollbar {
  display: none;
}

.zoom-timeline-container {
  -ms-overflow-style: none;  /* IE and Edge */
  scrollbar-width: none;  /* Firefox */
}
</style>
