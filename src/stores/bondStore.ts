import { computed, ref } from "vue";
import { defineStore } from "pinia";
import {
  deleteBondEntry,
  deleteBondPerson,
  getBondEntries,
  getBondPeople,
  saveBondEntry,
  saveBondPerson,
  type BondEntry,
  type BondPerson,
} from "@/api/client/tauriCommands";
import { getTodayStr } from "@/utils/date";

export const useBondStore = defineStore("bond", () => {
  const people = ref<BondPerson[]>([]);
  const entries = ref<BondEntry[]>([]);
  const selectedPersonId = ref<number | null>(null);
  const selectedEntryDate = ref(getTodayStr());
  const loading = ref(false);
  const saving = ref(false);

  const selectedPerson = computed(
    () => people.value.find((person) => person.id === selectedPersonId.value) ?? null
  );
  const selectedEntry = computed(
    () => entries.value.find((entry) => entry.entry_date === selectedEntryDate.value) ?? null
  );

  async function loadPeople() {
    loading.value = true;
    try {
      people.value = await getBondPeople();
      if (selectedPersonId.value === null || !people.value.some((person) => person.id === selectedPersonId.value)) {
        selectedPersonId.value = people.value[0]?.id ?? null;
      }
      if (selectedPersonId.value !== null) {
        await loadEntries(selectedPersonId.value);
      } else {
        entries.value = [];
      }
    } finally {
      loading.value = false;
    }
  }

  async function loadEntries(personId: number) {
    selectedPersonId.value = personId;
    entries.value = await getBondEntries(personId);
    if (!entries.value.some((entry) => entry.entry_date === selectedEntryDate.value)) {
      selectedEntryDate.value = entries.value[0]?.entry_date ?? getTodayStr();
    }
  }

  async function savePerson(payload: {
    personId?: number | null;
    name: string;
    relationLabel?: string | null;
    score: number;
    note?: string | null;
  }) {
    saving.value = true;
    try {
      const person = await saveBondPerson(payload);
      await loadPeople();
      selectedPersonId.value = person.id;
      await loadEntries(person.id);
      return person;
    } finally {
      saving.value = false;
    }
  }

  async function removePerson(personId: number) {
    saving.value = true;
    try {
      await deleteBondPerson(personId);
      if (selectedPersonId.value === personId) {
        selectedPersonId.value = null;
        entries.value = [];
      }
      await loadPeople();
    } finally {
      saving.value = false;
    }
  }

  async function saveEntry(title: string, content: string) {
    if (selectedPersonId.value === null) {
      throw new Error("no bond person selected");
    }

    saving.value = true;
    try {
      const entry = await saveBondEntry(selectedPersonId.value, selectedEntryDate.value, title, content);
      await loadEntries(selectedPersonId.value);
      await loadPeople();
      selectedEntryDate.value = entry.entry_date;
      return entry;
    } finally {
      saving.value = false;
    }
  }

  async function removeEntry(entryId: number) {
    if (selectedPersonId.value === null) {
      return;
    }

    saving.value = true;
    try {
      await deleteBondEntry(entryId);
      await loadEntries(selectedPersonId.value);
      await loadPeople();
    } finally {
      saving.value = false;
    }
  }

  function setSelectedEntryDate(entryDate: string) {
    selectedEntryDate.value = entryDate;
  }

  return {
    people,
    entries,
    selectedPersonId,
    selectedEntryDate,
    selectedPerson,
    selectedEntry,
    loading,
    saving,
    loadPeople,
    loadEntries,
    savePerson,
    removePerson,
    saveEntry,
    removeEntry,
    setSelectedEntryDate,
  };
});
