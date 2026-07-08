import { invoke } from '@tauri-apps/api/core'
import type { SubtitleLine } from '../utils/subtitleParser'
import { useTranslationStore } from '../stores/translation'
import { useProvidersStore } from '../stores/providers'

export interface TranslationChunk {
  id: number;
  lines: SubtitleLine[];
  status: 'pending' | 'processing' | 'success' | 'error';
  errorMessage?: string;
}

export class TranslationService {
  static createChunks(lines: SubtitleLine[], batchSize = 30): TranslationChunk[] {
    const chunks: TranslationChunk[] = []
    for (let i = 0; i < lines.length; i += batchSize) {
      chunks.push({
        id: i / batchSize,
        lines: lines.slice(i, i + batchSize),
        status: 'pending'
      })
    }
    return chunks
  }

  static async translateChunk(chunk: TranslationChunk): Promise<SubtitleLine[]> {
    const store = useTranslationStore()
    
    const providersStore = useProvidersStore()
    
    const providersToTry = [{ id: store.provider, model: store.model }]
    const fallbacks = providersStore.providers.filter(p => p.is_fallback && p.has_key && p.id !== store.provider)
    fallbacks.forEach(f => providersToTry.push({ id: f.id, model: f.default_model }))
    const sourceText = chunk.lines.map((l, idx) => `[${idx}] ${l.text}`).join('\n')
    
    let glossaryStr = ""
    if (store.glossary.length > 0) {
      glossaryStr += store.glossary.map(g => `${g.source} = ${g.target}`).join('\n') + "\n"
    }

    try {
      const globalDict: any[] = await invoke('get_dictionary')
      const matchedEntries = globalDict.filter(entry => sourceText.includes(entry.original))
      
      if (matchedEntries.length > 0) {
        matchedEntries.sort((a, b) => b.priority - a.priority)
        
        glossaryStr += "\nCRITICAL OVERRIDES (YOU MUST FOLLOW THESE STRICTLY):\n"
        matchedEntries.forEach(entry => {
          glossaryStr += `If you see '${entry.original}', you MUST translate it as '${entry.translation}'.\n`
        })
      }
    } catch (e) {
      console.warn("Could not load global dictionary", e)
    }

    const systemPrompt = `You are a professional subtitle translator. Translate from ${store.sourceLang} to ${store.targetLang}.
Strict Rules:
1. Maintain the exact same number of lines.
2. Return ONLY the translated text in the exact format: [index] translated_text
3. Do not add any conversational filler or explanations.
4. Provide a natural, contextually appropriate translation. Avoid overly harsh or literal profanity unless strictly necessary for the context.

Additional Instructions (Custom Prompt):
${store.customPrompt || 'None.'}

${glossaryStr}`
    let translatedText = ""
    let lastError: any = null

    for (const p of providersToTry) {
      try {
        let apiKey = ''
        try {
          apiKey = await invoke('get_api_key', { providerId: p.id })
        } catch (e) {
          throw new Error(`API Key for ${p.id} is missing. Configure it in Settings.`)
        }

        if (!apiKey) throw new Error(`API Key for ${p.id} is missing. Configure it in Settings.`)

        if (p.id === 'gemini') {
          const url = `https://generativelanguage.googleapis.com/v1beta/models/${p.model}:generateContent?key=${apiKey}`
          const body = JSON.stringify({
            system_instruction: { parts: [{ text: systemPrompt }] },
            contents: [{ parts: [{ text: sourceText }] }]
          })
          
          const res: string = await invoke('llm_request', { url, headers: {}, body })
          const data = JSON.parse(res)
          if (data.error) throw new Error(data.error.message)
          translatedText = data.candidates?.[0]?.content?.parts?.[0]?.text || ""

        } else if (p.id === 'openai' || p.id === 'openrouter' || p.id === 'deepseek' || p.id === 'ollama') {
          const url = providersStore.providers.find(x => x.id === p.id)?.api_url || 'https://api.openai.com/v1/chat/completions'
            
          const headers = {
            'Authorization': `Bearer ${apiKey}`
          }
          const body = JSON.stringify({
            model: p.model,
            messages: [
              { role: 'system', content: systemPrompt },
              { role: 'user', content: sourceText }
            ]
          })
          
          const res: string = await invoke('llm_request', { url, headers, body })
          const data = JSON.parse(res)
          if (data.error) throw new Error(data.error.message)
          translatedText = data.choices?.[0]?.message?.content || ""
        }

        if (translatedText.trim()) {
          lastError = null
          break
        }
      } catch (e) {
        console.warn(`Provider ${p.id} failed, trying fallback if available. Error:`, e)
        lastError = e
        continue
      }
    }

    if (lastError || !translatedText.trim()) {
      throw lastError || new Error("All fallback providers failed to return text.")
    }

    const translatedLines = translatedText.trim().split('\n')
    
    const newLines: SubtitleLine[] = JSON.parse(JSON.stringify(chunk.lines))
    
    let currentIdx = 0
    for (const line of translatedLines) {
      const match = line.match(/^\[(\d+)\]\s(.*)/)
      if (match) {
        const idx = parseInt(match[1])
        if (idx >= 0 && idx < newLines.length) {
          newLines[idx].text = match[2]
        }
      } else {
        if (currentIdx < newLines.length) {
           newLines[currentIdx].text = line.replace(/^\d+[\.\)\]]\s*/, '')
           currentIdx++
        }
      }
    }

    return newLines
  }
}
