import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { ArrowRightFromLine, CheckCircle2, XCircle } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { PageHeader } from "@/components/layout/PageHeader";
import { syncToRemote } from "@/lib/ipc";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

type SyncLogItem = {
  result: "success" | "error";
  message: string;
};

export default function FileSyncPage() {
  const [source, setSource] = useState("");
  const [dest, setDest] = useState("");
  const [remoteHost, setRemoteHost] = useState("");
  const [logs, setLogs] = useState<SyncLogItem[]>([]);

  const syncMutation = useMutation({
    mutationFn: () => syncToRemote(source, dest, remoteHost),
    onSuccess: (res) => {
      if (res.success) {
        toast.success(`Synced ${res.files_copied} items`);
        setLogs((prev) => [...prev, { result: "success", message: `Synced to ${remoteHost}: ${res.files_copied} items` }]);
      } else {
        toast.error("Sync completed with errors");
        setLogs((prev) => [...prev, { result: "error", message: `Sync errors:\n${res.output}` }]);
      }
    },
    onError: (e: unknown) => {
      const msg = String(e);
      toast.error(msg);
      setLogs((prev) => [...prev, { result: "error", message: msg }]);
    },
  });

  return (
    <div className="flex h-full flex-col p-6">
      <PageHeader title="File Sync" description="Sync XAMPP files to remote machines using robocopy (Windows only)." />
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_1fr]">
        <Card>
          <CardHeader>
            <CardTitle>Sync configuration</CardTitle>
          </CardHeader>
          <CardContent className="grid gap-4">
            <label className="grid gap-1.5 text-sm">
              <span className="text-muted-foreground">Source path</span>
              <Input value={source} onChange={(e) => setSource(e.target.value)} placeholder="C:\xampp\htdocs" />
            </label>
            <label className="grid gap-1.5 text-sm">
              <span className="text-muted-foreground">Remote host (IP or hostname)</span>
              <Input value={remoteHost} onChange={(e) => setRemoteHost(e.target.value)} placeholder="192.168.1.100" />
            </label>
            <label className="grid gap-1.5 text-sm">
              <span className="text-muted-foreground">Destination path on remote</span>
              <Input value={dest} onChange={(e) => setDest(e.target.value)} placeholder="C:\xampp\htdocs" />
            </label>
            <Button onClick={() => syncMutation.mutate()} disabled={syncMutation.isPending || !source || !remoteHost || !dest}>
              <ArrowRightFromLine />{syncMutation.isPending ? "Syncing…" : "Sync now"}
            </Button>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Sync history</CardTitle>
          </CardHeader>
          <CardContent className="max-h-96 space-y-2 overflow-y-auto">
            {logs.length === 0 && <p className="py-8 text-center text-sm text-muted-foreground">No sync operations yet.</p>}
            {logs.map((log, i) => (
              <div key={i} className={`flex items-start gap-2 rounded-lg p-2 text-sm ${log.result === "success" ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-400" : "bg-red-500/10 text-red-700 dark:text-red-400"}`}>
                {log.result === "success" ? <CheckCircle2 className="mt-0.5 size-4 shrink-0" /> : <XCircle className="mt-0.5 size-4 shrink-0" />}
                <span className="whitespace-pre-wrap">{log.message}</span>
              </div>
            ))}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
