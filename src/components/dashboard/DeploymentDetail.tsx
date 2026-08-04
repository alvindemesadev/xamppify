import { useEffect, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Archive,
  Code2,
  Copy,
  Database,
  FolderGit2,
  Globe,
  KeyRound,
  Pin,
  PinOff,
  RefreshCw,
  Shield,
  ShieldCheck,
  Tag,
  Trash2,
  X,
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { ConfirmActionDialog } from "@/components/ui/ConfirmActionDialog";
import { toast } from "sonner";
import {
  backupDeployment, deleteDeployment, duplicateDeployment, enableDeploymentSsl, getGitInfo, listBackups,
  provisionDatabase, readDeploymentEnv, restoreDeploymentBackup, runDependencyCommand,
  setCustomDomain, setLinkedDatabase, toggleVhost, updateDeploymentMeta, writeDeploymentEnv,
} from "@/lib/ipc";
import type { Deployment } from "@/lib/types";

interface Props {
  deployment: Deployment | null;
  onClose: () => void;
  onChanged: () => void;
}

export function DeploymentDetail({ deployment, onClose, onChanged }: Props) {
  const queryClient = useQueryClient();
  const [envDraft, setEnvDraft] = useState<string | null>(null);
  const [envLoaded, setEnvLoaded] = useState(false);
  const [domainDraft, setDomainDraft] = useState("");
  const [newName, setNewName] = useState("");
  const [tagDraft, setTagDraft] = useState("");
  const [dbName, setDbName] = useState("");
  const [dupOpen, setDupOpen] = useState(false);
  const [restoreOpen, setRestoreOpen] = useState(false);
  const [selectedBackup, setSelectedBackup] = useState<string | null>(null);
  const [envOpen, setEnvOpen] = useState(false);
  const [depsOutput, setDepsOutput] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);

  const { data: backups } = useQuery({
    queryKey: ["backups", deployment?.name],
    queryFn: () => listBackups(deployment?.name),
    enabled: !!deployment,
  });
  const { data: git } = useQuery({
    queryKey: ["git-info", deployment?.name],
    queryFn: () => getGitInfo(deployment!.name),
    enabled: !!deployment,
  });

  useEffect(() => {
    if (deployment && !envLoaded) {
      setDomainDraft(deployment.custom_domain ?? "");
      setDbName(deployment.linked_db ?? "");
      setEnvLoaded(true);
    }
  }, [deployment, envLoaded]);

  const changed = () => {
    onChanged();
    queryClient.invalidateQueries({ queryKey: ["deployments"] });
  };

  const envQuery = useQuery({
    queryKey: ["env", deployment?.name],
    queryFn: () => readDeploymentEnv(deployment!.name),
    enabled: !!deployment && !!deployment.has_env && envOpen,
  });

  const backupMutation = useMutation({
    mutationFn: () => backupDeployment(deployment!.name),
    onSuccess: (path) => { toast.success(`Backup saved to ${path}`); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const togglePinMutation = useMutation({
    mutationFn: () => updateDeploymentMeta(deployment!.name, { pinned: !deployment!.pinned }),
    onSuccess: (updated) => { toast.success(updated.pinned ? "Pinned to top" : "Unpinned"); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const domainMutation = useMutation({
    mutationFn: () => setCustomDomain(deployment!.name, domainDraft.trim() || null),
    onSuccess: (updated) => { setDomainDraft(updated.custom_domain ?? ""); toast.success("Custom domain updated"); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const vhostMutation = useMutation({
    mutationFn: () => toggleVhost(deployment!.name, !deployment!.vhost_enabled),
    onSuccess: (updated) => { toast.success(updated.vhost_enabled ? "Virtual host enabled" : "Virtual host disabled"); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const sslMutation = useMutation({
    mutationFn: () => enableDeploymentSsl(deployment!.name),
    onSuccess: () => { toast.success("HTTPS certificate created"); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const envSaveMutation = useMutation({
    mutationFn: () => writeDeploymentEnv(deployment!.name, envDraft ?? ""),
    onSuccess: () => { toast.success(".env saved"); setEnvOpen(false); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const addTagMutation = useMutation({
    mutationFn: () => updateDeploymentMeta(deployment!.name, { tags: [...deployment!.tags, tagDraft.trim()] }),
    onSuccess: () => { setTagDraft(""); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const removeTagMutation = useMutation({
    mutationFn: (tag: string) => updateDeploymentMeta(deployment!.name, { tags: deployment!.tags.filter((t) => t !== tag) }),
    onSuccess: () => changed(),
    onError: (e: Error) => toast.error(e.message),
  });
  const dupMutation = useMutation({
    mutationFn: () => duplicateDeployment(deployment!.name, newName),
    onSuccess: () => { setDupOpen(false); setNewName(""); toast.success("Deployment duplicated"); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const dbMutation = useMutation({
    mutationFn: () => provisionDatabase(deployment!.name, dbName.trim() || undefined),
    onSuccess: async (created) => {
      await setLinkedDatabase(deployment!.name, created);
      setDbName(created);
      toast.success(`Database '${created}' created and linked`);
      changed();
    },
    onError: (e: Error) => toast.error(e.message),
  });
  const restoreMutation = useMutation({
    mutationFn: () => restoreDeploymentBackup(deployment!.name, selectedBackup!),
    onSuccess: () => { setRestoreOpen(false); setSelectedBackup(null); toast.success("Backup restored"); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });
  const depsMutation = useMutation({
    mutationFn: ({ tool, action }: { tool: string; action: string }) => runDependencyCommand(deployment!.name, tool, action),
    onSuccess: (res) => { setDepsOutput(res.output || (res.success ? "Done" : "Failed")); if (!res.success) toast.error("Command reported errors"); },
    onError: (e: Error) => toast.error(e.message),
  });
  const deleteMutation = useMutation({
    mutationFn: () => deleteDeployment(deployment!.name),
    onSuccess: () => { setConfirmDelete(false); onClose(); toast.success("Deployment moved to the Recycle Bin"); changed(); },
    onError: (e: Error) => toast.error(e.message),
  });

  if (!deployment) return null;
  const copy = (text: string) => { navigator.clipboard.writeText(text); toast.success("Copied"); };
  const openSite = async () => { try { await openUrl(deployment.url); } catch { toast.error("Could not open the URL"); } };
  const openNetwork = async () => { try { await openUrl(deployment.network_url); } catch { toast.error("Could not open the URL"); } };

  return (
    <>
      <Dialog open={!!deployment} onOpenChange={(open) => !open && onClose()}>
        <DialogContent className="max-h-[85vh] max-w-2xl overflow-y-auto">
          <DialogHeader>
            <div className="flex items-center gap-2">
              <DialogTitle>{deployment.name}</DialogTitle>
              {deployment.pinned && <Pin className="size-4 text-primary" />}
              {deployment.ssl_enabled && <ShieldCheck className="size-4 text-emerald-500" />}
            </div>
            <DialogDescription>
              <span className="rounded bg-muted px-1.5 py-0.5 text-xs">{deployment.framework}</span> · Modified {deployment.modified}
              {deployment.custom_domain && <> · <span className="font-mono">{deployment.custom_domain}</span></>}
              {deployment.linked_db && <> · db <span className="font-mono">{deployment.linked_db}</span></>}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 text-sm">
            <div className="grid gap-2">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Local URL</span>
                <div className="flex items-center gap-1">
                  <button onClick={openSite} className="truncate font-mono text-xs text-primary hover:underline">{deployment.url}</button>
                  <Button size="icon-xs" variant="ghost" onClick={() => copy(deployment.url)}><Copy /></Button>
                </div>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">Network URL</span>
                <div className="flex items-center gap-1">
                  <button onClick={openNetwork} className="truncate font-mono text-xs text-primary hover:underline">{deployment.network_url}</button>
                  <Button size="icon-xs" variant="ghost" onClick={() => copy(deployment.network_url)}><Copy /></Button>
                </div>
              </div>
            </div>

            <div className="rounded-lg border border-border p-3">
              <p className="mb-2 flex items-center gap-1.5 font-medium"><Tag className="size-3.5" /> Tags</p>
              <div className="mb-2 flex flex-wrap gap-1.5">
                {deployment.tags.map((tag) => (
                  <Badge key={tag} variant="secondary" className="cursor-pointer" onClick={() => removeTagMutation.mutate(tag)}>
                    {tag} <X className="ml-1 size-3" />
                  </Badge>
                ))}
                {deployment.tags.length === 0 && <span className="text-xs text-muted-foreground">No tags yet</span>}
              </div>
              <div className="flex gap-2">
                <Input value={tagDraft} onChange={(e) => setTagDraft(e.target.value)} placeholder="Add a tag…" className="h-8 text-xs" onKeyDown={(e) => e.key === "Enter" && tagDraft.trim() && addTagMutation.mutate()} />
                <Button size="sm" variant="outline" disabled={!tagDraft.trim()} onClick={() => addTagMutation.mutate()}>Add</Button>
              </div>
            </div>

            <div className="rounded-lg border border-border p-3">
              <p className="mb-2 flex items-center gap-1.5 font-medium"><Globe className="size-3.5" /> Domain & virtual host</p>
              <div className="flex gap-2">
                <Input value={domainDraft} onChange={(e) => setDomainDraft(e.target.value)} placeholder="myapp.test" className="h-8 text-xs" />
                <Button size="sm" variant="outline" disabled={domainDraft.trim() === (deployment.custom_domain ?? "")} onClick={() => domainMutation.mutate()}>Set domain</Button>
              </div>
              <p className="mt-2 text-xs text-muted-foreground">Adds a VirtualHost to httpd-vhosts.conf and a hosts-file entry (may require admin rights).</p>
              <div className="mt-2 flex flex-wrap gap-2">
                <Button size="sm" variant="outline" onClick={() => vhostMutation.mutate()} disabled={vhostMutation.isPending}>
                  {deployment.vhost_enabled ? "Disable virtual host" : "Enable virtual host"}
                </Button>
                <Button size="sm" variant={deployment.ssl_enabled ? "secondary" : "outline"} onClick={() => sslMutation.mutate()} disabled={sslMutation.isPending}>
                  <Shield className="size-3.5" /> {deployment.ssl_enabled ? "HTTPS on (re-issue cert)" : "Enable HTTPS"}
                </Button>
              </div>
            </div>

            <div className="rounded-lg border border-border p-3">
              <p className="mb-2 flex items-center gap-1.5 font-medium"><Database className="size-3.5" /> Database</p>
              {deployment.linked_db ? (
                <p className="mb-2 font-mono text-xs">{deployment.linked_db}</p>
              ) : (
                <p className="mb-2 text-xs text-muted-foreground">No database linked. Create one to pair with this project.</p>
              )}
              <div className="flex gap-2">
                <Input value={dbName} onChange={(e) => setDbName(e.target.value)} placeholder={`${deployment.name}_db`} className="h-8 text-xs" />
                <Button size="sm" variant="outline" disabled={dbMutation.isPending} onClick={() => dbMutation.mutate()}>
                  <Database className="size-3.5" /> Create & link
                </Button>
              </div>
            </div>

            <div className="rounded-lg border border-border p-3">
              <p className="mb-2 flex items-center gap-1.5 font-medium"><Code2 className="size-3.5" /> Dependencies</p>
              <div className="flex flex-wrap gap-2">
                {deployment.has_composer && (
                  <>
                    <Button size="sm" variant="outline" disabled={depsMutation.isPending} onClick={() => depsMutation.mutate({ tool: "composer", action: "install" })}>composer install</Button>
                    <Button size="sm" variant="outline" disabled={depsMutation.isPending} onClick={() => depsMutation.mutate({ tool: "composer", action: "outdated" })}>composer outdated</Button>
                  </>
                )}
                {deployment.has_package_json && (
                  <>
                    <Button size="sm" variant="outline" disabled={depsMutation.isPending} onClick={() => depsMutation.mutate({ tool: "npm", action: "install" })}>npm install</Button>
                    <Button size="sm" variant="outline" disabled={depsMutation.isPending} onClick={() => depsMutation.mutate({ tool: "npm", action: "build" })}>npm build</Button>
                    <Button size="sm" variant="outline" disabled={depsMutation.isPending} onClick={() => depsMutation.mutate({ tool: "npm", action: "outdated" })}>npm outdated</Button>
                  </>
                )}
                {!deployment.has_composer && !deployment.has_package_json && (
                  <span className="text-xs text-muted-foreground">No composer.json or package.json detected.</span>
                )}
              </div>
              {depsOutput && (
                <pre className="mt-2 max-h-40 overflow-auto rounded bg-muted/40 p-2 font-mono text-[11px] whitespace-pre-wrap">{depsOutput}</pre>
              )}
            </div>

            {git?.is_git && (
              <div className="rounded-lg border border-border p-3">
                <p className="mb-2 flex items-center gap-1.5 font-medium"><FolderGit2 className="size-3.5" /> Git</p>
                <p className="text-xs">
                  Branch: <span className="font-mono">{git.branch}</span> · {git.dirty ? <span className="text-amber-500">uncommitted changes</span> : "clean"}
                </p>
                {git.last_commit && <p className="mt-1 truncate font-mono text-xs text-muted-foreground">{git.last_commit}</p>}
              </div>
            )}

            <div className="rounded-lg border border-border p-3">
              <p className="mb-2 flex items-center gap-1.5 font-medium"><Archive className="size-3.5" /> Backups</p>
              <div className="mb-2 flex flex-wrap gap-2">
                <Button size="sm" variant="outline" disabled={backupMutation.isPending} onClick={() => backupMutation.mutate()}>
                  <Archive className="size-3.5" /> {backupMutation.isPending ? "Backing up…" : "Back up now"}
                </Button>
                <Button size="sm" variant="outline" onClick={() => setRestoreOpen(true)}>Restore from backup</Button>
                <Button size="sm" variant="outline" onClick={() => setDupOpen(true)}>Duplicate…</Button>
              </div>
              <ScrollArea className="max-h-32">
                <div className="space-y-1">
                  {backups && backups.length > 0 ? backups.slice(0, 8).map((b) => (
                    <div key={b.path} className="flex items-center justify-between gap-2 rounded bg-muted/40 px-2 py-1">
                      <span className="truncate font-mono text-[11px]">{b.timestamp} · {(b.size / 1024).toFixed(0)} KB</span>
                    </div>
                  )) : <p className="text-xs text-muted-foreground">No backups yet.</p>}
                </div>
              </ScrollArea>
            </div>

            {deployment.has_env && (
              <div className="rounded-lg border border-border p-3">
                <p className="mb-2 flex items-center gap-1.5 font-medium"><KeyRound className="size-3.5" /> .env</p>
                {envOpen ? (
                  <div className="space-y-2">
                    {envQuery.isLoading ? <p className="text-xs">Loading…</p> : (
                      <textarea
                        className="h-40 w-full resize-none rounded border border-border bg-background p-2 font-mono text-xs"
                        value={envDraft ?? envQuery.data ?? ""}
                        onChange={(e) => setEnvDraft(e.target.value)}
                        spellCheck={false}
                      />
                    )}
                    <div className="flex gap-2">
                      <Button size="sm" disabled={envSaveMutation.isPending} onClick={() => envSaveMutation.mutate()}>Save .env</Button>
                      <Button size="sm" variant="ghost" onClick={() => { setEnvOpen(false); setEnvDraft(null); }}>Cancel</Button>
                    </div>
                  </div>
                ) : (
                  <Button size="sm" variant="outline" onClick={() => setEnvOpen(true)}>Edit .env</Button>
                )}
              </div>
            )}

            <div className="flex flex-wrap items-center gap-2 border-t border-border pt-3">
              <Button size="sm" variant="outline" onClick={() => togglePinMutation.mutate()}>
                {deployment.pinned ? <PinOff className="size-3.5" /> : <Pin className="size-3.5" />} {deployment.pinned ? "Unpin" : "Pin to top"}
              </Button>
              <Button size="sm" variant="destructive" onClick={() => setConfirmDelete(true)}>
                <Trash2 className="size-3.5" /> Delete
              </Button>
              <div className="ml-auto flex gap-1">
                <Button size="sm" variant="outline" onClick={() => copy(deployment.path)}>Copy path</Button>
                <Button size="sm" variant="outline" onClick={onClose}><X className="size-3.5" /> Close</Button>
              </div>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={dupOpen} onOpenChange={setDupOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Duplicate deployment</DialogTitle><DialogDescription>Creates a copy of {deployment.name} with a new name.</DialogDescription></DialogHeader>
          <Input autoFocus value={newName} onChange={(e) => setNewName(e.target.value)} placeholder={`${deployment.name}-copy`} />
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setDupOpen(false)}>Cancel</Button>
            <Button disabled={!newName.trim() || dupMutation.isPending} onClick={() => dupMutation.mutate()}>{dupMutation.isPending ? "Duplicating…" : "Duplicate"}</Button>
          </div>
        </DialogContent>
      </Dialog>

      <Dialog open={restoreOpen} onOpenChange={setRestoreOpen}>
        <DialogContent>
          <DialogHeader><DialogTitle>Restore backup</DialogTitle><DialogDescription>Choose a backup to restore as a new deployment. Existing folders are never overwritten.</DialogDescription></DialogHeader>
          <ScrollArea className="max-h-64">
            <div className="space-y-2">
              {backups && backups.length > 0 ? backups.map((b) => (
                <button key={b.path} onClick={() => setSelectedBackup(b.path)}
                  className={`w-full rounded-lg border p-2 text-left ${selectedBackup === b.path ? "border-primary bg-primary/10" : "border-border hover:bg-muted"}`}>
                  <p className="text-sm font-medium">{b.timestamp}</p>
                  <p className="truncate font-mono text-xs text-muted-foreground">{(b.size / 1024).toFixed(0)} KB</p>
                </button>
              )) : <p className="py-8 text-center text-sm text-muted-foreground">No backups available.</p>}
            </div>
          </ScrollArea>
          <div className="flex justify-end gap-2">
            <Button variant="outline" onClick={() => setRestoreOpen(false)}>Cancel</Button>
            <Button disabled={!selectedBackup || restoreMutation.isPending} onClick={() => restoreMutation.mutate()}>
              {restoreMutation.isPending ? "Restoring…" : `Restore as ${deployment.name}`}
            </Button>
          </div>
        </DialogContent>
      </Dialog>

      <ConfirmActionDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        title={`Delete ${deployment.name}?`}
        description="The project folder and all of its files will be moved to the Windows Recycle Bin."
        confirmLabel="Move to Recycle Bin"
        destructive
        pending={deleteMutation.isPending}
        onConfirm={() => deleteMutation.mutate()}
      />
    </>
  );
}

export function FrameworkIcon({ framework }: { framework: string }) {
  const map: Record<string, typeof Code2> = {
    html: Globe, php: Code2, laravel: Code2, wordpress: Globe, react: RefreshCw, node: Code2, custom: FolderGit2,
  };
  const Icon = map[framework] ?? Code2;
  return <Icon className="size-3.5" />;
}
