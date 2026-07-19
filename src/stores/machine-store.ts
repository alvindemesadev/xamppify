import { create } from "zustand";
import type { Machine } from "@/lib/types";

interface MachineStore {
  machines: Machine[];
  setMachines: (machines: Machine[]) => void;
  addMachine: (machine: Machine) => void;
  removeMachine: (id: string) => void;
  updateMachine: (id: string, updates: Partial<Machine>) => void;
}

export const useMachineStore = create<MachineStore>((set) => ({
  machines: [],
  setMachines: (machines) => set({ machines }),
  addMachine: (machine) =>
    set((state) => ({
      machines: state.machines.some((m) => m.id === machine.id)
        ? state.machines.map((m) => (m.id === machine.id ? machine : m))
        : [...state.machines, machine],
    })),
  removeMachine: (id) =>
    set((state) => ({
      machines: state.machines.filter((m) => m.id !== id),
    })),
  updateMachine: (id, updates) =>
    set((state) => ({
      machines: state.machines.map((m) =>
        m.id === id ? { ...m, ...updates } : m
      ),
    })),
}));
