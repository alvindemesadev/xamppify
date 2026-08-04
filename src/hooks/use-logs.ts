import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getLogs, startLogWatcher } from "@/lib/ipc";
import type { LogLine } from "@/lib/types";

export function useLogs(source: string, paused = false, maxLines = 100) {
  const [logs, setLogs] = useState<LogLine[]>([]);
  const pausedRef = useRef(paused);
  const pendingRef = useRef<LogLine[] | null>(null);

  useEffect(() => {
    pausedRef.current = paused;
    if (!paused && pendingRef.current !== null) {
      setLogs(pendingRef.current);
      pendingRef.current = null;
    }
  }, [paused]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    (async () => {
      const initial = await getLogs(source, maxLines);
      setLogs(initial);
      startLogWatcher(source).catch(() => {});

      unlisten = await listen<{ source: string; lines: LogLine[] }>("log-update", (event) => {
        if (event.payload.source.toLowerCase() !== source.toLowerCase()) return;
        if (pausedRef.current) {
          pendingRef.current = event.payload.lines;
        } else {
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
