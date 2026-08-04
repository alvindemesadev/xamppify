import { useMemo, useState } from "react";
import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ConfirmActionDialog } from "@/components/ui/ConfirmActionDialog";
import { Monitor, Globe, Database, Upload, Circle, Play, Square, RefreshCw, RotateCw } from "lucide-react";
import { useServiceControl, useServiceStatus } from "@/hooks/use-service";
import { useUiStore } from "@/stores/ui-store";
import type { Machine, ServiceStatus } from "@/lib/types";

interface MachineCardProps { machine: Machine; }
type PendingAction = { service: string; action: "start" | "stop" | "restart" | "start-all" | "stop-all" } | null;

function ServiceIcon({ name }: { name: string }) {
  if (name === "Apache") return <Globe className="size-3" />;
  if (name === "MySQL") return <Database className="size-3" />;
  if (name === "FileZilla") return <Upload className="size-3" />;
  return <Circle className="size-3" />;
}

function StatusDot({ status }: { status: string }) {
  const color = { Running: "text-emerald-500", Stopped: "text-rose-500", Starting: "text-amber-500", Error: "text-rose-500" }[status] ?? "text-zinc-500";
  return <Circle aria-hidden className={`size-2.5 fill-current ${color}`} />;
}

export function MachineCard({ machine }: MachineCardProps) {
  const { startMutation, stopMutation, restartMutation } = useServiceControl(machine.id);
  const { data: currentServices, isFetching, refetch } = useServiceStatus(machine.id);
  const selectedMachineId = useUiStore((state) => state.selectedMachineId);
  const setSelectedMachineId = useUiStore((state) => state.setSelectedMachineId);
  const [pendingAction, setPendingAction] = useState<PendingAction>(null);
  const services = currentServices ?? machine.services;
  const isPending = startMutation.isPending || stopMutation.isPending || restartMutation.isPending;
  const actionName = pendingAction?.action.replace("-", " ") ?? "";
  const lastSeen = useMemo(() => new Date(machine.last_seen).toLocaleString(), [machine.last_seen]);

  const runAction = async () => {
    if (!pendingAction) return;
    const invoke = pendingAction.action.startsWith("start") ? startMutation.mutateAsync : pendingAction.action.startsWith("stop") ? stopMutation.mutateAsync : restartMutation.mutateAsync;
    const targets = pendingAction.action.endsWith("-all") ? services.map((service) => service.name) : [pendingAction.service];
    try {
      for (const service of targets) await invoke(service);
      await refetch();
      setPendingAction(null);
    } catch {
      // Individual mutations already provide a toast with the backend error.
    }
  };

  return (
    <>
      <Card className={`p-4 transition-colors ${selectedMachineId === machine.id ? "border-primary ring-1 ring-primary/40" : "hover:border-zinc-600"}`}>
        <div className="mb-3 flex items-start justify-between gap-3">
          <button onClick={() => setSelectedMachineId(machine.id)} className="flex min-w-0 items-center gap-2 text-left" aria-label={`Select ${machine.hostname} as active machine`}>
            <Monitor className="size-4 shrink-0 text-muted-foreground" />
            <span className="truncate text-sm font-medium">{machine.hostname}</span>
          </button>
          <Badge variant={machine.online ? "default" : "secondary"} className="text-xs">{machine.online ? "Online" : "Offline"}</Badge>
        </div>

        <div className="mb-3 flex items-center justify-between gap-2 text-xs text-muted-foreground"><span className="truncate">{machine.ip}</span><span className="shrink-0" title={machine.last_seen}>Seen {lastSeen}</span></div>

        <div className="mb-3 space-y-2">
          {services.length === 0 ? <p className="text-xs text-muted-foreground">{isFetching ? "Checking services…" : "No supported services detected"}</p> : services.map((service: ServiceStatus) => (
            <div key={service.name} className="flex items-center gap-2 text-xs">
              <ServiceIcon name={service.name} /><StatusDot status={service.status} />
              <span className="w-16 truncate text-foreground">{service.name}</span><span className="flex-1 text-muted-foreground">{service.status} · :{service.port}</span>
              <div className="flex gap-1">
                {service.status !== "Running" && <Button variant="ghost" size="icon-xs" disabled={isPending || !machine.online} onClick={() => setPendingAction({ service: service.name, action: "start" })} aria-label={`Start ${service.name}`} title={`Start ${service.name}`}><Play /></Button>}
                {service.status === "Running" && <Button variant="ghost" size="icon-xs" disabled={isPending || !machine.online} onClick={() => setPendingAction({ service: service.name, action: "restart" })} aria-label={`Restart ${service.name}`} title={`Restart ${service.name}`}><RefreshCw /></Button>}
                {service.status === "Running" && <Button variant="destructive" size="icon-xs" disabled={isPending || !machine.online} onClick={() => setPendingAction({ service: service.name, action: "stop" })} aria-label={`Stop ${service.name}`} title={`Stop ${service.name}`}><Square /></Button>}
              </div>
            </div>
          ))}
        </div>

        <div className="flex flex-wrap items-center justify-between gap-1 border-t border-border pt-2 text-xs text-muted-foreground">
          <div className="flex gap-1"><Button variant="ghost" size="xs" disabled={isPending || !machine.online || !services.length} onClick={() => setPendingAction({ service: "all services", action: "start-all" })}>Start all</Button><Button variant="ghost" size="xs" disabled={isPending || !machine.online || !services.length} onClick={() => setPendingAction({ service: "all services", action: "stop-all" })}>Stop all</Button></div>
          <Button variant="ghost" size="icon-xs" onClick={() => refetch()} disabled={isFetching} aria-label={`Refresh ${machine.hostname}`} title="Refresh service status"><RotateCw className={isFetching ? "animate-spin" : ""} /></Button>
        </div>
      </Card>
      <ConfirmActionDialog open={!!pendingAction} onOpenChange={(open) => !open && setPendingAction(null)} title={`${actionName[0]?.toUpperCase()}${actionName.slice(1)} ${pendingAction?.service ?? ""}?`} description={`This will ${actionName} on ${machine.hostname} (${machine.ip}). Remote Windows actions require service-control permission.`} confirmLabel={actionName || "Confirm"} destructive={pendingAction?.action.startsWith("stop") ?? false} pending={isPending} onConfirm={runAction} />
    </>
  );
}
