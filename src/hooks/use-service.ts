import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { startService, stopService, restartService, getServiceStatus } from "@/lib/ipc";

export function useServiceStatus(machineId: string) {
  return useQuery({
    queryKey: ["services", machineId],
    queryFn: () => getServiceStatus(machineId),
  });
}

export function useServiceControl(machineId: string) {
  const queryClient = useQueryClient();

  const invalidate = () => {
    queryClient.invalidateQueries({ queryKey: ["services", machineId] });
  };

  const startMutation = useMutation({
    mutationFn: (service: string) => startService(machineId, service),
    onSuccess: (_, service) => {
      toast.success(`${service} started`);
      invalidate();
    },
    onError: (err) => toast.error(String(err)),
  });

  const stopMutation = useMutation({
    mutationFn: (service: string) => stopService(machineId, service),
    onSuccess: (_, service) => {
      toast.success(`${service} stopped`);
      invalidate();
    },
    onError: (err) => toast.error(String(err)),
  });

  const restartMutation = useMutation({
    mutationFn: (service: string) => restartService(machineId, service),
    onSuccess: (_, service) => {
      toast.success(`${service} restarted`);
      invalidate();
    },
    onError: (err) => toast.error(String(err)),
  });

  return { startMutation, stopMutation, restartMutation };
}
