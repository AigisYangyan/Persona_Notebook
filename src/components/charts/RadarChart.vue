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
    borderColor: 'rgba(0, 212, 255, 0.4)',
    textStyle: {
      color: '#ffffff',
      fontSize: 14,
    },
  },
  radar: {
    indicator: props.data.map((d) => ({ name: d.name, max: 100 })),
    radius: "72%",
    center: ["50%", "52%"],
    shape: "polygon",
    splitNumber: 4,
    axisName: {
      color: '#d7f4ff',
      fontSize: 16,
      fontWeight: 'bold',
      formatter: (value: string) => value,
    },
    splitArea: {
      areaStyle: {
        color: [
          'rgba(0, 80, 160, 0.12)',
          'rgba(0, 80, 160, 0.18)',
          'rgba(0, 80, 160, 0.24)',
          'rgba(0, 80, 160, 0.30)',
        ],
      },
    },
    splitLine: {
      lineStyle: {
        color: 'rgba(0, 180, 255, 0.25)',
        width: 1,
      },
    },
    axisLine: {
      lineStyle: {
        color: 'rgba(0, 180, 255, 0.35)',
        width: 1,
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
          symbol: "circle",
          symbolSize: 10,
          lineStyle: {
            color: '#00d4ff',
            width: 3,
            shadowColor: 'rgba(0, 212, 255, 0.6)',
            shadowBlur: 12,
          },
          itemStyle: {
            color: '#ffffff',
            borderColor: '#00d4ff',
            borderWidth: 3,
            shadowColor: 'rgba(0, 212, 255, 0.8)',
            shadowBlur: 10,
          },
          areaStyle: {
            color: {
              type: 'radial',
              x: 0.5,
              y: 0.5,
              r: 0.5,
              colorStops: [
                { offset: 0, color: 'rgba(0, 212, 255, 0.55)' },
                { offset: 0.7, color: 'rgba(0, 120, 220, 0.25)' },
                { offset: 1, color: 'rgba(0, 60, 120, 0.08)' },
              ],
            },
          },
          emphasis: {
            lineStyle: { width: 4 },
            itemStyle: { scale: 1.4 },
            areaStyle: {
              color: 'rgba(0, 212, 255, 0.45)',
            },
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
  height: 540px;
  width: 100%;
}
</style>
