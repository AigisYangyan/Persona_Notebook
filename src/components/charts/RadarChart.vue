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
  tooltip: {},
  radar: {
    indicator: props.data.map((d) => ({ name: d.name, max: 100 })),
    radius: "65%",
  },
  series: [
    {
      type: "radar",
      data: [
        {
          value: props.data.map((d) => d.value),
          name: "当前五维",
          areaStyle: { opacity: 0.3 },
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
