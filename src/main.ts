import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import './style.css'

import { AutomationService } from './services/automationService'

const app = createApp(App)

app.use(createPinia())
app.use(router)

AutomationService.start()

app.mount('#app')
