import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { FileCog, FlaskConical, Save, Search, WrapText } from "lucide-react";
import { getKnownConfigs, parseIniSections, readFile, saveConfigFile, testApacheConfig } from "@/lib/ipc";
import { PageHeader } from "@/components/layout/PageHeader";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ConfirmActionDialog } from "@/components/ui/ConfirmActionDialog";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import { toast } from "sonner";

type Section = { name: string; line: number };

export default function ConfigEditor() {
  const queryClient = useQueryClient();
  const configsQuery = useQuery({ queryKey: ["known-configs"], queryFn: getKnownConfigs });
  const [selected, setSelected] = useState<string | null>(null);
  const [content, setContent] = useState("");
  const [sections, setSections] = useState<Section[]>([]);
  const [confirmSave, setConfirmSave] = useState(false);
  const [findTerm, setFindTerm] = useState("");
  const [wrapLines, setWrapLines] = useState(false);
  const [cursor, setCursor] = useState({ line: 1, column: 1 });
  const editorRef = useRef<HTMLTextAreaElement>(null);
  const linesRef = useRef<HTMLPreElement>(null);
  const selectedConfig = configsQuery.data?.find((file) => file.path === selected);
  const fileQuery = useQuery({ queryKey: ["config-content", selected], queryFn: () => readFile(selected!), enabled: !!selected });

  useEffect(() => {
    if (fileQuery.data === undefined) return;
    setContent(fileQuery.data);
    setCursor({ line: 1, column: 1 });
    setFindTerm("");
    if (selected?.endsWith(".ini")) parseIniSections(fileQuery.data).then(setSections); else setSections([]);
  }, [fileQuery.data, selected]);

  const saveMutation = useMutation({
    mutationFn: () => saveConfigFile(selected!, content),
    onSuccess: (backupPath) => {
      queryClient.setQueryData(["config-content", selected], content);
      toast.success("Configuration saved", { description: `Previous version backed up to ${backupPath}` });
    },
    onError: (reason: Error) => toast.error(reason.message),
  });
  const testMutation = useMutation({
    mutationFn: testApacheConfig,
    onSuccess: (result) => {
      setTestResult(result);
      if (result.ok) toast.success("Apache configuration is valid");
    },
    onError: (reason: Error) => toast.error(reason.message),
  });
  const [testResult, setTestResult] = useState<{ ok: boolean; output: string } | null>(null);
  const grouped = useMemo(() => configsQuery.data?.reduce<Record<string, NonNullable<typeof configsQuery.data>>>((acc, config) => { (acc[config.category] ??= []).push(config); return acc; }, {}), [configsQuery.data]);
  const hasUnsavedChanges = fileQuery.data !== undefined && content !== fileQuery.data;
  const lineNumbers = useMemo(() => Array.from({ length: Math.max(1, content.split("\n").length) }, (_, index) => index + 1).join("\n"), [content]);

  const updateCursor = () => {
    const editor = editorRef.current;
    if (!editor) return;
    const beforeCursor = editor.value.slice(0, editor.selectionStart);
    const line = beforeCursor.split("\n").length;
    const column = beforeCursor.length - beforeCursor.lastIndexOf("\n");
    setCursor({ line, column });
  };
  const syncLineNumbers = () => { if (editorRef.current && linesRef.current) linesRef.current.scrollTop = editorRef.current.scrollTop; };
  const selectFile = (path: string) => { if (hasUnsavedChanges && !window.confirm("Discard unsaved configuration changes?")) return; setSelected(path); };
  const focusOffset = useCallback((offset: number) => {
    const editor = editorRef.current;
    if (!editor) return;
    editor.focus(); editor.setSelectionRange(offset, offset);
    const lineHeight = Number.parseFloat(getComputedStyle(editor).lineHeight) || 22;
    const line = content.slice(0, offset).split("\n").length;
    editor.scrollTop = Math.max(0, (line - 3) * lineHeight);
    updateCursor();
  }, [content]);
  const findNext = () => {
    const term = findTerm.trim();
    if (!term) return;
    const editor = editorRef.current;
    const from = editor?.selectionEnd ?? 0;
    const match = content.toLowerCase().indexOf(term.toLowerCase(), from);
    const offset = match === -1 ? content.toLowerCase().indexOf(term.toLowerCase()) : match;
    if (offset === -1) { toast.message("No matches found"); return; }
    focusOffset(offset);
    editorRef.current?.setSelectionRange(offset, offset + term.length);
    updateCursor();
  };
  const scrollToLine = (line: number) => focusOffset(content.split("\n").slice(0, line - 1).join("\n").length + (line > 1 ? 1 : 0));

  return <div className="flex h-full min-w-0 flex-col p-6">
    <PageHeader title="Config Editor" description="Review and update known local XAMPP configuration files." actions={hasUnsavedChanges ? <span className="rounded-md bg-amber-500/10 px-2.5 py-1.5 text-xs font-medium text-amber-700 dark:text-amber-400">Unsaved changes</span> : <span className="text-xs text-muted-foreground">All changes saved</span>} />
    <div className="grid min-h-0 min-w-0 flex-1 grid-cols-1 gap-4 lg:grid-cols-[18rem_minmax(0,1fr)]">
      <section className="flex min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-card shadow-sm"><div className="border-b border-border bg-muted/30 px-4 py-3"><p className="font-medium">Configuration files</p><p className="text-xs text-muted-foreground">Choose a file to open it in the editor.</p></div><ScrollArea className="min-h-0 flex-1"><div className="p-2">{Object.entries(grouped ?? {}).map(([category, files]) => <div key={category} className="mb-3"><p className="px-2 py-1 text-xs font-semibold uppercase tracking-wider text-muted-foreground">{category}</p>{files.map((file) => <button key={file.path} onClick={() => selectFile(file.path)} className={`flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-left text-sm ${selected === file.path ? "bg-primary text-primary-foreground" : "hover:bg-muted"}`}><FileCog className="size-4 shrink-0" /><span className="truncate">{file.name}</span></button>)}</div>)}</div></ScrollArea></section>
      {selectedConfig ? <section className={`grid min-h-0 min-w-0 gap-4 ${sections.length ? "xl:grid-cols-[minmax(0,1fr)_13rem]" : "grid-cols-1"}`}>
        <div className="flex min-h-0 min-w-0 flex-col overflow-hidden rounded-xl border border-border bg-card shadow-sm">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-border bg-muted/30 px-4 py-3"><div className="min-w-0"><p className="font-medium">{selectedConfig.name}</p><p className="truncate font-mono text-xs text-muted-foreground" title={selectedConfig.path}>{selectedConfig.path}</p></div><div className="flex items-center gap-2">{selectedConfig.category === "Apache" && <Button size="sm" variant="outline" onClick={() => testMutation.mutate()} disabled={testMutation.isPending}><FlaskConical />{testMutation.isPending ? "Testing…" : "Test config"}</Button>}<Button size="sm" onClick={() => setConfirmSave(true)} disabled={!hasUnsavedChanges || saveMutation.isPending}><Save />{saveMutation.isPending ? "Saving…" : "Save"}</Button></div></div>
          <div className="flex flex-wrap items-center gap-2 border-b border-border px-3 py-2"><div className="relative min-w-48 flex-1"><Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" /><Input className="pl-8" value={findTerm} onChange={(event) => setFindTerm(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") findNext(); }} placeholder="Find in file" aria-label="Find in configuration file" /></div><Button size="sm" variant="outline" onClick={findNext} disabled={!findTerm.trim()}>Find</Button><Button size="sm" variant={wrapLines ? "secondary" : "outline"} onClick={() => setWrapLines(!wrapLines)} title="Toggle line wrapping" aria-pressed={wrapLines}><WrapText />Wrap</Button></div>
          {fileQuery.isLoading ? <EditorState text="Loading configuration…" /> : fileQuery.error ? <EditorState text="Unable to load this configuration file." destructive /> : <div className="relative min-h-0 flex-1 overflow-hidden bg-background"><pre ref={linesRef} aria-hidden className="pointer-events-none absolute inset-y-0 left-0 w-12 overflow-hidden border-r border-border bg-muted/30 py-3 pr-2 text-right font-mono text-xs leading-[1.375rem] text-muted-foreground">{lineNumbers}</pre><textarea ref={editorRef} id="config-editor" className={`h-full w-full resize-none bg-transparent py-3 pl-16 pr-4 font-mono text-xs leading-[1.375rem] text-foreground outline-none ${wrapLines ? "whitespace-pre-wrap break-words" : "whitespace-pre"}`} value={content} onChange={(event) => setContent(event.target.value)} onScroll={syncLineNumbers} onSelect={updateCursor} onKeyUp={updateCursor} onClick={updateCursor} spellCheck={false} wrap={wrapLines ? "soft" : "off"} /></div>}
          <div className="flex items-center justify-between border-t border-border bg-muted/20 px-4 py-1.5 text-xs text-muted-foreground"><span>Ln {cursor.line}, Col {cursor.column}</span><span>{content.split("\n").length} lines · {content.length.toLocaleString()} characters</span></div>
        </div>
        {sections.length > 0 && <aside className="flex min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-card shadow-sm"><div className="border-b border-border bg-muted/30 px-4 py-3"><p className="font-medium">Sections</p><p className="text-xs text-muted-foreground">Jump to a section</p></div><ScrollArea className="min-h-0 flex-1"><div className="p-2">{sections.map((section) => <button key={`${section.name}-${section.line}`} onClick={() => scrollToLine(section.line)} className="w-full rounded-lg px-2.5 py-2 text-left text-sm text-muted-foreground hover:bg-muted hover:text-foreground"><span className="block truncate">{section.name}</span><span className="mt-0.5 block text-xs opacity-70">Line {section.line}</span></button>)}</div></ScrollArea></aside>}
      </section> : <section className="flex min-h-0 flex-col items-center justify-center rounded-xl border border-dashed border-border bg-card p-6 text-center"><FileCog className="mb-3 size-10 text-muted-foreground" /><h2 className="font-medium">Select a configuration file</h2><p className="mt-1 max-w-sm text-sm text-muted-foreground">Choose a file from the list to inspect and edit its contents.</p></section>}
    </div>
    <ConfirmActionDialog open={confirmSave} onOpenChange={setConfirmSave} title="Save configuration changes?" description={`This will overwrite ${selectedConfig?.name ?? "the selected file"}. The current version is backed up automatically before saving.`} confirmLabel="Save configuration" pending={saveMutation.isPending} onConfirm={() => { saveMutation.mutate(); setConfirmSave(false); }} />
    <Dialog open={testResult !== null} onOpenChange={(isOpen) => !isOpen && setTestResult(null)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Apache configuration test</DialogTitle>
          <DialogDescription>{testResult?.ok ? "httpd -t completed without errors. The configuration is valid." : "httpd -t reported configuration problems."}</DialogDescription>
        </DialogHeader>
        {testResult?.output ? <pre className={`max-h-64 overflow-auto rounded-md border border-border bg-muted/40 p-3 font-mono text-xs whitespace-pre-wrap ${testResult.ok ? "text-emerald-600 dark:text-emerald-400" : "text-destructive"}`}>{testResult.output}</pre> : <p className="text-sm text-muted-foreground">httpd -t produced no output.</p>}
      </DialogContent>
    </Dialog>
  </div>;
}

function EditorState({ text, destructive = false }: { text: string; destructive?: boolean }) { return <div className={`flex flex-1 items-center justify-center p-6 text-sm ${destructive ? "text-destructive" : "text-muted-foreground"}`}>{text}</div>; }
