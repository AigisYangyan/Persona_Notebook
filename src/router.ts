import { createRouter, createWebHashHistory } from "vue-router";
import Dashboard from "@/pages/Dashboard.vue";
import Today from "@/pages/Today.vue";
import Calendar from "@/pages/Calendar.vue";
import Ledger from "@/pages/Ledger.vue";
import Settings from "@/pages/Settings.vue";

const routes = [
  { path: "/", redirect: "/dashboard" },
  { path: "/dashboard", name: "Dashboard", component: Dashboard },
  { path: "/today", name: "Today", component: Today },
  { path: "/calendar", name: "Calendar", component: Calendar },
  { path: "/ledger", name: "Ledger", component: Ledger },
  { path: "/settings", name: "Settings", component: Settings },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
