import { useEffect } from "react";

type Hotkey = {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  shift?: boolean;
  handler: () => void;
};

export function useHotkeys(hotkeys: Hotkey[]) {
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      for (const h of hotkeys) {
        const ctrl = h.ctrl ?? false;
        const meta = h.meta ?? false;
        const shift = h.shift ?? false;
        if (
          e.key.toLowerCase() === h.key.toLowerCase() &&
          e.ctrlKey === ctrl &&
          e.metaKey === meta &&
          e.shiftKey === shift &&
          !e.repeat
        ) {
          e.preventDefault();
          h.handler();
          return;
        }
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [hotkeys]);
}
