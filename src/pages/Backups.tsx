import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Database, Download, Trash2, Upload, FileText } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { PageHeader } from "@/components/layout/PageHeader";
import { Skeleton } from "@/components/ui/Skeleton";
import { ConfirmActionDialog } from "@/components/ui/ConfirmActionDialog";
import { useState } from "react";
import { formatBytes } from "@/lib/utils";
import { createBackup, listBackups, deleteBackup, dumpMysql } from "@/lib/ipc";

export default function BackupsPage() {
  const queryClient = useQueryClient();
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  const backupsQuery = useQuery({
    queryKey: ["backups"],
    queryFn: listBackups,
    refetchInterval: 15_000,
  });

  const onErr = (e: unknown) => toast.error(String(e));

  const createMutation = useMutation({
    mutationFn: () => createBackup(),
    onSuccess: () => {
      toast.success("Backup created");
      queryClient.invalidateQueries({ queryKey: ["backups"] });
    },
    onError: onErr,
  });

  const mysqlDumpMutation = useMutation({
    mutationFn: dumpMysql,
    onSuccess: (path) => {
      toast.success(`MySQL dump saved to ${path}`);
      queryClient.invalidateQueries({ queryKey: ["backups"] });
    },
    onError: onErr,
  });

  const deleteMutation = useMutation({
    mutationFn: () => {
      if (!deleteTarget) return Promise.reject("no target");
      return deleteBackup(deleteTarget);
    },
    onSuccess: () => {
      toast.success("Backup deleted");
      setDeleteTarget(null);
      queryClient.invalidateQueries({ queryKey: ["backups"] });
    },
    onError: onErr,
  });

  return (
    <div className="flex h-full flex-col p-6">
      <PageHeader
        title="Backups"
        description="Create and restore htdocs + MySQL database backups."
        actions={
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => mysqlDumpMutation.mutate()} disabled={mysqlDumpMutation.isPending}>
              <Database />{mysqlDumpMutation.isPending ? "Dumping…" : "MySQL dump"}
            </Button>
            <Button onClick={() => createMutation.mutate()} disabled={createMutation.isPending}>
              <Upload />{createMutation.isPending ? "Creating…" : "Create backup"}
            </Button>
          </div>
        }
      />
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">
        {backupsQuery.isLoading ? (
          Array.from({ length: 6 }).map((_, i) => (
            <div key={i} className="rounded-xl border border-border bg-card p-4 shadow-sm">
              <Skeleton className="mb-3 h-4 w-3/4" />
              <Skeleton className="mb-2 h-3 w-1/2" />
              <Skeleton className="h-3 w-1/3" />
            </div>
          ))
        ) : backupsQuery.isError ? (
          <div className="col-span-full flex flex-col items-center justify-center rounded-xl border border-dashed border-destructive/50 bg-card p-8 text-center">
            <p className="text-sm text-destructive">Failed to load backups: {String(backupsQuery.error)}</p>
          </div>
        ) : backupsQuery.data?.length === 0 ? (
          <div className="col-span-full flex flex-col items-center justify-center rounded-xl border border-dashed border-border bg-card p-8 text-center">
            <FileText className="mb-4 size-12 text-muted-foreground" />
            <h2 className="text-lg font-semibold">No backups yet</h2>
            <p className="mt-2 max-w-md text-sm text-muted-foreground">Create your first backup to protect your htdocs and databases.</p>
          </div>
        ) : (
          backupsQuery.data?.map((b) => (
            <div key={b.name} className="rounded-xl border border-border bg-card p-4 shadow-sm transition-all hover:border-primary/30 hover:shadow-md">
              <div className="mb-3 flex items-start justify-between">
                <div className="min-w-0">
                  <p className="truncate font-medium">{b.name}</p>
                  <p className="mt-0.5 text-xs text-muted-foreground">{b.created}</p>
                </div>
                <Button variant="ghost" size="icon-sm" onClick={() => setDeleteTarget(b.name)} aria-label="Delete backup"><Trash2 className="size-4 text-destructive" /></Button>
              </div>
              <div className="flex items-center gap-2 text-xs text-muted-foreground">
                <Download className="size-3.5" />
                <span>{formatBytes(b.size)}</span>
              </div>
            </div>
          ))
        )}
      </div>
      <ConfirmActionDialog
        open={!!deleteTarget}
        onOpenChange={(v) => { if (!v) setDeleteTarget(null); }}
        title="Delete backup"
        description="This action cannot be undone. The backup file will be permanently removed."
        confirmLabel="Delete"
        destructive
        pending={deleteMutation.isPending}
        onConfirm={() => deleteMutation.mutate()}
      />
    </div>
  );
}
