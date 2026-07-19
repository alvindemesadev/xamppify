import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getLogs, startLogWatcher } from "@/lib/ipc";
import type { LogLine } from "@/lib/types";

export function useLogs(source: string, maxLines = 100) {
  const [logs, setLogs] = useState<LogLine[]>([]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      const initial = await getLogs(source, maxLines);
      setLogs(initial);
      startLogWatcher(source).catch(() => {});

      unlisten = await listen<{ source: string; lines: LogLine[] }>("log-update", (event) => {
        if (event.payload.source.toLowerCase() === source.toLowerCase()) {
          setLogs(event.payload.lines);
        }
      });
    })();

    return () => {
      unlisten?.();
    };
  }, [source, maxLines]);

  return { data: logs };
}
