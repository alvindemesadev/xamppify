import { useState, useRef, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Search, ArrowRight } from "lucide-react";
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
  { id: "backups", label: "Go to Backups", to: "/backups" },
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
      <DialogContent showCloseButton={false} className="top-[15%] -translate-y-0 p-0">
        <div className="flex items-center gap-2 border-b border-border px-4">
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
            className="border-0 bg-transparent px-0 shadow-none focus-visible:ring-0"
          />
        </div>
        <div className="max-h-72 overflow-y-auto p-2">
          {filtered.length === 0 && <p className="py-8 text-center text-sm text-muted-foreground">No results</p>}
          {filtered.map((action, i) => (
            <button
              key={action.id}
              className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-sm ${i === selectedIdx ? "bg-primary/10 text-primary" : "text-foreground hover:bg-muted"}`}
              onClick={() => execute(action)}
              onMouseEnter={() => setSelectedIdx(i)}
            >
              <ArrowRight className="size-3.5 shrink-0 text-muted-foreground" />
              <span>{action.label}</span>
            </button>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
