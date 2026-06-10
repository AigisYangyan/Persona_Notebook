<script setup lang="ts">
import { computed, onMounted, onErrorCaptured, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { NLayout, NLayoutSider, NLayoutContent, NMenu, NIcon, NAlert, NMessageProvider } from "naive-ui";
import type { MenuOption } from "naive-ui";
import {
  StatsChartOutline,
  TodayOutline,
  CalendarOutline,
  BookOutline,
  SettingsOutline,
} from "@vicons/ionicons5";
import { h } from "vue";
import { useSettingStore } from "@/stores/settingStore";

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

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) });
}

const menuOptions: MenuOption[] = [
  { label: "Dashboard", key: "/dashboard", icon: renderIcon(StatsChartOutline) },
  { label: "今日记录", key: "/today", icon: renderIcon(TodayOutline) },
  { label: "日历", key: "/calendar", icon: renderIcon(CalendarOutline) },
  { label: "成长账本", key: "/ledger", icon: renderIcon(BookOutline) },
  { label: "设置", key: "/settings", icon: renderIcon(SettingsOutline) },
];

const activeKey = computed(() => route.path);

function handleMenuUpdate(key: string) {
  pageError.value = "";
  router.push(key);
}
</script>

<template>
  <n-layout has-sider style="height: 100vh">
    <n-layout-sider
      bordered
      collapse-mode="width"
      :collapsed-width="64"
      :width="180"
      show-trigger
    >
      <n-menu
        :value="activeKey"
        :collapsed-width="64"
        :collapsed-icon-size="22"
        :options="menuOptions"
        @update:value="handleMenuUpdate"
      />
    </n-layout-sider>
    <n-layout-content content-style="padding: 24px">
      <n-message-provider>
        <n-alert v-if="pageError" type="error" title="页面渲染错误" style="margin-bottom: 16px">
          {{ pageError }}
        </n-alert>
        <router-view />
      </n-message-provider>
    </n-layout-content>
  </n-layout>
</template>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: "Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto,
    sans-serif;
}
</style>
