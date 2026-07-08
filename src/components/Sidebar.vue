<template>
  <aside class="sidebar">
    <div class="logo-container">
      <img :src="logoUrl" alt="Logo" class="logo-icon" />
      <h1 class="logo-text">SubPilot</h1>
    </div>
    <nav class="nav-menu">
      <router-link to="/" class="nav-item" active-class="active">
        <n-icon size="20">
          <DashboardIcon />
        </n-icon>
        <span>{{ $t('nav.dashboard') }}</span>
      </router-link>
      <router-link to="/projects" class="nav-item" active-class="active">
        <n-icon size="20">
          <FolderIcon />
        </n-icon>
        <span>{{ $t('nav.projects') }}</span>
      </router-link>
      <router-link to="/dictionary" class="nav-item" active-class="active">
        <n-icon size="20">
          <BookIcon />
        </n-icon>
        <span>{{ $t('nav.dictionary') }}</span>
      </router-link>
      <router-link to="/styling" class="nav-item" active-class="active">
        <n-icon size="20">
          <ColorPaletteIcon />
        </n-icon>
        <span>{{ $t('nav.styling') }}</span>
      </router-link>
      <router-link to="/automation" class="nav-item" active-class="active">
        <n-icon size="20">
          <FlashIcon />
        </n-icon>
        <span>{{ $t('nav.automation') }}</span>
      </router-link>

      <router-link to="/diagnostics" class="nav-item" active-class="active">
        <n-icon size="20">
          <MedkitIcon />
        </n-icon>
        <span>{{ $t('nav.diagnostics') }}</span>
      </router-link>

      <router-link to="/settings" class="nav-item" active-class="active">
        <n-icon size="20">
          <SettingsIcon />
        </n-icon>
        <span>{{ $t('nav.settings') }}</span>
      </router-link>
    </nav>
  </aside>
</template>

<script setup lang="ts">
import { NIcon } from 'naive-ui'
import logoUrl from '../assets/logo.png'
import {
  GridOutline as DashboardIcon,
  FolderOutline as FolderIcon,
  BookOutline as BookIcon,
  ColorPaletteOutline as ColorPaletteIcon,
  SettingsOutline as SettingsIcon,
  FlashOutline as FlashIcon,
  MedkitOutline as MedkitIcon
} from '@vicons/ionicons5'
</script>

<style scoped>
.sidebar {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  width: 68px;
  background-color: #1e293b;
  border-right: 1px solid #334155;
  display: flex;
  flex-direction: column;
  transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 1000;
  overflow: hidden;
  white-space: nowrap;
}

.sidebar:hover {
  width: 250px;
}

.logo-container {
  padding: 18px;
  border-bottom: 1px solid #334155;
  display: flex;
  align-items: center;
  overflow: hidden;
}

.logo-icon {
  width: 32px;
  height: 32px;
  margin-right: 12px;
  flex-shrink: 0;
  border-radius: 6px;
}

.logo-text {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  background: linear-gradient(to right, #6366f1, #a855f7);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  opacity: 0;
  transition: opacity 0.3s;
}

.sidebar:hover .logo-text {
  opacity: 1;
}

.nav-menu {
  padding: 16px 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 24px;
  color: #cbd5e1;
  text-decoration: none;
  font-weight: 500;
  transition: all 0.2s;
}

.nav-item span {
  opacity: 0;
  transition: opacity 0.3s;
}

.sidebar:hover .nav-item span {
  opacity: 1;
}

.nav-item:hover {
  background-color: #334155;
  color: #fff;
}

.nav-item.active {
  background-color: #4f46e5;
  color: #fff;
  border-left: 4px solid #818cf8;
}
</style>
