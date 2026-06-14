import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { usePersonalMemoryStore } from "@/stores/personalMemoryStore";

vi.mock("@/api/client/tauriCommands", () => ({
  getPersonalProfile: vi.fn(async () => ({
    birthday: "1998-01-02",
    personality: "calm",
    experiences: "studied design",
    personal_notes: "likes routine",
    updated_at: "2026-06-13 18:00:00",
  })),
  savePersonalProfile: vi.fn(async (profile) => ({
    ...profile,
    updated_at: "2026-06-13 18:00:00",
  })),
  getPersonalMemoryOverview: vi.fn(async () => ({
    total_items: 1,
    active_items: 1,
    pending_items: 0,
    rejected_items: 0,
    top_items: [
      {
        id: 1,
        memory_type: "habit",
        title: "Morning study",
        summary: "Usually studies in the morning",
        detail: "",
        tags: ["study"],
        importance: 75,
        confidence: 0.8,
        first_seen_date: "2026-06-10",
        last_seen_date: "2026-06-13",
        status: "active",
        supersedes_id: null,
        created_by: "ai",
        source_count: 2,
        evidence_ids: ["rec:1", "jr:2"],
      },
    ],
  })),
  searchPersonalMemory: vi.fn(async () => [
    {
      id: 1,
      memory_type: "habit",
      title: "Morning study",
      summary: "Usually studies in the morning",
      detail: "",
      tags: ["study"],
      importance: 75,
      confidence: 0.8,
      first_seen_date: "2026-06-10",
      last_seen_date: "2026-06-13",
      status: "active",
      supersedes_id: null,
      created_by: "ai",
      source_count: 2,
      evidence_ids: ["rec:1", "jr:2"],
    },
  ]),
  applyPersonalMemoryPatch: vi.fn(async () => ({
    patch_run_id: 3,
    validation_status: "applied",
    apply_status: "applied",
    applied_operations: 1,
    rejected_operations: 0,
    message: "ok",
  })),
}));

describe("personalMemoryStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("loads profile and overview together", async () => {
    const store = usePersonalMemoryStore();

    await store.loadFoundation();

    expect(store.profile.birthday).toBe("1998-01-02");
    expect(store.overview?.active_items).toBe(1);
    expect(store.overview?.top_items[0].title).toBe("Morning study");
  });

  it("applies a patch and stores the result state", async () => {
    const store = usePersonalMemoryStore();

    await store.applyPatch('{"schema_version":"1.0","profile_updates":null,"memory_operations":[]}');

    expect(store.lastPatchResult?.apply_status).toBe("applied");
  });
});
