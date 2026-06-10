import { ref } from "vue";
import { defineStore } from "pinia";
import { getDimensionTotals } from "@/api/client/tauriCommands";

export interface DimensionScore {
  key: string;
  name: string;
  value: number;
  dailyCap: number;
}

export const useStatStore = defineStore("stat", () => {
  const dimensions = ref<DimensionScore[]>([
    { key: "knowledge", name: "学识", value: 0, dailyCap: 10 },
    { key: "willpower", name: "觉悟", value: 0, dailyCap: 8 },
    { key: "expression", name: "表达", value: 0, dailyCap: 8 },
    { key: "physique", name: "体魄", value: 0, dailyCap: 8 },
    { key: "bond", name: "羁绊", value: 0, dailyCap: 6 },
  ]);

  const totalLevel = ref(1);
  const totalExp = ref(0);
  const nextLevelExp = ref(50);

  async function refreshStats() {
    const totals = await getDimensionTotals();
    const totalMap = new Map(totals.map((t) => [t.key, t.total]));
    dimensions.value = dimensions.value.map((d) => ({
      ...d,
      value: totalMap.get(d.key) ?? 0,
    }));
    const total = dimensions.value.reduce((sum, d) => sum + d.value, 0);
    totalExp.value = total;
    totalLevel.value = Math.floor(total / 50) + 1;
    nextLevelExp.value = totalLevel.value * 50;
  }

  return { dimensions, totalLevel, totalExp, nextLevelExp, refreshStats };
});
