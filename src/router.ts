import { createRouter, createWebHashHistory } from "vue-router";
import Dashboard from "@/pages/Dashboard.vue";
import Today from "@/pages/Today.vue";
import Calendar from "@/pages/Calendar.vue";
import BondRelations from "@/pages/BondRelations.vue";
import DailyJournal from "@/pages/DailyJournal.vue";
import WeekPlan from "@/pages/WeekPlan.vue";
import MonthPlan from "@/pages/MonthPlan.vue";
import Tarot from "@/pages/Tarot.vue";
import Reports from "@/pages/Reports.vue";
import Settings from "@/pages/Settings.vue";

const routes = [
  { path: "/", redirect: "/dashboard" },
  { path: "/dashboard", name: "Dashboard", component: Dashboard },
  { path: "/today", name: "Today", component: Today },
  { path: "/calendar", name: "Calendar", component: Calendar },
  { path: "/bond-relations", name: "BondRelations", component: BondRelations },
  { path: "/daily-journal", name: "DailyJournal", component: DailyJournal },
  { path: "/week-plan", name: "WeekPlan", component: WeekPlan },
  { path: "/month-plan", name: "MonthPlan", component: MonthPlan },
  { path: "/tarot", name: "Tarot", component: Tarot },
  { path: "/reports", name: "Reports", component: Reports },
  { path: "/settings", name: "Settings", component: Settings },
  { path: "/:pathMatch(.*)*", redirect: "/calendar" },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
