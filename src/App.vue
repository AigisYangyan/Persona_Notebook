<script setup lang="ts">
import { computed, onMounted, onErrorCaptured, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NIcon, NAlert, NMessageProvider } from "naive-ui";
import {
  StatsChartOutline,
  TodayOutline,
  CalendarOutline,
  BookOutline,
  SettingsOutline,
} from "@vicons/ionicons5";
import { useSettingStore } from "@/stores/settingStore";
import bgUrl from "@/assets/background.png";

const route = useRoute();
const router = useRouter();
const settingStore = useSettingStore();
const pageError = ref("");

onErrorCaptured((err) => {
  pageError.value = String(err);
  console.error("页面渲染错误:", err);
  return false;
});

onMounted(() => {
  settingStore.loadSettings();
});

const activeKey = computed(() => route.path);

function handleMenuUpdate(key: string) {
  pageError.value = "";
  router.push(key);
}

const navItems = [
  { num: '01', label: 'Dashboard', sub: '仪表盘', path: '/dashboard', icon: StatsChartOutline },
  { num: '02', label: '今日记录', sub: 'TODAY RECORD', path: '/today', icon: TodayOutline },
  { num: '03', label: '日历', sub: 'CALENDAR', path: '/calendar', icon: CalendarOutline },
  { num: '04', label: '成长账本', sub: 'GROWTH LEDGER', path: '/ledger', icon: BookOutline },
  { num: '05', label: '设置', sub: 'SETTINGS', path: '/settings', icon: SettingsOutline },
];
</script>

<template>
  <div class="bg-layer" :style="{ backgroundImage: `url(${bgUrl})` }"></div>
  <div class="app-container">
    <!-- Custom Sidebar -->
    <aside class="cyber-sidebar">
      <div class="sidebar-header">
        <div class="app-icon">🌙</div>
        <div class="app-name">PGRN</div>
      </div>

      <nav class="sidebar-nav">
        <div
          v-for="item in navItems"
          :key="item.path"
          class="nav-item"
          :class="{ active: activeKey === item.path }"
          @click="handleMenuUpdate(item.path)"
        >
          <div class="nav-num">{{ item.num }}</div>
          <div class="nav-icon">
            <n-icon size="22">
              <component :is="item.icon" />
            </n-icon>
          </div>
          <div class="nav-text">
            <div class="nav-label">{{ item.label }}</div>
            <div class="nav-sub">{{ item.sub }}</div>
          </div>
          <div v-if="activeKey === item.path" class="nav-indicator"></div>
        </div>
      </nav>

      <div class="sidebar-footer">
        <div class="footer-line"></div>
        <div class="footer-text">v0.2.0</div>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="cyber-main">
      <n-message-provider>
        <n-alert
          v-if="pageError"
          type="error"
          title="页面渲染错误"
          style="margin-bottom: 16px"
        >
          {{ pageError }}
        </n-alert>
        <router-view />
      </n-message-provider>
    </main>
  </div>
</template>

<style>
/* Global font override for the app */
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto,
    'PingFang SC', 'Microsoft YaHei', sans-serif;
}
</style>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  position: relative;
  z-index: 1;
}

/* ===== Sidebar ===== */
.cyber-sidebar {
  width: 220px;
  min-width: 220px;
  background: linear-gradient(180deg, rgba(8, 14, 30, 0.98), rgba(5, 10, 22, 0.98));
  border-right: 1px solid rgba(0, 180, 255, 0.15);
  display: flex;
  flex-direction: column;
  position: relative;
  z-index: 10;
}

.cyber-sidebar::before {
  content: '';
  position: absolute;
  top: 0;
  right: -1px;
  width: 1px;
  height: 100%;
  background: linear-gradient(180deg, transparent, var(--cyber-cyan), transparent);
  opacity: 0.3;
}

.sidebar-header {
  padding: 24px 20px;
  display: flex;
  align-items: center;
  gap: 10px;
  border-bottom: 1px solid rgba(0, 180, 255, 0.1);
}

.app-icon {
  font-size: 24px;
}

.app-name {
  font-size: 18px;
  font-weight: 800;
  letter-spacing: 3px;
  color: var(--cyber-text-primary);
  text-shadow: 0 0 10px rgba(0, 212, 255, 0.3);
}

/* ===== Nav Items ===== */
.sidebar-nav {
  flex: 1;
  padding: 16px 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  margin: 0 8px;
  border-radius: 4px;
  cursor: pointer;
  position: relative;
  transition: all 0.25s ease;
  border: 1px solid transparent;
}

.nav-item:hover {
  background: rgba(0, 180, 255, 0.06);
  border-color: rgba(0, 180, 255, 0.15);
}

.nav-item.active {
  background: linear-gradient(90deg, rgba(0, 180, 255, 0.12), rgba(0, 180, 255, 0.03));
  border-color: rgba(0, 212, 255, 0.3);
  box-shadow: 0 0 12px rgba(0, 180, 255, 0.08);
}

.nav-num {
  font-size: 20px;
  font-weight: 900;
  font-style: italic;
  color: var(--cyber-blue-dim);
  width: 32px;
  text-align: center;
  line-height: 1;
  transition: color 0.25s;
}

.nav-item:hover .nav-num,
.nav-item.active .nav-num {
  color: var(--cyber-cyan);
  text-shadow: 0 0 8px rgba(0, 212, 255, 0.4);
}

.nav-icon {
  color: var(--cyber-text-muted);
  transition: color 0.25s;
  display: flex;
  align-items: center;
}

.nav-item:hover .nav-icon,
.nav-item.active .nav-icon {
  color: var(--cyber-cyan);
}

.nav-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--cyber-text-secondary);
  transition: color 0.25s;
}

.nav-sub {
  font-size: 10px;
  font-weight: 500;
  letter-spacing: 1px;
  color: var(--cyber-text-dim);
  transition: color 0.25s;
}

.nav-item:hover .nav-label,
.nav-item.active .nav-label {
  color: var(--cyber-text-primary);
}

.nav-item:hover .nav-sub,
.nav-item.active .nav-sub {
  color: var(--cyber-text-muted);
}

.nav-indicator {
  position: absolute;
  right: -8px;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-top: 8px solid transparent;
  border-bottom: 8px solid transparent;
  border-left: 10px solid var(--cyber-cyan);
  filter: drop-shadow(0 0 4px rgba(0, 212, 255, 0.5));
}

/* ===== Sidebar Footer ===== */
.sidebar-footer {
  padding: 16px 20px;
  border-top: 1px solid rgba(0, 180, 255, 0.1);
}

.footer-line {
  height: 1px;
  background: linear-gradient(90deg, var(--cyber-cyan), transparent);
  margin-bottom: 8px;
  opacity: 0.4;
}

.footer-text {
  font-size: 11px;
  color: var(--cyber-text-dim);
  letter-spacing: 2px;
}

/* ===== Main Content ===== */
.cyber-main {
  flex: 1;
  overflow: hidden;
  position: relative;
  z-index: 1;
}

.cyber-main :deep(.n-message-provider) {
  height: 100%;
  overflow-y: auto;
}

/* Override n-alert inside main */
.cyber-main :deep(.n-alert) {
  margin: 16px 32px 0;
}

/* Background Layer */
.bg-layer {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 0;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
  pointer-events: none;
}
</style>
