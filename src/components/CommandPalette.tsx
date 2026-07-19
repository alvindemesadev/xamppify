import { useState, useRef, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Search, ArrowRight, Command, CornerDownLeft } from "lucide-react";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { useHotkeys } from "@/hooks/use-hotkeys";

type Action = {
  id: string;
  label: string;
  to?: string;
  action?: () => void;
};

const defaultActions: Action[] = [
  { id: "deployments", label: "Go to Deployments", to: "/" },
  { id: "logs", label: "Go to Logs", to: "/logs" },
  { id: "files", label: "Go to Files", to: "/files" },
  { id: "database", label: "Go to Database", to: "/database" },
  { id: "config", label: "Go to Config", to: "/config" },
  { id: "ssl", label: "Go to SSL", to: "/ssl" },
  { id: "file-sync", label: "Go to File Sync", to: "/file-sync" },
  { id: "performance", label: "Go to Performance", to: "/performance" },
  { id: "settings", label: "Go to Settings", to: "/settings" },
];

export function CommandPalette() {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [selectedIdx, setSelectedIdx] = useState(0);
  const navigate = useNavigate();
  const inputRef = useRef<HTMLInputElement>(null);

  const isMac = navigator.platform.toUpperCase().includes("MAC");
  useHotkeys([
    { key: "k", ctrl: !isMac, meta: isMac, handler: () => setOpen((v) => !v) },
  ]);

  useEffect(() => {
    if (open) {
      setQuery("");
      setSelectedIdx(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [open]);

  const filtered = query.trim()
    ? defaultActions.filter((a) => a.label.toLowerCase().includes(query.toLowerCase()))
    : defaultActions;

  const execute = useCallback((action: Action) => {
    setOpen(false);
    if (action.action) {
      action.action();
    } else if (action.to) {
      navigate(action.to);
    }
  }, [navigate]);

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent showCloseButton={false} className="top-[12%] -translate-y-0 w-full max-w-lg p-0 overflow-hidden">
        <div className="flex items-center gap-3 border-b border-border px-4">
          <Search className="size-4 shrink-0 text-muted-foreground" />
          <Input
            ref={inputRef}
            value={query}
            onChange={(e) => { setQuery(e.target.value); setSelectedIdx(0); }}
            onKeyDown={(e) => {
              if (e.key === "ArrowDown") { e.preventDefault(); setSelectedIdx((i) => Math.min(i + 1, filtered.length - 1)); }
              if (e.key === "ArrowUp") { e.preventDefault(); setSelectedIdx((i) => Math.max(i - 1, 0)); }
              if (e.key === "Enter" && filtered[selectedIdx]) { execute(filtered[selectedIdx]); }
            }}
            placeholder="Search pages…"
            className="border-0 bg-transparent shadow-none focus-visible:ring-0 h-11 text-sm"
          />
        </div>
        <div className="max-h-80 overflow-y-auto p-1.5">
          {filtered.length === 0 && (
            <div className="flex flex-col items-center gap-1 py-10">
              <p className="text-sm text-muted-foreground">No results</p>
              <p className="text-xs text-muted-foreground/60">Try a different search term</p>
            </div>
          )}
          {filtered.map((action, i) => (
            <button
              key={action.id}
              className={`flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left text-sm transition-colors ${
                i === selectedIdx
                  ? "bg-primary text-primary-foreground"
                  : "text-foreground hover:bg-muted"
              }`}
              onClick={() => execute(action)}
              onMouseEnter={() => setSelectedIdx(i)}
            >
              <ArrowRight className={`size-3.5 shrink-0 ${i === selectedIdx ? "text-primary-foreground/70" : "text-muted-foreground"}`} />
              <span className="flex-1">{action.label}</span>
              {i === selectedIdx && (
                <CornerDownLeft className="size-3.5 shrink-0 text-primary-foreground/50" />
              )}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-4 border-t border-border px-4 py-2">
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground/60">
            <kbd className="flex size-4 items-center justify-center rounded border border-border bg-muted text-[10px] font-medium text-muted-foreground">
              {isMac ? <Command className="size-2.5" /> : "^"}
            </kbd>
            <span>K</span>
            <span className="mx-1">to toggle</span>
          </div>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground/60">
            <kbd className="flex items-center gap-0.5 rounded border border-border bg-muted px-1 py-0.5 text-[10px] font-medium text-muted-foreground">
              <span>↑</span><span>↓</span>
            </kbd>
            <span>to navigate</span>
          </div>
          <div className="flex items-center gap-1.5 text-xs text-muted-foreground/60">
            <CornerDownLeft className="size-3" />
            <span>to open</span>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
