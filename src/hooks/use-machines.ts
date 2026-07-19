import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useMachineStore } from "@/stores/machine-store";
import { useDiscoveryStore } from "@/stores/discovery-store";
import { getDiscoveredMachines, startDiscovery, stopDiscovery } from "@/lib/ipc";
import type { DiscoveryProgress, Machine } from "@/lib/types";

export function useMachines() {
  const { setMachines, addMachine } = useMachineStore();

  const query = useQuery({
    queryKey: ["machines"],
    queryFn: getDiscoveredMachines,
  });

  useEffect(() => {
    if (query.data) {
      setMachines(query.data);
    }
  }, [query.data, setMachines]);

  useEffect(() => {
    const unlisten1 = listen<Machine>("machine-discovered", (event) => {
      addMachine(event.payload);
    });
    const unlisten2 = listen<Machine>("machine-offline", (event) => {
      addMachine(event.payload);
    });
    const unlisten3 = listen<Machine>("machine-online", (event) => {
      addMachine(event.payload);
    });
    const unlisten4 = listen<DiscoveryProgress>("discovery-progress", (event) => {
      useDiscoveryStore.getState().setProgress(event.payload);
    });

    return () => {
      unlisten1.then((fn) => fn());
      unlisten2.then((fn) => fn());
      unlisten3.then((fn) => fn());
      unlisten4.then((fn) => fn());
    };
  }, [addMachine]);

  return query;
}

export function useDiscovery() {
  const queryClient = useQueryClient();

  const startMutation = useMutation({
    mutationFn: startDiscovery,
    onSuccess: () => {
      useDiscoveryStore.getState().setProgress({ scanned: 0, total: 254 });
      queryClient.invalidateQueries({ queryKey: ["machines"] });
    },
  });

  const stopMutation = useMutation({
    mutationFn: stopDiscovery,
    onSuccess: () => {
      useDiscoveryStore.getState().setProgress(null);
      queryClient.invalidateQueries({ queryKey: ["machines"] });
    },
  });

  return { startMutation, stopMutation };
}
