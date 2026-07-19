import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Copy, ExternalLink, Eye, EyeOff, FolderInput, FolderOpen, Globe2, Pencil, Plus, Search, Trash2 } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useNavigate } from "react-router-dom";
import { createDeployment, deleteDeployment, importDeployment, listDeployments } from "@/lib/ipc";
import type { Deployment } from "@/lib/types";
import { PageHeader } from "@/components/layout/PageHeader";
import { Button } from "@/components/ui/button";
import { ConfirmActionDialog } from "@/components/ui/ConfirmActionDialog";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ScrollArea } from "@/components/ui/scroll-area";
import { toast } from "sonner";

type Template = "html" | "php";
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
  const [search, setSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [hiddenOpen, setHiddenOpen] = useState(false);
  const [name, setName] = useState("");
  const [template, setTemplate] = useState<Template>("html");
  const [importName, setImportName] = useState("");
  const [importSource, setImportSource] = useState("");
  const [hiddenNames, setHiddenNames] = useState(readHiddenDeployments);
  const [deploymentToDelete, setDeploymentToDelete] = useState<Deployment | null>(null);
  useEffect(() => {
    localStorage.setItem(HIDDEN_DEPLOYMENTS_KEY, JSON.stringify(hiddenNames));
    localStorage.removeItem(LEGACY_HIDDEN_DEPLOYMENTS_KEY);
  }, [hiddenNames]);
  const openWorkspace = (deployment: Deployment) => navigate("/files", { state: { deploymentPath: deployment.path, deploymentName: deployment.name } });
  const createMutation = useMutation({ mutationFn: () => createDeployment(name, template), onSuccess: (deployment) => { setCreateOpen(false); setName(""); queryClient.invalidateQueries({ queryKey: ["deployments"] }); toast.success(`${deployment.name} starter project is ready`); openWorkspace(deployment); }, onError: (reason: Error) => toast.error(reason.message) });
  const importMutation = useMutation({ mutationFn: () => importDeployment(importName, importSource), onSuccess: (deployment) => { setImportOpen(false); setImportName(""); setImportSource(""); queryClient.invalidateQueries({ queryKey: ["deployments"] }); toast.success(`${deployment.name} was imported into htdocs`); openWorkspace(deployment); }, onError: (reason: Error) => toast.error(reason.message) });
  const deleteMutation = useMutation({ mutationFn: (deployment: Deployment) => deleteDeployment(deployment.name), onSuccess: (_, deployment) => { setDeploymentToDelete(null); setHiddenNames((current) => current.filter((name) => name !== deployment.name)); queryClient.invalidateQueries({ queryKey: ["deployments"] }); toast.success("Deployment removed"); }, onError: (reason: Error) => toast.error(reason.message) });
  const allDeployments = deploymentsQuery.data ?? [];
  const hiddenDeployments = allDeployments.filter((deployment) => hiddenNames.includes(deployment.name));
  const visibleDeployments = allDeployments.filter((deployment) => !hiddenNames.includes(deployment.name));
  const deployments = useMemo(() => visibleDeployments.filter((deployment) => deployment.name.toLowerCase().includes(search.toLowerCase())), [visibleDeployments, search]);
  const hideDeployment = (name: string) => { setHiddenNames((current) => current.includes(name) ? current : [...current, name]); toast.success(`${name} is hidden from the deployments list`); };
  const unhideDeployment = (name: string) => { setHiddenNames((current) => current.filter((item) => item !== name)); toast.success(`${name} is visible again`); };
  const chooseProject = async () => {
    try {
      const source = await open({ directory: true, multiple: false, title: "Select a project folder to import" });
      if (typeof source !== "string") return;
      setImportSource(source);
      setImportName(source.split(/[\\/]/).filter(Boolean).pop() ?? "");
      setImportOpen(true);
    } catch { toast.error("Could not open the project folder picker"); }
  };

  return <div className="flex min-h-full flex-col p-6 pb-10"><PageHeader title="Deployments" description="Create, import, customize, and open local projects in C:\xampp\htdocs." actions={<div className="flex flex-wrap justify-end gap-2"><Button variant="outline" onClick={() => void chooseProject()}><FolderInput />Import project</Button><Button onClick={() => setCreateOpen(true)}><Plus />New deployment</Button></div>} />
    <div className="mb-5 flex flex-wrap items-center gap-3 rounded-xl border border-border bg-card p-3 shadow-sm"><label className="relative min-w-56 flex-1"><Search className="absolute left-3 top-2.5 size-4 text-muted-foreground" /><Input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search deployments" className="pl-9" aria-label="Search deployments" /></label><Button variant="outline" size="sm" onClick={() => setHiddenOpen(true)}><EyeOff />Hidden ({hiddenDeployments.length})</Button><span className="rounded-md bg-muted px-3 py-2 text-sm text-muted-foreground">{visibleDeployments.length} visible project(s)</span></div>
    {deploymentsQuery.isLoading ? <DeploymentsState text="Loading deployments…" /> : deploymentsQuery.error ? <DeploymentsState text="Could not load deployments." destructive /> : deployments.length === 0 ? <div className="flex flex-1 flex-col items-center justify-center rounded-xl border border-dashed border-border bg-card p-8 text-center"><FolderOpen className="mb-4 size-12 text-muted-foreground" /><h2 className="text-lg font-semibold">{visibleDeployments.length ? "No deployments match your search" : "No visible deployments"}</h2><p className="mt-2 max-w-md text-sm text-muted-foreground">Create a starter project or import an existing project folder into htdocs.</p><div className="mt-5 flex gap-2"><Button variant="outline" onClick={() => void chooseProject()}><FolderInput />Import project</Button><Button onClick={() => setCreateOpen(true)}><Plus />New deployment</Button></div></div> : <div className="grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-3">{deployments.map((deployment) => <DeploymentCard key={deployment.path} deployment={deployment} onCustomize={() => openWorkspace(deployment)} onHide={() => hideDeployment(deployment.name)} onDelete={() => setDeploymentToDelete(deployment)} />)}</div>}
    <Dialog open={createOpen} onOpenChange={setCreateOpen}><DialogContent><DialogHeader><DialogTitle>Create deployment</DialogTitle><DialogDescription>Creates an editable starter project inside C:\xampp\htdocs, then opens its workspace.</DialogDescription></DialogHeader><label className="grid gap-2 text-sm"><span className="font-medium">Project name</span><Input autoFocus value={name} onChange={(event) => setName(event.target.value)} placeholder="my-project" onKeyDown={(event) => event.key === "Enter" && createMutation.mutate()} /><span className="text-xs text-muted-foreground">Letters, numbers, hyphens, and underscores only.</span></label><label className="grid gap-2 text-sm"><span className="font-medium">Starter template</span><Select value={template} onValueChange={(value) => setTemplate(value as Template)}><SelectTrigger><SelectValue /></SelectTrigger><SelectContent><SelectItem value="html">HTML website</SelectItem><SelectItem value="php">PHP website</SelectItem></SelectContent></Select><span className="text-xs text-muted-foreground">Includes an index page, CSS, JavaScript, and .gitignore.</span></label><DialogFooter><Button variant="outline" onClick={() => setCreateOpen(false)} disabled={createMutation.isPending}>Cancel</Button><Button onClick={() => createMutation.mutate()} disabled={createMutation.isPending || !name.trim()}>{createMutation.isPending ? "Creating…" : "Create and customize"}</Button></DialogFooter></DialogContent></Dialog>
    <Dialog open={importOpen} onOpenChange={(isOpen) => { setImportOpen(isOpen); if (!isOpen) setImportSource(""); }}><DialogContent><DialogHeader><DialogTitle>Import project</DialogTitle><DialogDescription>Copies the selected folder and its files into C:\xampp\htdocs. The original project is not changed.</DialogDescription></DialogHeader><div className="grid gap-2 text-sm"><span className="font-medium">Selected folder</span><p className="truncate rounded-md bg-muted px-3 py-2 font-mono text-xs text-muted-foreground" title={importSource}>{importSource}</p></div><label className="grid gap-2 text-sm"><span className="font-medium">Deployment name</span><Input autoFocus value={importName} onChange={(event) => setImportName(event.target.value)} placeholder="my-project" onKeyDown={(event) => event.key === "Enter" && importMutation.mutate()} /><span className="text-xs text-muted-foreground">Letters, numbers, hyphens, and underscores only. Existing folders are never overwritten.</span></label><DialogFooter><Button variant="outline" onClick={() => setImportOpen(false)} disabled={importMutation.isPending}>Cancel</Button><Button onClick={() => importMutation.mutate()} disabled={importMutation.isPending || !importName.trim() || !importSource}>{importMutation.isPending ? "Importing…" : "Import and customize"}</Button></DialogFooter></DialogContent></Dialog>
    <Dialog open={hiddenOpen} onOpenChange={setHiddenOpen}><DialogContent><DialogHeader><DialogTitle>Hidden deployments</DialogTitle><DialogDescription>Hidden folders remain untouched in htdocs and can be shown again at any time.</DialogDescription></DialogHeader><ScrollArea className="max-h-72"><div className="space-y-2 pr-2">{hiddenDeployments.length ? hiddenDeployments.map((deployment) => <div key={deployment.path} className="flex items-center justify-between gap-3 rounded-lg border border-border p-3"><div className="min-w-0"><p className="truncate text-sm font-medium">{deployment.name}</p><p className="truncate font-mono text-xs text-muted-foreground">{deployment.url}</p></div><Button size="sm" variant="outline" onClick={() => unhideDeployment(deployment.name)}><Eye />Unhide</Button></div>) : <p className="py-8 text-center text-sm text-muted-foreground">No deployments are hidden.</p>}</div></ScrollArea><DialogFooter><Button onClick={() => setHiddenOpen(false)}>Done</Button></DialogFooter></DialogContent></Dialog>
    <ConfirmActionDialog open={!!deploymentToDelete} onOpenChange={(isOpen) => !isOpen && setDeploymentToDelete(null)} title={`Delete ${deploymentToDelete?.name ?? "deployment"}?`} description="This permanently removes the project folder and all of its files from htdocs." confirmLabel="Delete deployment" destructive pending={deleteMutation.isPending} onConfirm={() => deploymentToDelete && deleteMutation.mutate(deploymentToDelete)} />
  </div>;
}

function DeploymentCard({ deployment, onCustomize, onHide, onDelete }: { deployment: Deployment; onCustomize: () => void; onHide: () => void; onDelete: () => void }) {
  const openSite = async () => { try { await openUrl(deployment.url); } catch { toast.error("Could not open your default browser. Copy the URL and open it manually."); } };
  const copyUrl = () => { navigator.clipboard.writeText(deployment.url); toast.success("Deployment URL copied"); };
  return <article className="group flex min-h-60 flex-col overflow-hidden rounded-xl border border-border bg-card shadow-sm transition-all hover:-translate-y-0.5 hover:border-primary/30 hover:shadow-md"><div className="flex items-start gap-3 p-4 pb-3"><span className="flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary"><Globe2 className="size-5" /></span><div className="min-w-0 flex-1"><div className="flex items-start justify-between gap-2"><h2 className="truncate font-semibold">{deployment.name}</h2><span className="shrink-0 rounded-full bg-emerald-500/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-emerald-700 dark:text-emerald-400">Local</span></div><p className="mt-1 text-xs text-muted-foreground">htdocs project · Modified {deployment.modified}</p></div></div><div className="mx-4 rounded-lg border border-border bg-muted/40 px-3 py-2"><p className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">Project URL</p><button onClick={openSite} className="mt-1 block w-full truncate text-left font-mono text-xs text-foreground hover:text-primary" title={`Open ${deployment.url}`}>{deployment.url}</button></div><div className="mt-auto space-y-2 p-4"><div className="grid grid-cols-2 gap-2"><Button size="sm" onClick={openSite} className="w-full"><ExternalLink />Open site</Button><Button variant="outline" size="sm" onClick={onCustomize} className="w-full"><Pencil />Customize</Button></div><div className="flex items-center justify-between border-t border-border pt-2"><Button variant="ghost" size="sm" onClick={copyUrl}><Copy />Copy URL</Button><div className="flex items-center gap-1"><Button variant="ghost" size="icon-sm" onClick={onHide} aria-label={`Hide ${deployment.name}`} title="Hide deployment"><EyeOff /></Button><Button variant="ghost" size="icon-sm" onClick={onDelete} aria-label={`Delete ${deployment.name}`} title="Delete deployment"><Trash2 className="text-destructive" /></Button></div></div></div></article>;
}
function DeploymentsState({ text, destructive = false }: { text: string; destructive?: boolean }) { return <div className={`flex flex-1 items-center justify-center rounded-xl border border-border bg-card p-6 text-sm ${destructive ? "text-destructive" : "text-muted-foreground"}`}>{text}</div>; }
