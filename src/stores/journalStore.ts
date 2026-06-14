import { ref } from "vue";
import { defineStore } from "pinia";
import {
  deleteDailyJournal,
  getDailyJournalByDate,
  getRecentDailyJournals,
  saveDailyJournal,
  type DailyJournal,
} from "@/api/client/tauriCommands";
import { getTodayStr } from "@/utils/date";

export const useJournalStore = defineStore("journal", () => {
  const currentDate = ref(getTodayStr());
  const currentJournal = ref<DailyJournal | null>(null);
  const recentJournals = ref<DailyJournal[]>([]);
  const loading = ref(false);
  const saving = ref(false);

  async function loadJournal(entryDate = currentDate.value) {
    currentDate.value = entryDate;
    loading.value = true;
    try {
      currentJournal.value = await getDailyJournalByDate(entryDate);
      recentJournals.value = await getRecentDailyJournals(20);
    } finally {
      loading.value = false;
    }
  }

  async function saveCurrentJournal(title: string, content: string, mood: string) {
    saving.value = true;
    try {
      currentJournal.value = await saveDailyJournal(currentDate.value, title, content, mood);
      recentJournals.value = await getRecentDailyJournals(20);
      return currentJournal.value;
    } finally {
      saving.value = false;
    }
  }

  async function removeCurrentJournal() {
    if (!currentJournal.value) {
      return;
    }

    saving.value = true;
    try {
      await deleteDailyJournal(currentJournal.value.id);
      currentJournal.value = null;
      recentJournals.value = await getRecentDailyJournals(20);
    } finally {
      saving.value = false;
    }
  }

  return {
    currentDate,
    currentJournal,
    recentJournals,
    loading,
    saving,
    loadJournal,
    saveCurrentJournal,
    removeCurrentJournal,
  };
});
