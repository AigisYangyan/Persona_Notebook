<script setup lang="ts">
import { computed } from "vue";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { PieChart } from "echarts/charts";
import { GraphicComponent, TooltipComponent } from "echarts/components";
import VChart from "vue-echarts";
import { formatDuration } from "@/features/records/taskMetrics";

use([CanvasRenderer, PieChart, TooltipComponent, GraphicComponent]);

interface ChartItem {
  id: number;
  title: string;
  value: number;
}

const props = defineProps<{
  items: ChartItem[];
}>();

const totalSeconds = computed(() =>
  props.items.reduce((sum, item) => sum + item.value, 0)
);

const option = computed(() => ({
  backgroundColor: "transparent",
  tooltip: {
    trigger: "item",
    backgroundColor: "rgba(10, 18, 40, 0.95)",
    borderColor: "rgba(0, 212, 255, 0.3)",
    textStyle: { color: "#ffffff" },
    formatter: (params: { name: string; value: number; percent: number }) =>
      `${params.name}<br/>${formatDuration(params.value)} (${params.percent}%)`,
  },
  graphic: [
    {
      type: "text",
      left: "center",
      top: "42%",
      style: {
        text: formatDuration(totalSeconds.value),
        fill: "#ffffff",
        fontSize: 20,
        fontWeight: 700,
      },
    },
    {
      type: "text",
      left: "center",
      top: "54%",
      style: {
        text: "TODAY",
        fill: "#5a7a9a",
        fontSize: 11,
        fontWeight: 600,
      },
    },
  ],
  series: [
    {
      type: "pie",
      radius: ["58%", "78%"],
      center: ["50%", "50%"],
      avoidLabelOverlap: true,
      itemStyle: {
        borderColor: "#08101f",
        borderWidth: 2,
      },
      label: {
        color: "#a0c4e8",
        formatter: "{b|{b}}\n{c|{d}%}",
        rich: {
          b: { fontSize: 11, fontWeight: 600, color: "#a0c4e8" },
          c: { fontSize: 10, color: "#5a7a9a" },
        },
      },
      labelLine: {
        lineStyle: {
          color: "rgba(0, 212, 255, 0.35)",
        },
      },
      data: props.items.map((item, index) => ({
        value: item.value,
        name: item.title,
        itemStyle: {
          color: PIE_COLORS[index % PIE_COLORS.length],
          shadowBlur: 12,
          shadowColor: "rgba(0, 212, 255, 0.2)",
        },
      })),
    },
  ],
}));

const PIE_COLORS = [
  "#00d4ff",
  "#00ffaa",
  "#ffcc00",
  "#ff3366",
  "#2f7cff",
  "#5ce1e6",
  "#86efac",
  "#f97316",
];
</script>

<template>
  <div v-if="items.length === 0" class="empty-chart">
    <div class="empty-value">00:00</div>
    <div class="empty-label">暂无计时数据</div>
  </div>
  <v-chart v-else class="chart" :option="option" autoresize />
</template>

<style scoped>
.chart {
  width: 100%;
  height: 320px;
}

.empty-chart {
  height: 320px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--cyber-text-muted);
}

.empty-value {
  font-size: 34px;
  font-weight: 800;
  color: var(--cyber-text-primary);
}

.empty-label {
  margin-top: 8px;
  font-size: 12px;
  letter-spacing: 1px;
}
</style>
