import { create } from "zustand";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { toast } from "sonner";

interface UpdaterState {
  update: Update | null;
  checking: boolean;
  installing: boolean;
  progress: number | null;
  error: string | null;
  lastResult: "found" | "none" | "error" | null;
  checkNow: (opts?: { notify?: boolean }) => Promise<void>;
  installUpdate: () => Promise<void>;
  dismissUpdate: () => void;
}

export const useUpdaterStore = create<UpdaterState>()((set, get) => ({
  update: null,
  checking: false,
  installing: false,
  progress: null,
  error: null,
  lastResult: null,

  checkNow: async (opts) => {
    const { checking, installing } = get();
    if (checking || installing) return;
    set({ checking: true, error: null });
    try {
      const found = await check();
      if (found) {
        set({ update: found, lastResult: "found" });
      } else {
        set({ lastResult: "none" });
        if (opts?.notify) toast.success("You're up to date");
      }
    } catch (e) {
      set({ lastResult: "error", error: String(e) });
      if (opts?.notify) toast.error(`Update check failed: ${String(e)}`);
    } finally {
      set({ checking: false });
    }
  },

  installUpdate: async () => {
    const { update, installing } = get();
    if (!update || installing) return;
    set({ installing: true, progress: 0, error: null });
    try {
      let downloaded = 0;
      let total = 0;
      await update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            total = event.data.contentLength ?? 0;
            set({ progress: 0 });
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            set({
              progress:
                total > 0 ? Math.min(99, Math.round((downloaded / total) * 100)) : null,
            });
            break;
          case "Finished":
            set({ progress: 100 });
            break;
        }
      });
      // On Windows the app exits during the install step; relaunch to
      // finish into the new version.
      await relaunch();
    } catch (e) {
      set({ installing: false, error: String(e) });
    }
  },

  dismissUpdate: () => set({ update: null, installing: false, progress: null }),
}));
