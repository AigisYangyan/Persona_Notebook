import { ref } from "vue";
import { defineStore } from "pinia";
import {
  applyPersonalMemoryPatch,
  buildPersonalContextPack,
  exportRagMemory,
  getPersonalMemoryOverview,
  getPersonalProfile,
  rebuildRagMemoryFiles,
  savePersonalProfile,
  searchPersonalMemory,
  type PersonalContextPack,
  type PersonalMemoryOverview,
  type PersonalMemoryPatchApplyResult,
  type PersonalMemoryViewItem,
  type PersonalProfile,
} from "@/api/client/tauriCommands";

const EMPTY_PROFILE: PersonalProfile = {
  birthday: "",
  personality: "",
  experiences: "",
  personal_notes: "",
  updated_at: null,
};

export const usePersonalMemoryStore = defineStore("personalMemory", () => {
  const profile = ref<PersonalProfile>({ ...EMPTY_PROFILE });
  const overview = ref<PersonalMemoryOverview | null>(null);
  const searchResults = ref<PersonalMemoryViewItem[]>([]);
  const contextPack = ref<PersonalContextPack | null>(null);
  const lastPatchResult = ref<PersonalMemoryPatchApplyResult | null>(null);
  const lastMirrorManifest = ref("");
  const lastExportSnapshot = ref("");
  const loading = ref(false);
  const saving = ref(false);
  const searching = ref(false);
  const patching = ref(false);

  async function loadFoundation() {
    loading.value = true;
    try {
      const [nextProfile, nextOverview] = await Promise.all([
        getPersonalProfile(),
        getPersonalMemoryOverview(),
      ]);
      profile.value = nextProfile;
      overview.value = nextOverview;
    } finally {
      loading.value = false;
    }
  }

  async function saveProfile(nextProfile?: PersonalProfile) {
    saving.value = true;
    try {
      profile.value = await savePersonalProfile(nextProfile ?? profile.value);
      overview.value = await getPersonalMemoryOverview();
      return profile.value;
    } finally {
      saving.value = false;
    }
  }

  async function searchMemory(query = "", tags: string[] = [], limit = 20) {
    searching.value = true;
    try {
      searchResults.value = await searchPersonalMemory(query, tags, limit);
      return searchResults.value;
    } finally {
      searching.value = false;
    }
  }

  async function generateContextPack(date?: string, mode = "general") {
    loading.value = true;
    try {
      contextPack.value = await buildPersonalContextPack(date ?? null, mode);
      return contextPack.value;
    } finally {
      loading.value = false;
    }
  }

  async function applyPatch(patchJson: string, sourceContextId = "manual-settings") {
    patching.value = true;
    try {
      lastPatchResult.value = await applyPersonalMemoryPatch(patchJson, sourceContextId);
      await loadFoundation();
      return lastPatchResult.value;
    } finally {
      patching.value = false;
    }
  }

  async function rebuildMirrorFiles() {
    saving.value = true;
    try {
      lastMirrorManifest.value = await rebuildRagMemoryFiles();
      return lastMirrorManifest.value;
    } finally {
      saving.value = false;
    }
  }

  async function exportSnapshot() {
    saving.value = true;
    try {
      lastExportSnapshot.value = await exportRagMemory();
      return lastExportSnapshot.value;
    } finally {
      saving.value = false;
    }
  }

  function updateProfileField<K extends keyof PersonalProfile>(key: K, value: PersonalProfile[K]) {
    profile.value = {
      ...profile.value,
      [key]: value,
    };
  }

  return {
    profile,
    overview,
    searchResults,
    contextPack,
    lastPatchResult,
    lastMirrorManifest,
    lastExportSnapshot,
    loading,
    saving,
    searching,
    patching,
    loadFoundation,
    saveProfile,
    searchMemory,
    generateContextPack,
    applyPatch,
    rebuildMirrorFiles,
    exportSnapshot,
    updateProfileField,
  };
});
