import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Copy, Database, ExternalLink, Eye, EyeOff, FolderGit2, FolderInput, FolderOpen, Globe2, Pencil, Pin, Plus, Search, Trash2 } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useNavigate } from "react-router-dom";
import { createDeployment, deleteDeployment, getFrameworks, gitImportDeployment, importDeployment, listDeployments, updateDeploymentMeta } from "@/lib/ipc";
import type { Deployment } from "@/lib/types";
import { PageHeader } from "@/components/layout/PageHeader";
import { Button } from "@/components/ui/button";
import { ConfirmActionDialog } from "@/components/ui/ConfirmActionDialog";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ServiceStrip } from "@/components/dashboard/ServiceStrip";
import { DeploymentDetail, FrameworkIcon } from "@/components/dashboard/DeploymentDetail";
import { useHotkeys } from "@/hooks/use-hotkeys";
import { toast } from "sonner";

const HIDDEN_DEPLOYMENTS_KEY = "xamppify.hidden-deployments";
const LEGACY_HIDDEN_DEPLOYMENTS_KEY = "xampp-lan-manager.hidden-deployments";

function readHiddenDeployments() {
  try {
    const source = localStorage.getItem(HIDDEN_DEPLOYMENTS_KEY) ?? localStorage.getItem(LEGACY_HIDDEN_DEPLOYMENTS_KEY) ?? "[]";
    const saved: unknown = JSON.parse(source);
    return Array.isArray(saved) ? saved.filter((name): name is string => typeof name === "string") : [];
  } catch { return []; }
}

export default function Dashboard() {
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const deploymentsQuery = useQuery({ queryKey: ["deployments"], queryFn: listDeployments });
  const frameworksQuery = useQuery({ queryKey: ["frameworks"], queryFn: getFrameworks });
  const frameworks = frameworksQuery.data ?? [];
  const [search, setSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [hiddenOpen, setHiddenOpen] = useState(false);
  const [name, setName] = useState("");
  const [framework, setFramework] = useState("html");
  const [importName, setImportName] = useState("");
  const [importSource, setImportSource] = useState("");
  const [importFramework, setImportFramework] = useState<string | null>(null);
  const [importTab, setImportTab] = useState<"folder" | "git">("folder");
  const [gitUrl, setGitUrl] = useState("");
  const [hiddenNames, setHiddenNames] = useState(readHiddenDeployments);
  const [deploymentToDelete, setDeploymentToDelete] = useState<Deployment | null>(null);
  const [detailDeployment, setDetailDeployment] = useState<Deployment | null>(null);
  const [sortBy, setSortBy] = useState<"name" | "modified" | "framework">("name");
  const [tagFilter, setTagFilter] = useState("");

  useEffect(() => {
    localStorage.setItem(HIDDEN_DEPLOYMENTS_KEY, JSON.stringify(hiddenNames));
    localStorage.removeItem(LEGACY_HIDDEN_DEPLOYMENTS_KEY);
  }, [hiddenNames]);

  const isMac = navigator.platform.toUpperCase().includes("MAC");
  useHotkeys([
    { key: "n", ctrl: !isMac, meta: isMac, shift: true, handler: () => setCreateOpen(true) },
    { key: "o", ctrl: !isMac, meta: isMac, shift: true, handler: () => void chooseProject() },
  ]);

  const openWorkspace = (deployment: Deployment) => navigate("/files", { state: { deploymentPath: deployment.path, deploymentName: deployment.name } });
  const openDetail = (deployment: Deployment) => setDetailDeployment(deployment);
  const invalidate = () => queryClient.invalidateQueries({ queryKey: ["deployments"] });

  const createMutation = useMutation({
    mutationFn: () => createDeployment(name, framework),
    onSuccess: (deployment) => { setCreateOpen(false); setName(""); invalidate(); toast.success(`${deployment.name} starter project is ready`); openDetail(deployment); },
    onError: (reason: Error) => toast.error(reason.message),
  });
  const importMutation = useMutation({
    mutationFn: () => importDeployment(importName, importSource, importFramework ?? undefined),
    onSuccess: (deployment) => { setImportOpen(false); setImportName(""); setImportSource(""); invalidate(); toast.success(`${deployment.name} was imported into htdocs`); openDetail(deployment); },
    onError: (reason: Error) => toast.error(reason.message),
  });
  const gitMutation = useMutation({
    mutationFn: () => gitImportDeployment(importName, gitUrl, importFramework ?? undefined),
    onSuccess: (deployment) => { setImportOpen(false); setImportName(""); setGitUrl(""); invalidate(); toast.success(`${deployment.name} was cloned into htdocs`); openDetail(deployment); },
    onError: (reason: Error) => toast.error(reason.message),
  });
  const deleteMutation = useMutation({
    mutationFn: (deployment: Deployment) => deleteDeployment(deployment.name),
    onSuccess: (_, deployment) => { setDeploymentToDelete(null); setHiddenNames((current) => current.filter((n) => n !== deployment.name)); invalidate(); toast.success("Deployment moved to the Recycle Bin"); },
    onError: (reason: Error) => toast.error(reason.message),
  });
  const pinMutation = useMutation({
    mutationFn: (deployment: Deployment) => updateDeploymentMeta(deployment.name, { pinned: !deployment.pinned }),
    onSuccess: () => invalidate(),
    onError: (reason: Error) => toast.error(reason.message),
  });

  const allDeployments = deploymentsQuery.data ?? [];
  const hiddenDeployments = allDeployments.filter((d) => hiddenNames.includes(d.name));
  const visibleDeployments = allDeployments.filter((d) => !hiddenNames.includes(d.name));
  const deployments = useMemo(() => {
    let filtered = visibleDeployments.filter((d) => d.name.toLowerCase().includes(search.toLowerCase()));
    if (tagFilter) filtered = filtered.filter((d) => d.tags.includes(tagFilter));
    const sorted = [...filtered];
    if (sortBy === "name") sorted.sort((a, b) => a.name.localeCompare(b.name));
    if (sortBy === "modified") sorted.sort((a, b) => b.modified.localeCompare(a.modified));
    if (sortBy === "framework") sorted.sort((a, b) => a.framework.localeCompare(b.framework));
    return sorted;
  }, [visibleDeployments, search, tagFilter, sortBy]);

  const allTags = useMemo(() => {
    const set = new Set<string>();
    visibleDeployments.forEach((d) => d.tags.forEach((t) => set.add(t)));
    return [...set];
  }, [visibleDeployments]);

  const hideDeployment = (name: string) => { setHiddenNames((c) => c.includes(name) ? c : [...c, name]); toast.success(`${name} is hidden`); };
  const unhideDeployment = (name: string) => { setHiddenNames((c) => c.filter((i) => i !== name)); toast.success(`${name} is visible again`); };
  const chooseProject = async () => {
    try {
      const source = await open({ directory: true, multiple: false, title: "Select a project folder to import" });
      if (typeof source !== "string") return;
      setImportSource(source);
      setImportName(source.split(/[\\/]/).filter(Boolean).pop() ?? "");
      setImportFramework(null);
      setImportTab("folder");
      setImportOpen(true);
    } catch { toast.error("Could not open the project folder picker"); }
  };
  const openGitImport = () => { setImportName(""); setGitUrl(""); setImportFramework(null); setImportTab("git"); setImportOpen(true); };

  return <div className="flex min-h-full flex-col p-6 pb-10"><PageHeader title="Deployments" description="Create, import, customize, and open local projects in C:\xampp\htdocs." actions={<div className="flex flex-wrap justify-end gap-2"><Button variant="outline" onClick={() => void chooseProject()}><FolderInput />Import project</Button><Button variant="outline" onClick={openGitImport}><FolderGit2 />From Git</Button><Button onClick={() => setCreateOpen(true)}><Plus />New deployment</Button></div>} />
    <ServiceStrip />
    <div className="mb-5 flex flex-wrap items-center gap-3 rounded-xl border border-border bg-card p-3 shadow-sm">
      <label className="relative min-w-0 flex-1"><Search className="absolute left-3 top-2.5 size-4 text-muted-foreground" /><Input value={search} onChange={(e) => setSearch(e.target.value)} placeholder="Search deployments" className="pl-9" aria-label="Search deployments" /></label>
      <Select value={sortBy} onValueChange={(v) => setSortBy(v as typeof sortBy)}><SelectTrigger className="h-9 text-xs"><SelectValue /></SelectTrigger><SelectContent><SelectItem value="name">Sort: Name</SelectItem><SelectItem value="modified">Sort: Modified</SelectItem><SelectItem value="framework">Sort: Framework</SelectItem></SelectContent></Select>
      {allTags.length > 0 && (
        <Select value={tagFilter} onValueChange={setTagFilter}><SelectTrigger className="h-9 text-xs"><SelectValue placeholder="All tags" /></SelectTrigger><SelectContent><SelectItem value="">All tags</SelectItem>{allTags.map((tag) => <SelectItem key={tag} value={tag}>{tag}</SelectItem>)}</SelectContent></Select>
      )}
      <Button variant="outline" size="sm" onClick={() => setHiddenOpen(true)}><EyeOff />Hidden ({hiddenDeployments.length})</Button>
      <span className="rounded-md bg-muted px-3 py-2 text-sm text-muted-foreground shrink-0">{visibleDeployments.length} visible project(s)</span>
    </div>
    {deploymentsQuery.isLoading ? <DeploymentsState text="Loading deployments…" /> : deploymentsQuery.error ? <DeploymentsState text="Could not load deployments." destructive /> : deployments.length === 0 ? <div className="flex flex-1 flex-col items-center justify-center rounded-xl border border-dashed border-border bg-card p-8 text-center"><FolderOpen className="mb-4 size-12 text-muted-foreground" /><h2 className="text-lg font-semibold">{visibleDeployments.length ? "No deployments match your filters" : "No visible deployments"}</h2><p className="mt-2 max-w-md text-sm text-muted-foreground">Create a starter project, import an existing folder, or clone from Git.</p><div className="mt-5 flex gap-2"><Button variant="outline" onClick={() => void chooseProject()}><FolderInput />Import project</Button><Button onClick={() => setCreateOpen(true)}><Plus />New deployment</Button></div></div> : <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">{deployments.map((deployment) => <DeploymentCard key={deployment.path} deployment={deployment} onCustomize={() => openWorkspace(deployment)} onManage={() => openDetail(deployment)} onHide={() => hideDeployment(deployment.name)} onDelete={() => setDeploymentToDelete(deployment)} onTogglePin={() => pinMutation.mutate(deployment)} />)}</div>}

    <Dialog open={createOpen} onOpenChange={setCreateOpen}><DialogContent><DialogHeader><DialogTitle>Create deployment</DialogTitle><DialogDescription>Creates an editable starter project inside C:\xampp\htdocs, then opens its workspace.</DialogDescription></DialogHeader>
      <label className="grid gap-2 text-sm"><span className="font-medium">Project name</span><Input autoFocus value={name} onChange={(e) => setName(e.target.value)} placeholder="my-project" onKeyDown={(e) => e.key === "Enter" && createMutation.mutate()} /><span className="text-xs text-muted-foreground">Letters, numbers, hyphens, and underscores only.</span></label>
      <label className="grid gap-2 text-sm"><span className="font-medium">Framework</span><Select value={framework} onValueChange={setFramework}><SelectTrigger><SelectValue /></SelectTrigger><SelectContent>{frameworks.map((f) => <SelectItem key={f.id} value={f.id}>{f.label}</SelectItem>)}</SelectContent></Select><span className="text-xs text-muted-foreground">The starter template, URL, and tooling adapt to the framework you pick.</span></label>
      <DialogFooter><Button variant="outline" onClick={() => setCreateOpen(false)} disabled={createMutation.isPending}>Cancel</Button><Button onClick={() => createMutation.mutate()} disabled={createMutation.isPending || !name.trim()}>{createMutation.isPending ? "Creating…" : "Create and customize"}</Button></DialogFooter>
    </DialogContent></Dialog>

    <Dialog open={importOpen} onOpenChange={(isOpen) => { setImportOpen(isOpen); if (!isOpen) setImportSource(""); }}>
      <DialogContent>
        <DialogHeader><DialogTitle>Import project</DialogTitle><DialogDescription>Bring an existing project into htdocs. The original is never changed.</DialogDescription></DialogHeader>
        <div className="flex gap-1">
          <Button size="sm" variant={importTab === "folder" ? "secondary" : "ghost"} onClick={() => setImportTab("folder")}><FolderInput />From folder</Button>
          <Button size="sm" variant={importTab === "git" ? "secondary" : "ghost"} onClick={() => setImportTab("git")}><FolderGit2 />From Git</Button>
        </div>
        {importTab === "folder" ? (
          <div className="grid gap-3">
            <div className="grid gap-2 text-sm"><span className="font-medium">Selected folder</span><p className="truncate rounded-md bg-muted px-3 py-2 font-mono text-xs text-muted-foreground" title={importSource}>{importSource || "No folder selected"}</p>
              <div className="flex gap-2"><Button size="sm" variant="outline" onClick={() => void chooseProject()}>Choose folder</Button></div>
            </div>
            <label className="grid gap-2 text-sm"><span className="font-medium">Deployment name</span><Input value={importName} onChange={(e) => setImportName(e.target.value)} placeholder="my-project" /></label>
            <label className="grid gap-2 text-sm"><span className="font-medium">Framework (auto-detected if left as Auto)</span>
              <Select value={importFramework ?? ""} onValueChange={(v) => setImportFramework(v || null)}><SelectTrigger><SelectValue placeholder="Auto-detect" /></SelectTrigger><SelectContent><SelectItem value="">Auto-detect</SelectItem>{frameworks.map((f) => <SelectItem key={f.id} value={f.id}>{f.label}</SelectItem>)}</SelectContent></Select>
            </label>
            <DialogFooter><Button variant="outline" onClick={() => setImportOpen(false)} disabled={importMutation.isPending}>Cancel</Button><Button onClick={() => importMutation.mutate()} disabled={importMutation.isPending || !importName.trim() || !importSource}>{importMutation.isPending ? "Importing…" : "Import and customize"}</Button></DialogFooter>
          </div>
        ) : (
          <div className="grid gap-3">
            <label className="grid gap-2 text-sm"><span className="font-medium">Git repository URL</span><Input value={gitUrl} onChange={(e) => setGitUrl(e.target.value)} placeholder="https://github.com/user/repo.git" /></label>
            <label className="grid gap-2 text-sm"><span className="font-medium">Deployment name</span><Input value={importName} onChange={(e) => setImportName(e.target.value)} placeholder="my-project" /></label>
            <label className="grid gap-2 text-sm"><span className="font-medium">Framework (optional)</span>
              <Select value={importFramework ?? ""} onValueChange={(v) => setImportFramework(v || null)}><SelectTrigger><SelectValue placeholder="Auto-detect" /></SelectTrigger><SelectContent><SelectItem value="">Auto-detect</SelectItem>{frameworks.map((f) => <SelectItem key={f.id} value={f.id}>{f.label}</SelectItem>)}</SelectContent></Select>
            </label>
            <DialogFooter><Button variant="outline" onClick={() => setImportOpen(false)} disabled={gitMutation.isPending}>Cancel</Button><Button onClick={() => gitMutation.mutate()} disabled={gitMutation.isPending || !importName.trim() || !gitUrl.trim()}>{gitMutation.isPending ? "Cloning…" : "Clone and customize"}</Button></DialogFooter>
          </div>
        )}
      </DialogContent>
    </Dialog>

    <Dialog open={hiddenOpen} onOpenChange={setHiddenOpen}><DialogContent><DialogHeader><DialogTitle>Hidden deployments</DialogTitle><DialogDescription>Hidden folders remain untouched in htdocs and can be shown again at any time.</DialogDescription></DialogHeader><ScrollArea className="max-h-72"><div className="space-y-2 pr-2">{hiddenDeployments.length ? hiddenDeployments.map((deployment) => <div key={deployment.path} className="flex items-center justify-between gap-3 rounded-lg border border-border p-3"><div className="min-w-0"><p className="truncate text-sm font-medium">{deployment.name}</p><p className="truncate font-mono text-xs text-muted-foreground">{deployment.url}</p></div><Button size="sm" variant="outline" onClick={() => unhideDeployment(deployment.name)}><Eye />Unhide</Button></div>) : <p className="py-8 text-center text-sm text-muted-foreground">No deployments are hidden.</p>}</div></ScrollArea><DialogFooter><Button onClick={() => setHiddenOpen(false)}>Done</Button></DialogFooter></DialogContent></Dialog>

    <ConfirmActionDialog open={!!deploymentToDelete} onOpenChange={(isOpen) => !isOpen && setDeploymentToDelete(null)} title={`Delete ${deploymentToDelete?.name ?? "deployment"}?`} description="The project folder and all of its files will be moved to the Windows Recycle Bin." confirmLabel="Move to Recycle Bin" destructive pending={deleteMutation.isPending} onConfirm={() => deploymentToDelete && deleteMutation.mutate(deploymentToDelete)} />
    <DeploymentDetail deployment={detailDeployment} onClose={() => setDetailDeployment(null)} onChanged={invalidate} />
  </div>;
}

function DeploymentCard({ deployment, onCustomize, onManage, onHide, onDelete, onTogglePin }: { deployment: Deployment; onCustomize: () => void; onManage: () => void; onHide: () => void; onDelete: () => void; onTogglePin: () => void }) {
  const openSite = async () => { try { await openUrl(deployment.url); } catch { toast.error("Could not open your default browser. Copy the URL and open it manually."); } };
  const openNetwork = async () => { try { await openUrl(deployment.network_url); } catch { toast.error("Could not open your default browser. Copy the URL and open it manually."); } };
  const copyUrl = (url: string) => { navigator.clipboard.writeText(url); toast.success("URL copied"); };
  return <article className="group relative flex min-h-60 flex-col overflow-hidden rounded-xl border border-border bg-card shadow-sm transition-all hover:-translate-y-0.5 hover:border-primary/30 hover:shadow-md" onContextMenu={(e) => { e.preventDefault(); onManage(); }}>
    <div className="flex items-start gap-3 p-4 pb-3">
      <button onClick={onManage} className="flex min-w-0 flex-1 items-start gap-3 text-left">
        <span className="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary"><Globe2 className="size-5" /></span>
        <span className="min-w-0 flex-1">
          <span className="flex items-start justify-between gap-2">
            <span className="truncate font-semibold">{deployment.name}</span>
            {deployment.ssl_enabled && <span className="shrink-0 text-[10px] text-emerald-500" title="HTTPS enabled">🔒</span>}
          </span>
          <span className="mt-1 flex flex-wrap items-center gap-1 text-xs text-muted-foreground">
            <span className="rounded bg-muted px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide"><FrameworkIcon framework={deployment.framework} /> {deployment.framework}</span>
            <span>Modified {deployment.modified}</span>
            {deployment.linked_db && <span title={`Database: ${deployment.linked_db}`}>· <Database className="inline size-3" />{deployment.linked_db}</span>}
          </span>
        </span>
      </button>
    </div>
    <div className="mx-4 space-y-1.5 rounded-lg border border-border bg-muted/40 px-3 py-2">
      <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">Local URL</p>
      <button onClick={openSite} className="block w-full truncate text-left font-mono text-xs text-foreground hover:text-primary" title={`Open ${deployment.url}`}>{deployment.url}</button>
      <p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">Network URL</p>
      <button onClick={openNetwork} className="block w-full truncate text-left font-mono text-xs text-foreground hover:text-primary" title={`Open ${deployment.network_url}`}>{deployment.network_url}</button>
    </div>
    <div className="mt-auto space-y-2 p-4">
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
        <Button size="sm" onClick={openSite} className="w-full"><ExternalLink />Open site</Button>
        <Button variant="outline" size="sm" onClick={onCustomize} className="w-full"><Pencil />Customize</Button>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-1 border-t border-border pt-2">
        <div className="flex gap-1">
          <Button variant="ghost" size="sm" onClick={() => copyUrl(deployment.url)}><Copy />Local</Button>
          <Button variant="ghost" size="sm" onClick={() => copyUrl(deployment.network_url)}><Copy />Network</Button>
        </div>
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="icon-sm" onClick={onTogglePin} aria-label={deployment.pinned ? "Unpin" : "Pin"} title={deployment.pinned ? "Unpin deployment" : "Pin deployment"}><Pin className={deployment.pinned ? "text-primary" : ""} /></Button>
          <Button variant="ghost" size="icon-sm" onClick={onHide} aria-label={`Hide ${deployment.name}`} title="Hide deployment"><EyeOff /></Button>
          <Button variant="ghost" size="icon-sm" onClick={onDelete} aria-label={`Delete ${deployment.name}`} title="Delete deployment"><Trash2 className="text-destructive" /></Button>
        </div>
      </div>
    </div>
  </article>;
}
function DeploymentsState({ text, destructive = false }: { text: string; destructive?: boolean }) { return <div className={`flex flex-1 items-center justify-center rounded-xl border border-border bg-card p-6 text-sm ${destructive ? "text-destructive" : "text-muted-foreground"}`}>{text}</div>; }
