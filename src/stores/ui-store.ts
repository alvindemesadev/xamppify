import { create } from "zustand";
import { persist } from "zustand/middleware";

interface UiStore {
  sidebarOpen: boolean;
  theme: "dark" | "light";
  selectedMachineId: string | null;
  onboardingComplete: boolean;
  compactMode: boolean;
  toggleSidebar: () => void;
  setTheme: (theme: "dark" | "light") => void;
  setSelectedMachineId: (id: string | null) => void;
  completeOnboarding: () => void;
  resetOnboarding: () => void;
  toggleCompactMode: () => void;
}

export const useUiStore = create<UiStore>()(
  persist(
    (set) => ({
      sidebarOpen: true,
      theme: "dark",
      selectedMachineId: null,
      onboardingComplete: false,
      compactMode: false,
      toggleSidebar: () =>
        set((state) => ({ sidebarOpen: !state.sidebarOpen })),
      setTheme: (theme) => set({ theme }),
      setSelectedMachineId: (selectedMachineId) => set({ selectedMachineId }),
      completeOnboarding: () => set({ onboardingComplete: true }),
      resetOnboarding: () => set({ onboardingComplete: false }),
      toggleCompactMode: () =>
        set((state) => ({ compactMode: !state.compactMode })),
    }),
    { name: "ui-preferences" }
  )
);
