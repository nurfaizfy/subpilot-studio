import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { SubtitleParser, type SubtitleLine, type AssStyle } from '../utils/subtitleParser'

export const useSubtitleStore = defineStore('subtitle', () => {
  const lines = ref<SubtitleLine[]>([])
  
  const format = ref<'srt' | 'ass'>('srt')
  const assMetadata = ref<{ scriptInfo: string[], styleFormat: string[], eventsFormat: string[] }>({
    scriptInfo: [], styleFormat: [], eventsFormat: []
  })
  const assStyles = ref<Record<string, AssStyle>>({})

  const history = ref<SubtitleLine[][]>([])
  const historyIndex = ref(-1)

  const currentFilePath = ref<string | null>(null)

  const saveState = () => {
    if (historyIndex.value < history.value.length - 1) {
      history.value = history.value.slice(0, historyIndex.value + 1)
    }
    
    const snapshot = JSON.parse(JSON.stringify(lines.value))
    history.value.push(snapshot)
    
    if (history.value.length > 50) {
      history.value.shift()
    } else {
      historyIndex.value++
    }
  }

  const undo = () => {
    if (historyIndex.value > 0) {
      historyIndex.value--
      lines.value = JSON.parse(JSON.stringify(history.value[historyIndex.value]))
    }
  }

  const redo = () => {
    if (historyIndex.value < history.value.length - 1) {
      historyIndex.value++
      lines.value = JSON.parse(JSON.stringify(history.value[historyIndex.value]))
    }
  }


  const loadFromFile = async (path: string) => {
    try {
      const content: string = await invoke('read_text_file', { path })
      
      if (path.toLowerCase().endsWith('.ass')) {
        const parsed = SubtitleParser.parseAss(content)
        lines.value = parsed.lines
        format.value = 'ass'
        assMetadata.value = {
          scriptInfo: parsed.scriptInfo,
          styleFormat: parsed.styleFormat,
          eventsFormat: parsed.eventsFormat
        }
        assStyles.value = parsed.styles
      } else {
        const parsed = SubtitleParser.parseSrt(content)
        lines.value = parsed
        format.value = 'srt'
      }
      currentFilePath.value = path
      
      history.value = []
      historyIndex.value = -1
      saveState()
      
      return true
    } catch (e) {
      console.error(e)
      return false
    }
  }

  const saveToFile = async () => {
    if (!currentFilePath.value) return false
    
    try {
      let content = ''
      if (format.value === 'ass') {
        const { useStylesStore } = await import('./styles')
        const stylesStore = useStylesStore()
        if (stylesStore.styles.length === 0) {
          await stylesStore.loadStyles()
        }
        content = SubtitleParser.serializeAss({
          scriptInfo: assMetadata.value.scriptInfo,
          styles: assStyles.value,
          styleFormat: assMetadata.value.styleFormat,
          eventsFormat: assMetadata.value.eventsFormat,
          lines: lines.value
        }, stylesStore.styles)
      } else {
        content = SubtitleParser.serializeSrt(lines.value)
      }
      await invoke('write_text_file', { path: currentFilePath.value, content })
      return true
    } catch (e) {
      console.error(e)
      return false
    }
  }

  const updateGlobalStyle = (updates: Partial<AssStyle>) => {
    if (assStyles.value['Default']) {
      assStyles.value['Default'] = { ...assStyles.value['Default'], ...updates }
    }
  }

  const updateLine = (id: string, newText: string, newStart?: number, newEnd?: number) => {
    const idx = lines.value.findIndex(l => l.id === id)
    if (idx !== -1) {
      const oldLine = lines.value[idx]
      let changed = false
      
      if (oldLine.text !== newText) changed = true
      if (newStart !== undefined && oldLine.start !== newStart) changed = true
      if (newEnd !== undefined && oldLine.end !== newEnd) changed = true
      
      if (changed) {
        lines.value[idx].text = newText
        if (newStart !== undefined) lines.value[idx].start = newStart
        if (newEnd !== undefined) lines.value[idx].end = newEnd
        saveState()
      }
    }
  }

  const splitLine = (id: string) => {
    const idx = lines.value.findIndex(l => l.id === id)
    if (idx !== -1) {
      const line = lines.value[idx]
      const midPoint = line.start + ((line.end - line.start) / 2)
      
      const parts = line.text.split('\n')
      let text1 = line.text
      let text2 = "---"
      if (parts.length > 1) {
        text1 = parts[0]
        text2 = parts.slice(1).join('\n')
      }

      lines.value[idx].end = midPoint
      lines.value[idx].text = text1

      const newLine: SubtitleLine = {
        id: Math.random().toString(36).substring(2, 15),
        start: midPoint + 0.001,
        end: line.end,
        text: text2
      }
      lines.value.splice(idx + 1, 0, newLine)
      saveState()
    }
  }

  const mergeLine = (id: string) => {
    const idx = lines.value.findIndex(l => l.id === id)
    if (idx !== -1 && idx < lines.value.length - 1) {
      const line = lines.value[idx]
      const nextLine = lines.value[idx + 1]
      
      lines.value[idx].end = nextLine.end
      lines.value[idx].text = `${line.text}\n${nextLine.text}`
      
      lines.value.splice(idx + 1, 1)
      saveState()
    }
  }

  const deleteLine = (id: string) => {
    const idx = lines.value.findIndex(l => l.id === id)
    if (idx !== -1) {
      lines.value.splice(idx, 1)
      saveState()
    }
  }

  const shiftTiming = (amountSeconds: number) => {
    for (let i = 0; i < lines.value.length; i++) {
      lines.value[i].start = Math.max(0, lines.value[i].start + amountSeconds)
      lines.value[i].end = Math.max(0, lines.value[i].end + amountSeconds)
    }
    saveState()
  }

  return {
    lines,
    historyIndex,
    historyLength: computed(() => history.value.length),
    currentFilePath,
    format,
    assStyles,
    undo,
    redo,
    loadFromFile,
    saveToFile,
    updateLine,
    updateGlobalStyle,
    splitLine,
    mergeLine,
    deleteLine,
    shiftTiming,
    saveState
  }
})
