<script setup lang="ts">
import { ref, watch } from "vue";
import { NButton, NCheckbox, NInput, NProgress, NSelect } from "naive-ui";
import type { PlanItem } from "@/api/client/tauriCommands";

const props = defineProps<{
  item: PlanItem;
  busy?: boolean;
}>();

const emit = defineEmits<{
  save: [payload: {
    itemId: number;
    title: string;
    description: string;
    dimensionKey: string | null;
    sortOrder: number;
    isCompleted: boolean;
  }];
  remove: [itemId: number];
}>();

const draftTitle = ref("");
const draftDescription = ref("");
const draftDimension = ref<string | null>(null);
const draftCompleted = ref(false);

const dimensionOptions = [
  { label: "知识", value: "knowledge" },
  { label: "觉悟", value: "willpower" },
  { label: "表达", value: "expression" },
  { label: "体魄", value: "physique" },
  { label: "羁绊", value: "bond" },
];

watch(
  () => props.item,
  (item) => {
    draftTitle.value = item.title;
    draftDescription.value = item.description;
    draftDimension.value = item.dimension_key;
    draftCompleted.value = item.is_completed;
  },
  { immediate: true, deep: true }
);

function handleSave() {
  emit("save", {
    itemId: props.item.id,
    title: draftTitle.value.trim(),
    description: draftDescription.value.trim(),
    dimensionKey: draftDimension.value,
    sortOrder: props.item.sort_order,
    isCompleted: draftCompleted.value,
  });
}
</script>

<template>
  <div class="goal-card cyber-panel">
    <div class="goal-card-head">
      <div class="goal-card-percent">{{ props.item.progress_percent }}%</div>
      <n-checkbox v-model:checked="draftCompleted">完成</n-checkbox>
    </div>

    <div class="goal-card-fields">
      <n-input v-model:value="draftTitle" placeholder="Goal 标题" />
      <n-input
        v-model:value="draftDescription"
        type="textarea"
        placeholder="把这条 Goal 写具体一点，AI 才更容易判断进度"
        :autosize="{ minRows: 2, maxRows: 4 }"
      />
      <n-select
        v-model:value="draftDimension"
        :options="dimensionOptions"
        clearable
        placeholder="关联维度"
      />
    </div>

    <div class="goal-card-progress">
      <n-progress
        type="line"
        :percentage="props.item.progress_percent"
        :show-indicator="false"
        :height="8"
      />
      <div v-if="props.item.ai_comment" class="goal-card-comment">
        {{ props.item.ai_comment }}
      </div>
    </div>

    <div class="goal-card-actions">
      <n-button size="small" :disabled="busy" @click="handleSave">保存</n-button>
      <n-button size="small" type="error" ghost :disabled="busy" @click="emit('remove', props.item.id)">
        删除
      </n-button>
    </div>
  </div>
</template>

<style scoped>
.goal-card {
  padding: 16px;
  display: grid;
  gap: 12px;
}

.goal-card-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.goal-card-percent {
  font-size: 13px;
  font-weight: 800;
  letter-spacing: 1px;
  color: var(--cyber-cyan);
}

.goal-card-fields,
.goal-card-progress {
  display: grid;
  gap: 10px;
}

.goal-card-comment {
  font-size: 13px;
  line-height: 1.6;
  color: var(--cyber-text-secondary);
}

.goal-card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
