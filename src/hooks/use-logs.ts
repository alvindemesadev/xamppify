import { useQuery } from "@tanstack/react-query";
import { getLogs } from "@/lib/ipc";

export function useLogs(source: string, maxLines = 100) {
  return useQuery({
    queryKey: ["logs", source],
    queryFn: () => getLogs(source, maxLines),
  });
}
