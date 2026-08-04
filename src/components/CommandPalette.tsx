import { useState, useRef, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Search, ArrowRight, Command, CornerDownLeft, Globe, Database, Play, Square } from "lucide-react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { useHotkeys } from "@/hooks/use-hotkeys";
import { useUiStore } from "@/stores/ui-store";
import { getDiscoveredMachines, getServiceStatus, listDeployments, restartService, startService, stopService } from "@/lib/ipc";

type Action = {
  id: string;
  label: string;
  hint?: string;
  to?: string;
  action?: () => Promise<void> | void;
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
  const queryClient = useQueryClient();
  const selectedMachineId = useUiStore((state) => state.selectedMachineId);
  const { data: machines } = useQuery({
    queryKey: ["discovered-machines"],
    queryFn: getDiscoveredMachines,
  });
  const { data: deployments } = useQuery({
    queryKey: ["deployments"],
    queryFn: listDeployments,
  });

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

  const machineId =
    selectedMachineId && machines?.some((m) => m.id === selectedMachineId)
      ? selectedMachineId
      : machines?.[0]?.id ?? null;
  const machineName = machines?.find((m) => m.id === machineId)?.hostname ?? null;

  const runServiceAction = useCallback(
    async (fn: () => Promise<void>, message: string) => {
      try {
        await fn();
        toast.success(message);
        if (machineId) queryClient.invalidateQueries({ queryKey: ["services", machineId] });
      } catch (err) {
        toast.error(String(err));
      }
    },
    [machineId, queryClient],
  );

  const serviceActions: Action[] = machineId
    ? [
        {
          id: "restart-apache",
          label: "Restart Apache",
          hint: machineName ? `${machineName} service` : "service",
          action: () =>
            runServiceAction(
              () => restartService(machineId, "Apache"),
              "Apache restarted",
            ),
        },
        {
          id: "restart-mysql",
          label: "Restart MySQL",
          hint: machineName ? `${machineName} service` : "service",
          action: () =>
            runServiceAction(
              () => restartService(machineId, "MySQL"),
              "MySQL restarted",
            ),
        },
        {
          id: "start-all",
          label: "Start all services",
          hint: machineName ? machineName : "service",
          action: async () => {
            const services = await getServiceStatus(machineId);
            for (const service of services.filter((s) => s.status !== "Running")) {
              await runServiceAction(
                () => startService(machineId, service.name),
                `${service.name} started`,
              );
            }
          },
        },
        {
          id: "stop-all",
          label: "Stop all services",
          hint: machineName ? machineName : "service",
          action: async () => {
            const services = await getServiceStatus(machineId);
            for (const service of services.filter((s) => s.status === "Running")) {
              await runServiceAction(
                () => stopService(machineId, service.name),
                `${service.name} stopped`,
              );
            }
          },
        },
      ]
    : [];

  const deploymentActions: Action[] = (deployments ?? []).slice(0, 10).map((d) => ({
    id: `open-${d.name}`,
    label: `Open ${d.name}`,
    hint: "deployment",
    action: () => navigate("/files", { state: { deploymentPath: d.path, deploymentName: d.name } }),
  }));

  const allActions: Action[] = [...defaultActions, ...serviceActions, ...deploymentActions];

  const filtered = query.trim()
    ? allActions.filter((a) =>
        (a.label + (a.hint ? ` ${a.hint}` : "")).toLowerCase().includes(query.toLowerCase()),
      )
    : allActions;

  const execute = useCallback(
    (action: Action) => {
      setOpen(false);
      if (action.action) {
        void action.action();
      } else if (action.to) {
        navigate(action.to);
      }
    },
    [navigate],
  );

  const actionIcon = (id: string, selected: boolean) => {
    const color = selected ? "text-primary-foreground/70" : "text-muted-foreground";
    if (id === "restart-apache") return <Globe className={`size-3.5 shrink-0 ${color}`} />;
    if (id === "restart-mysql") return <Database className={`size-3.5 shrink-0 ${color}`} />;
    if (id === "start-all") return <Play className={`size-3.5 shrink-0 ${color}`} />;
    if (id === "stop-all") return <Square className={`size-3.5 shrink-0 ${color}`} />;
    return <ArrowRight className={`size-3.5 shrink-0 ${color}`} />;
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent showCloseButton={false} className="top-[12%] -translate-y-0 w-full max-w-lg p-0 overflow-hidden">
        <div className="flex items-center gap-3 border-b border-border px-4 py-1.5">
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
            placeholder="Search pages or run commands…"
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
              {actionIcon(action.id, i === selectedIdx)}
              <span className="flex-1">{action.label}</span>
              {action.hint && (
                <span className={`text-xs ${i === selectedIdx ? "text-primary-foreground/60" : "text-muted-foreground/70"}`}>
                  {action.hint}
                </span>
              )}
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
            <span>to run</span>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
