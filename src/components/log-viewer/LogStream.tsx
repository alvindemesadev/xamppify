import { useEffect, useMemo, useRef, useState } from "react";
import { AlertTriangle, Copy, FileSearch, Pause, Play, RefreshCw } from "lucide-react";
import { useLogs } from "@/hooks/use-logs";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { LogLine } from "@/lib/types";

interface LogStreamProps { source: "Apache" | "MySQL"; }
const levels = ["ERROR", "WARN", "INFO", "DEBUG"];
const levelStyle: Record<string, string> = { ERROR: "bg-destructive/10 text-destructive", WARN: "bg-amber-500/10 text-amber-700 dark:text-amber-400", INFO: "bg-sky-500/10 text-sky-700 dark:text-sky-400", DEBUG: "bg-muted text-muted-foreground" };

export function LogStream({ source }: LogStreamProps) {
  const { data: logs, isLoading, error, refetch, isFetching } = useLogs(source.toLowerCase());
  const [filter, setFilter] = useState("");
  const [autoScroll, setAutoScroll] = useState(true);
  const [visibleLevels, setVisibleLevels] = useState(levels);
  const bottomRef = useRef<HTMLDivElement>(null);
  const filtered = useMemo(() => logs?.filter((line) => visibleLevels.includes(line.level) && `${line.level} ${line.message}`.toLowerCase().includes(filter.toLowerCase())) ?? [], [logs, filter, visibleLevels]);

  useEffect(() => { if (autoScroll && !error) bottomRef.current?.scrollIntoView({ behavior: "smooth" }); }, [filtered, autoScroll, error]);
  const toggleLevel = (level: string) => setVisibleLevels((current) => current.includes(level) ? current.filter((item) => item !== level) : [...current, level]);
  const stateLabel = error ? "Log unavailable" : `${filtered.length} visible ${filtered.length === 1 ? "entry" : "entries"}`;

  return <section className="flex min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-card shadow-sm">
    <div className="border-b border-border bg-muted/30 px-4 py-3">
      <div className="flex items-center justify-between gap-2">
        <div><h2 className="font-medium">{source}</h2><p className={`text-xs ${error ? "text-destructive" : "text-muted-foreground"}`}>{stateLabel}</p></div>
        <div className="flex items-center gap-1">
          <Button variant={autoScroll ? "secondary" : "ghost"} size="icon-xs" onClick={() => setAutoScroll(!autoScroll)} title={autoScroll ? "Pause auto-scroll" : "Resume auto-scroll"} aria-label={autoScroll ? "Pause auto-scroll" : "Resume auto-scroll"}>{autoScroll ? <Pause /> : <Play />}</Button>
          <Button variant="ghost" size="icon-xs" onClick={() => refetch()} disabled={isFetching} title="Refresh log" aria-label={`Refresh ${source} log`}><RefreshCw className={isFetching ? "animate-spin" : ""} /></Button>
          <Button variant="ghost" size="icon-xs" disabled={!filtered.length} onClick={() => navigator.clipboard.writeText(filtered.map((line) => line.message).join("\n"))} title="Copy visible entries" aria-label={`Copy visible ${source} log entries`}><Copy /></Button>
        </div>
      </div>
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <Input value={filter} onChange={(event) => setFilter(event.target.value)} placeholder="Search messages or levels" className="min-w-44 flex-1" aria-label={`Search ${source} log`} disabled={!!error} />
        {levels.map((level) => <Button key={level} variant={visibleLevels.includes(level) ? "secondary" : "ghost"} size="xs" onClick={() => toggleLevel(level)} disabled={!!error} aria-pressed={visibleLevels.includes(level)}>{level}</Button>)}
      </div>
    </div>
    <ScrollArea className="min-h-0 flex-1"><div className="min-w-[38rem] font-mono text-xs">
      {isLoading ? <Empty text="Loading log entries…" /> : error ? <LogUnavailable source={source} message={String(error)} onRefresh={() => refetch()} refreshing={isFetching} /> : filtered.length === 0 ? <Empty text={logs?.length ? "No entries match the current filters." : "No log entries have been recorded yet."} /> : <>{filtered.map((line: LogLine, index) => <div key={`${line.timestamp}-${index}`} className="grid grid-cols-[3rem_8rem_4rem_minmax(0,1fr)] gap-3 border-b border-border/60 px-4 py-2 hover:bg-muted/50"><span className="text-right text-muted-foreground">{index + 1}</span><span className="text-muted-foreground">{line.timestamp || "—"}</span><span className={`w-fit rounded px-1.5 py-0.5 text-[10px] font-bold ${levelStyle[line.level] ?? levelStyle.INFO}`}>{line.level}</span><span className="break-words text-foreground">{line.message}</span></div>)}<div ref={bottomRef} /></>}
    </div></ScrollArea>
  </section>;
}

function Empty({ text }: { text: string }) { return <div className="flex h-48 items-center justify-center px-6 text-center text-sm text-muted-foreground">{text}</div>; }

function LogUnavailable({ source, message, onRefresh, refreshing }: { source: string; message: string; onRefresh: () => void; refreshing: boolean }) {
  return <div className="flex min-h-56 items-center justify-center p-6 text-center">
    <div className="max-w-sm"><div className="mx-auto mb-3 flex size-10 items-center justify-center rounded-full bg-destructive/10 text-destructive"><AlertTriangle className="size-5" /></div><h3 className="font-sans text-sm font-semibold text-foreground">{source} log unavailable</h3><p className="mt-1 font-sans text-sm leading-6 text-muted-foreground">{message}</p>{source === "MySQL" && <p className="mt-2 font-sans text-xs leading-5 text-muted-foreground">The app checks the local XAMPP MySQL data folder for the error log. If it is not there, review the <code className="rounded bg-muted px-1 py-0.5">log_error</code> setting in <code className="rounded bg-muted px-1 py-0.5">my.ini</code>.</p>}<Button className="mt-4" size="sm" variant="outline" onClick={onRefresh} disabled={refreshing}><FileSearch />{refreshing ? "Checking…" : "Check again"}</Button></div>
  </div>;
}
