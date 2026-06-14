import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useBondStore } from "@/stores/bondStore";

vi.mock("@/api/client/tauriCommands", () => ({
  getBondPeople: vi.fn(async () => [
    {
      id: 1,
      name: "Alice",
      relation_label: "friend",
      score: 8,
      note: "",
      latest_entry_date: "2026-06-13",
      entry_count: 1,
    },
  ]),
  saveBondPerson: vi.fn(async () => ({
    id: 1,
    name: "Alice",
    relation_label: "friend",
    score: 8,
    note: "",
    latest_entry_date: "2026-06-13",
    entry_count: 1,
  })),
  deleteBondPerson: vi.fn(async () => undefined),
  getBondEntries: vi.fn(async () => [
    {
      id: 10,
      person_id: 1,
      entry_date: "2026-06-13",
      title: "Note",
      content: "Content",
    },
  ]),
  saveBondEntry: vi.fn(async () => ({
    id: 10,
    person_id: 1,
    entry_date: "2026-06-13",
    title: "Note",
    content: "Content",
  })),
  deleteBondEntry: vi.fn(async () => undefined),
}));

describe("bondStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("loads people and selects the first person with entries", async () => {
    const store = useBondStore();

    await store.loadPeople();

    expect(store.people).toHaveLength(1);
    expect(store.selectedPersonId).toBe(1);
    expect(store.entries).toHaveLength(1);
  });
});
