import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import i18n from './i18n'
import './style.css'
import { createDiscreteApi, darkTheme } from 'naive-ui'

const { message, dialog } = createDiscreteApi(['message', 'dialog'], {
  configProviderProps: { theme: darkTheme }
})

;(window as any).$message = message
;(window as any).$dialog = dialog

import { AutomationService } from './services/automationService'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(i18n)

AutomationService.start()

app.mount('#app')
