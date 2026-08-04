import { Circle, Database, Globe, Play, RefreshCw, Square, Upload } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useServiceControl, useServiceStatus } from "@/hooks/use-service";
import { useUiStore } from "@/stores/ui-store";
import { getDiscoveredMachines } from "@/lib/ipc";
import type { ServiceStatus } from "@/lib/types";
import { Button } from "@/components/ui/button";

function ServiceIcon({ name }: { name: string }) {
  if (name === "Apache") return <Globe className="size-3.5" />;
  if (name === "MySQL") return <Database className="size-3.5" />;
  if (name === "FileZilla") return <Upload className="size-3.5" />;
  return <Circle className="size-3.5" />;
}

function StatusDot({ status }: { status: string }) {
  const color = {
    Running: "text-emerald-500",
    Stopped: "text-rose-500",
    Starting: "text-amber-500",
    Error: "text-rose-500",
  }[status] ?? "text-zinc-500";
  return <Circle aria-hidden className={`size-2 fill-current ${color}`} />;
}

export function ServiceStrip() {
  const selectedMachineId = useUiStore((state) => state.selectedMachineId);
  const { data: machines } = useQuery({
    queryKey: ["discovered-machines"],
    queryFn: getDiscoveredMachines,
  });
  const machineId =
    selectedMachineId && machines?.some((m) => m.id === selectedMachineId)
      ? selectedMachineId
      : machines?.[0]?.id ?? null;
  const machine = machines?.find((m) => m.id === machineId);
  const { data: services, isFetching, refetch } = useServiceStatus(machineId ?? "");
  const { startMutation, stopMutation, restartMutation } = useServiceControl(machineId ?? "");
  const isPending = startMutation.isPending || stopMutation.isPending || restartMutation.isPending;

  if (!machineId || !machine) return null;

  const listed = services ?? machine.services;

  return (
    <div className="mb-5 flex flex-wrap items-center gap-2 rounded-xl border border-border bg-card p-3 shadow-sm">
      <div className="mr-1 flex items-center gap-2 text-xs font-medium text-muted-foreground">
        <Circle className="size-2.5 fill-current text-emerald-500" />
        <span className="max-w-32 truncate">{machine.hostname}</span>
        <span className="font-mono text-muted-foreground/60">{machine.ip}</span>
      </div>
      <div className="h-6 w-px bg-border" />
      <div className="flex flex-wrap items-center gap-2">
        {listed.length === 0 ? (
          <p className="px-2 text-xs text-muted-foreground">
            {isFetching ? "Checking services…" : "No supported services detected"}
          </p>
        ) : (
          listed.map((service: ServiceStatus) => (
            <div
              key={service.name}
              className="flex items-center gap-2 rounded-lg border border-border bg-muted/40 px-3 py-1.5 text-xs"
            >
              <ServiceIcon name={service.name} />
              <StatusDot status={service.status} />
              <span className="font-medium text-foreground">{service.name}</span>
              <span className="text-muted-foreground">
                {service.status} · :{service.port}
              </span>
              <div className="ml-1 flex gap-0.5">
                {service.status !== "Running" && (
                  <Button
                    variant="ghost"
                    size="icon-xs"
                    disabled={isPending}
                    onClick={() => startMutation.mutate(service.name)}
                    aria-label={`Start ${service.name}`}
                    title={`Start ${service.name}`}
                  >
                    <Play />
                  </Button>
                )}
                {service.status === "Running" && (
                  <>
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      disabled={isPending}
                      onClick={() => restartMutation.mutate(service.name)}
                      aria-label={`Restart ${service.name}`}
                      title={`Restart ${service.name}`}
                    >
                      <RefreshCw />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      disabled={isPending}
                      onClick={() => stopMutation.mutate(service.name)}
                      aria-label={`Stop ${service.name}`}
                      title={`Stop ${service.name}`}
                    >
                      <Square />
                    </Button>
                  </>
                )}
              </div>
            </div>
          ))
        )}
      </div>
      <div className="ml-auto">
        <Button variant="ghost" size="icon-sm" onClick={() => refetch()} disabled={isFetching} aria-label="Refresh service status" title="Refresh service status">
          <RefreshCw className={isFetching ? "animate-spin" : ""} />
        </Button>
      </div>
    </div>
  );
}
