import { createI18n } from 'vue-i18n'
import en from './locales/en.json'
import id from './locales/id.json'

export type MessageSchema = typeof en

const i18n = createI18n<[MessageSchema], 'en-US' | 'id-ID'>({
  legacy: false, 
  locale: 'en-US', 
  fallbackLocale: 'en-US', 
  messages: {
    'en-US': en,
    'id-ID': id
  }
})

export default i18n
