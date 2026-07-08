import { invoke } from '@tauri-apps/api/core'
import { appDataDir, join } from '@tauri-apps/api/path'
import { useSettingsStore } from '../stores/settings'

export interface SubtitleVersion {
  path: string
  category: 'imported' | 'transcripted' | 'translated'
  version: number
  filename: string
}

export class SubtitleManager {
  static async getProjectSubtitles(projectId: string): Promise<SubtitleVersion[]> {
    try {
      const dir = await appDataDir()
      const projectSubDir = await join(dir, 'subtitles', projectId)
      
      const files: string[] = await invoke('list_directory_files', { path: projectSubDir })
      
      const versions: SubtitleVersion[] = []
      
      for (const filepath of files) {
        const filename = filepath.split('\\').pop()?.split('/').pop() || ''
        
        const match = filename.match(/^(imported|transcripted|translated)_v(\d+)\.(srt|ass)$/i)
        if (match) {
          versions.push({
            path: filepath,
            category: match[1] as any,
            version: parseInt(match[2]),
            filename
          })
        }
      }
      
      versions.sort((a, b) => b.version - a.version)
      return versions
    } catch (e) {
      return []
    }
  }

  static async saveNewSubtitle(
    projectId: string, 
    category: 'imported' | 'transcripted' | 'translated', 
    content: string, 
    ext: 'srt' | 'ass'
  ): Promise<string> {
    const settingsStore = useSettingsStore()
    const autoBackup = settingsStore.autoBackup

    let nextVersion = 1

    if (autoBackup) {
      const existing = await this.getProjectSubtitles(projectId)
      const sameCategory = existing.filter(x => x.category === category)
      if (sameCategory.length > 0) {
        nextVersion = sameCategory[0].version + 1
      }
    }

    const dir = await appDataDir()
    const projectSubDir = await join(dir, 'subtitles', projectId)
    
    const filename = `${category}_v${nextVersion}.${ext}`
    const outPath = await join(projectSubDir, filename)
    
    await invoke('write_text_file', { path: outPath, content })
    
    return outPath
  }

  static async getNextTranscriptionPrefix(projectId: string): Promise<string> {
    const settingsStore = useSettingsStore()
    const autoBackup = settingsStore.autoBackup

    let nextVersion = 1
    if (autoBackup) {
      const existing = await this.getProjectSubtitles(projectId)
      const sameCategory = existing.filter(x => x.category === 'transcripted')
      if (sameCategory.length > 0) {
        nextVersion = sameCategory[0].version + 1
      }
    }

    const dir = await appDataDir()
    const projectSubDir = await join(dir, 'subtitles', projectId)
    
    return await join(projectSubDir, `transcripted_v${nextVersion}`)
  }
}
