<script setup lang="ts">
import { ref, watch } from "vue";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { RadarChart as ERadarChart } from "echarts/charts";
import { TooltipComponent, LegendComponent } from "echarts/components";
import VChart from "vue-echarts";

use([CanvasRenderer, ERadarChart, TooltipComponent, LegendComponent]);

interface DimensionData {
  key: string;
  name: string;
  value: number;
}

const props = defineProps<{
  data: DimensionData[];
}>();

const option = ref({
  tooltip: {
    backgroundColor: 'rgba(10, 18, 40, 0.95)',
    borderColor: 'rgba(0, 212, 255, 0.3)',
    textStyle: {
      color: '#ffffff',
    },
  },
  radar: {
    indicator: props.data.map((d) => ({ name: d.name, max: 100 })),
    radius: "65%",
    axisName: {
      color: '#a0c4e8',
      fontSize: 13,
      fontWeight: 'bold',
    },
    splitArea: {
      areaStyle: {
        color: ['rgba(0, 60, 120, 0.05)', 'rgba(0, 60, 120, 0.1)', 'rgba(0, 60, 120, 0.15)', 'rgba(0, 60, 120, 0.2)'],
      },
    },
    splitLine: {
      lineStyle: {
        color: 'rgba(0, 180, 255, 0.15)',
      },
    },
    axisLine: {
      lineStyle: {
        color: 'rgba(0, 180, 255, 0.2)',
      },
    },
  },
  series: [
    {
      type: "radar",
      data: [
        {
          value: props.data.map((d) => d.value),
          name: "当前五维",
          areaStyle: {
            color: 'rgba(0, 212, 255, 0.2)',
          },
          lineStyle: {
            color: '#00d4ff',
            width: 2,
          },
          itemStyle: {
            color: '#00d4ff',
            borderColor: '#ffffff',
            borderWidth: 1,
          },
        },
      ],
    },
  ],
});

watch(
  () => props.data,
  (newData) => {
    option.value.radar.indicator = newData.map((d) => ({ name: d.name, max: 100 }));
    option.value.series[0].data[0].value = newData.map((d) => d.value);
  },
  { deep: true }
);
</script>

<template>
  <v-chart class="chart" :option="option" autoresize />
</template>

<style scoped>
.chart {
  height: 400px;
  width: 100%;
}
</style>
