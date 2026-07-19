import { useQuery } from "@tanstack/react-query";
import { Activity, Cpu, HardDrive, Server, Timer } from "lucide-react";
import { getLocalPerformance } from "@/lib/ipc";
import { PageHeader } from "@/components/layout/PageHeader";
import { Skeleton } from "@/components/ui/Skeleton";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

function Gauge({ label, value, icon: Icon, color }: { label: string; value: number; icon: typeof Cpu; color: string }) {
  const clamped = Math.min(value, 100);
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
          <Icon className="size-4" />{label}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="mb-2 flex items-end gap-2">
          <span className="text-3xl font-bold">{clamped.toFixed(1)}</span>
          <span className="mb-1 text-sm text-muted-foreground">%</span>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-muted">
          <div className="h-full rounded-full transition-all duration-500" style={{ width: `${clamped}%`, backgroundColor: color }} />
        </div>
      </CardContent>
    </Card>
  );
}

export default function PerformancePage() {
  const { data, isLoading, error } = useQuery({
    queryKey: ["performance"],
    queryFn: getLocalPerformance,
    refetchInterval: 5_000,
  });

  return (
    <div className="flex h-full flex-col p-6">
      <PageHeader title="Performance" description="Real-time CPU, memory, and disk usage for the local machine." />
      {isLoading ? (
        <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <Card key={i}><CardHeader><Skeleton className="h-4 w-24" /></CardHeader><CardContent><Skeleton className="mb-2 h-8 w-20" /><Skeleton className="h-2 w-full" /></CardContent></Card>
          ))}
        </div>
      ) : error ? (
        <div className="flex flex-1 items-center justify-center">
          <p className="text-destructive">Failed to load performance data: {String(error)}</p>
        </div>
      ) : data ? (
        <div className="space-y-6">
          <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
            <Gauge label="CPU" value={data.cpu_percent} icon={Cpu} color="#3b82f6" />
            <Gauge label="Memory" value={data.memory_percent} icon={Server} color="#8b5cf6" />
            <Gauge label="Disk (C:)" value={data.disk_percent} icon={HardDrive} color="#10b981" />
          </div>
          <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                  <Activity className="size-4" />Memory details
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-1 text-sm">
                <p><span className="text-muted-foreground">Used:</span> {data.memory_used_gb.toFixed(1)} GB</p>
                <p><span className="text-muted-foreground">Total:</span> {data.memory_total_gb.toFixed(1)} GB</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                  <HardDrive className="size-4" />Disk details
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-1 text-sm">
                <p><span className="text-muted-foreground">Free:</span> {data.disk_free_gb.toFixed(1)} GB</p>
                <p><span className="text-muted-foreground">Total:</span> {data.disk_total_gb.toFixed(1)} GB</p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                  <Timer className="size-4" />Uptime
                </CardTitle>
              </CardHeader>
              <CardContent>
                <span className="text-3xl font-bold">{data.uptime_days.toFixed(1)}</span>
                <span className="ml-1 text-sm text-muted-foreground">days</span>
              </CardContent>
            </Card>
          </div>
        </div>
      ) : null}
    </div>
  );
}
