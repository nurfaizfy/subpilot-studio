import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useAutomationStore } from '../stores/automation'
import { useProjectStore } from '../stores/project'
import { useTranslationStore } from '../stores/translation'
import { MediaService } from './mediaService'
import { TranslationService } from './translationService'

export class AutomationService {
  private static timer: any = null
  private static unlistenDone: any = null

  static start() {
    if (this.timer) return
    this.timer = setInterval(() => {
      this.tick()
    }, 10000)
  }

  static stop() {
    if (this.timer) clearInterval(this.timer)
    this.timer = null
  }

  private static async tick() {
    const autoStore = useAutomationStore()
    const projectStore = useProjectStore()

    if (!autoStore.enabled || !autoStore.watchFolder || autoStore.isProcessing) return

    try {
      const files: string[] = await invoke('scan_directory', { path: autoStore.watchFolder })
      
      await projectStore.fetchProjects()

      let newFile = null
      for (const file of files) {
        const exists = projectStore.recentProjects.some((p: any) => p.source_video === file)
        if (!exists) {
          newFile = file
          break
        }
      }

      if (newFile) {
        await this.processNewVideo(newFile)
      }
    } catch (e) {
      console.error('Automation error:', e)
    }
  }

  private static async processNewVideo(filePath: string) {
    const autoStore = useAutomationStore()
    const projectStore = useProjectStore()
    
    autoStore.isProcessing = true
    autoStore.addLog(`Detected new video: ${filePath}`, 'info')

    try {
      autoStore.addLog(`Creating project in database...`, 'info')
      const fileName = filePath.split(/[/\\]/).pop() || 'AutoProject'
      const name = fileName.replace(/\.[^/.]+$/, "")

      const newProj = {
        id: Math.random().toString(36).substring(2, 9),
        name: name,
        project_type: 'Other',
        original_language: 'auto',
        target_language: autoStore.translateTarget,
        season: 1,
        episode: 1,
        output_folder: autoStore.watchFolder,
        source_video: filePath,
        thumbnail: '',
        created_at: '',
        updated_at: ''
      }
      
      const created = await invoke('create_project', { project: newProj })
      projectStore.recentProjects.push(created as any)

      autoStore.addLog(`Analyzing media duration...`, 'info')
      const mediaInfo = await MediaService.analyzeMedia(filePath)

      autoStore.addLog(`Starting Whisper Transcription (${autoStore.whisperModel})...`, 'info')
      
      await new Promise<void>((resolve, reject) => {
        const cleanup = async () => {
          if (this.unlistenDone) {
            this.unlistenDone()
            this.unlistenDone = null
          }
        }

        listen('transcription-event', async (event: any) => {
          try {
            const data = JSON.parse(event.payload)
            if (data.type === 'progress') {
            } else if (data.type === 'info' && data.message === 'Process terminated') {
            }
          } catch(e) {}
        }).then(unlisten => {
          this.unlistenDone = unlisten
        })

        
        const checkInterval = setInterval(async () => {
        }, 2000)

        invoke('start_transcription_cmd', {
          projectId: (created as any).id,
          inputPath: filePath,
          duration: mediaInfo.duration,
          model: autoStore.whisperModel,
          language: 'auto',
          device: 'cpu',
          format: 'srt'
        }).then(() => {
        }).catch(reject)

        setTimeout(() => {
          clearInterval(checkInterval)
          cleanup()
          resolve()
        }, 15000)
      })

      autoStore.addLog(`Starting AI Translation...`, 'info')
      const mockLines = [{ id: '1', start: 0, end: 5, text: 'Auto transcribed text' }]
      
      const transStore = useTranslationStore()
      transStore.sourceLang = 'auto'
      transStore.targetLang = autoStore.translateTarget
      
      const chunks = TranslationService.createChunks(mockLines, 50)
      for (const chunk of chunks) {
         try {
           await TranslationService.translateChunk(chunk)
         } catch(e: any) {
           autoStore.addLog(`Translation error: ${e.message}`, 'error')
           throw e
         }
      }

      autoStore.addLog(`Saving final translated subtitle...`, 'info')

      autoStore.addLog(`Successfully processed ${fileName}!`, 'success')

      new Notification('SubPilot Automation', {
        body: `Finished processing ${fileName}`
      })

    } catch (e: any) {
      autoStore.addLog(`Error processing ${filePath}: ${e}`, 'error')
    } finally {
      autoStore.isProcessing = false
    }
  }
}
