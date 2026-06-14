import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useJournalStore } from "@/stores/journalStore";

vi.mock("@/api/client/tauriCommands", () => ({
  getDailyJournalByDate: vi.fn(async (entryDate: string) => ({
    id: 1,
    entry_date: entryDate,
    title: "Today",
    content: "Thoughts",
    mood: "calm",
  })),
  saveDailyJournal: vi.fn(async (entryDate: string, title: string, content: string, mood: string) => ({
    id: 1,
    entry_date: entryDate,
    title,
    content,
    mood,
  })),
  deleteDailyJournal: vi.fn(async () => undefined),
  getRecentDailyJournals: vi.fn(async () => [
    {
      id: 1,
      entry_date: "2026-06-13",
      title: "Today",
      content: "Thoughts",
      mood: "calm",
    },
  ]),
}));

describe("journalStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("loads a journal by date and refreshes recent items", async () => {
    const store = useJournalStore();

    await store.loadJournal("2026-06-13");

    expect(store.currentJournal?.entry_date).toBe("2026-06-13");
    expect(store.recentJournals).toHaveLength(1);
  });
});
