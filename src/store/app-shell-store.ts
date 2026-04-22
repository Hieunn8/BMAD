import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

type AppShellStatus = "idle" | "loading" | "ready" | "error";

type AppShellState = {
  error: string | null;
  ffmpegPath: string | null;
  status: AppShellStatus;
  loadFfmpegPath: () => Promise<void>;
};

export const useAppShellStore = create<AppShellState>((set) => ({
  error: null,
  ffmpegPath: null,
  status: "idle",
  loadFfmpegPath: async () => {
    set({ status: "loading", error: null });

    try {
      const ffmpegPath = await invoke<string>("get_ffmpeg_path");
      set({ ffmpegPath, status: "ready" });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      set({ error: message, status: "error" });
    }
  },
}));
