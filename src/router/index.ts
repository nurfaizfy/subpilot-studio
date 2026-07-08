import { createRouter, createWebHistory } from 'vue-router'
import Dashboard from '../views/Dashboard.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'Dashboard',
      component: Dashboard
    },
    {
      path: '/projects',
      name: 'Projects',
      component: () => import('../views/Projects.vue')
    },
    {
      path: '/project/:id',
      name: 'ProjectWorkspace',
      component: () => import('../views/ProjectWorkspace.vue')
    },
    {
      path: '/dictionary',
      name: 'Dictionary',
      component: () => import('../views/Dictionary.vue')
    },
    {
      path: '/styling',
      name: 'SubtitleStyling',
      component: () => import('../views/SubtitleStyling.vue')
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/Settings.vue')
    },
    {
      path: '/automation',
      name: 'automation',
      component: () => import('../views/Automation.vue')
    },
    {
      path: '/setup',
      name: 'setup',
      component: () => import('../views/Setup.vue')
    },
    {
      path: '/diagnostics',
      name: 'diagnostics',
      component: () => import('../views/Diagnostics.vue')
    }
  ]
})

import { useSystemStore } from '../stores/system'

router.beforeEach(async (to, from, next) => {
  const systemStore = useSystemStore()

  if (systemStore.isAppBusy && to.path !== from.path) {
    if (typeof (window as any).$message !== 'undefined') {
      (window as any).$message.warning("Please wait for the current process to finish.")
    } else {
      alert("Please wait for the current process to finish before navigating.")
    }
    return next(false)
  }

  if (to.name === 'setup') {
    return next()
  }

  next()
})

export default router
