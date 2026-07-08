<template>
  <n-config-provider :theme="darkTheme" :theme-overrides="themeOverrides">
    <n-message-provider>
      <n-dialog-provider>
        <div v-if="!showSplash" class="app-layout">
          <Sidebar v-if="!isSetup" />
          <div class="main-container">
            <Topbar v-if="!isSetup" />
            <main class="content-area" :class="{ 'no-padding': isSetup }">
              <router-view v-slot="{ Component }">
                <keep-alive>
                  <component :is="Component" />
                </keep-alive>
              </router-view>
            </main>
            <StatusBar v-if="!isSetup" />
          </div>
        </div>
        <div v-else class="initial-loading">
          <div class="logo-container">
            <n-icon size="56" class="logo-icon">
              <FilmIcon />
            </n-icon>
          </div>
          <h2 class="loading-title">SubPilot Studio</h2>
          <p class="loading-subtitle">Initializing Engine...</p>
          <div class="loading-bar">
            <div class="loading-bar-inner"></div>
          </div>
        </div>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>
<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { darkTheme, NConfigProvider, NMessageProvider, NDialogProvider, NIcon } from 'naive-ui'
import { FilmOutline as FilmIcon } from '@vicons/ionicons5'
import Sidebar from './components/Sidebar.vue'
import Topbar from './components/Topbar.vue'
import StatusBar from './components/StatusBar.vue'
import { useRuntimeStore } from './stores/runtime'

const route = useRoute()
const router = useRouter()
const runtimeStore = useRuntimeStore()

const isSetup = computed(() => route.name === 'setup')
const showSplash = ref(true)

const themeOverrides = {
  common: {
    primaryColor: '#6366f1',
    primaryColorHover: '#818cf8',
    primaryColorPressed: '#4f46e5',
    bodyColor: '#0f172a',
    cardColor: '#1e293b'
  }
}

onMounted(async () => {
  // Start checking the runtime and wait for a minimum of 2 seconds
  const checkPromise = runtimeStore.checkRuntime()
  const delayPromise = new Promise(resolve => setTimeout(resolve, 3000))

  await Promise.all([checkPromise, delayPromise])

  if (!runtimeStore.isAllGood && route.name !== 'setup') {
    router.push('/setup')
  }

  showSplash.value = false
})
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background-color: var(--n-color);
}

.main-container {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
  margin-left: 68px;
  /* Room for collapsed sidebar */
}

.content-area {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
  background-color: #0f172a;
}

.content-area.no-padding {
  padding: 0;
}

.initial-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  width: 100vw;
  background-color: #0f172a;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
}

.logo-container {
  margin-bottom: 16px;
  animation: float 3s ease-in-out infinite;
}

.logo-icon {
  color: #818cf8;
}

@keyframes float {
  0% {
    transform: translateY(0px);
  }

  50% {
    transform: translateY(-8px);
  }

  100% {
    transform: translateY(0px);
  }
}

.loading-title {
  margin: 0 0 8px 0;
  font-size: 28px;
  font-weight: 700;
  color: #f8fafc;
  letter-spacing: -0.5px;
}

.loading-subtitle {
  margin: 0 0 32px 0;
  font-size: 14px;
  color: #94a3b8;
  font-weight: 500;
  letter-spacing: 0.5px;
}

.loading-bar {
  width: 200px;
  height: 4px;
  background-color: #1e293b;
  border-radius: 4px;
  overflow: hidden;
  position: relative;
}

.loading-bar-inner {
  position: absolute;
  top: 0;
  left: -50%;
  width: 50%;
  height: 100%;
  background-color: #6366f1;
  border-radius: 4px;
  animation: slide 1.5s ease-in-out infinite;
}

@keyframes slide {
  0% {
    left: -50%;
    width: 50%;
  }

  50% {
    left: 25%;
    width: 50%;
  }

  100% {
    left: 100%;
    width: 50%;
  }
}
</style>
