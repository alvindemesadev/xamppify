import { create } from "zustand";

interface DiscoveryStore {
  isRunning: boolean;
  progress: { scanned: number; total: number } | null;
  setRunning: (running: boolean) => void;
  setProgress: (progress: { scanned: number; total: number } | null) => void;
}

export const useDiscoveryStore = create<DiscoveryStore>((set) => ({
  isRunning: false,
  progress: null,
  setRunning: (running) => set({ isRunning: running }),
  setProgress: (progress) => set({ progress }),
}));
