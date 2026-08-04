import { useEffect, useMemo, useRef, useState } from "react";
import { Copy, CopyCheck, Pause, Play, Regex, WrapText, CircleStop, CirclePlay } from "lucide-react";
import { useLogs } from "@/hooks/use-logs";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { LogLine } from "@/lib/types";

interface LogStreamProps { source: "Apache" | "MySQL"; }
const levels = ["ERROR", "WARN", "INFO", "DEBUG"];
const levelStyle: Record<string, string> = { ERROR: "bg-destructive/10 text-destructive", WARN: "bg-amber-500/10 text-amber-700 dark:text-amber-400", INFO: "bg-sky-500/10 text-sky-700 dark:text-sky-400", DEBUG: "bg-muted text-muted-foreground" };

function matchFilter(line: LogLine, filter: string, regex: boolean): boolean {
  if (!filter.trim()) return true;
  const text = `${line.level} ${line.message}`;
  if (!regex) return text.toLowerCase().includes(filter.toLowerCase());
  try {
    return new RegExp(filter, "i").test(text);
  } catch {
    return false;
  }
}

export function LogStream({ source }: LogStreamProps) {
  const [streamPaused, setStreamPaused] = useState(false);
  const { data: logs } = useLogs(source.toLowerCase(), streamPaused);
  const [filter, setFilter] = useState("");
  const [regex, setRegex] = useState(false);
  const [autoScroll, setAutoScroll] = useState(true);
  const [wrap, setWrap] = useState(false);
  const [visibleLevels, setVisibleLevels] = useState(levels);
  const [copiedLine, setCopiedLine] = useState<number | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  const invalidRegex = regex && !!filter.trim() && (() => { try { new RegExp(filter, "i"); return false; } catch { return true; } })();
  const filtered = useMemo(() => logs?.filter((line) => visibleLevels.includes(line.level) && matchFilter(line, filter, regex)) ?? [], [logs, filter, regex, visibleLevels]);

  useEffect(() => { if (autoScroll) bottomRef.current?.scrollIntoView({ behavior: "smooth" }); }, [filtered, autoScroll]);
  const toggleLevel = (level: string) => setVisibleLevels((current) => current.includes(level) ? current.filter((item) => item !== level) : [...current, level]);

  return <section className="flex min-h-0 flex-col overflow-hidden rounded-xl border border-border bg-card shadow-sm">
    <div className="border-b border-border bg-muted/30 px-4 py-3">
      <div className="flex items-center justify-between gap-2">
        <div><h2 className="font-medium">{source}</h2><p className="text-xs text-muted-foreground">{streamPaused ? "Stream paused" : `${filtered.length} visible ${filtered.length === 1 ? "entry" : "entries"}`}</p></div>
        <div className="flex items-center gap-1">
          <Button variant={wrap ? "secondary" : "ghost"} size="icon-xs" onClick={() => setWrap(!wrap)} title="Toggle word wrap" aria-label="Toggle word wrap" aria-pressed={wrap}><WrapText /></Button>
          <Button variant={streamPaused ? "secondary" : "ghost"} size="icon-xs" onClick={() => setStreamPaused(!streamPaused)} title={streamPaused ? "Resume live log stream" : "Pause live log stream"} aria-label={streamPaused ? "Resume live log stream" : "Pause live log stream"}>{streamPaused ? <CirclePlay /> : <CircleStop />}</Button>
          <Button variant={autoScroll ? "secondary" : "ghost"} size="icon-xs" onClick={() => setAutoScroll(!autoScroll)} title={autoScroll ? "Pause auto-scroll" : "Resume auto-scroll"} aria-label={autoScroll ? "Pause auto-scroll" : "Resume auto-scroll"}>{autoScroll ? <Pause /> : <Play />}</Button>
          <Button variant="ghost" size="icon-xs" disabled={!filtered.length} onClick={() => navigator.clipboard.writeText(filtered.map((line) => line.message).join("\n"))} title="Copy visible entries" aria-label={`Copy visible ${source} log entries`}><Copy /></Button>
        </div>
      </div>
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <div className="relative min-w-0 flex-1">
          <Input value={filter} onChange={(event) => setFilter(event.target.value)} placeholder={regex ? "Regex search (e.g. ^\\[error\\]|fatal)" : "Search messages or levels"} className={`min-w-0 flex-1 ${invalidRegex ? "border-destructive" : ""}`} aria-label={`Search ${source} log`} />
          <Button variant={regex ? "secondary" : "ghost"} size="icon-xs" className="absolute right-1.5 top-1/2 -translate-y-1/2" onClick={() => setRegex(!regex)} title={regex ? "Using regular expressions" : "Use regular expressions"} aria-label="Toggle regular expression search" aria-pressed={regex}><Regex /></Button>
        </div>
          {levels.map((level) => <Button key={level} variant={visibleLevels.includes(level) ? "secondary" : "ghost"} size="xs" onClick={() => toggleLevel(level)} aria-pressed={visibleLevels.includes(level)}>{level}</Button>)}
      </div>
    </div>
    <ScrollArea className="min-h-0 flex-1"><div className={`font-mono text-xs ${wrap ? "" : "min-w-0 overflow-x-auto"}`}>
      {filtered.length === 0 ? <Empty text={invalidRegex ? "The regular expression is invalid." : logs?.length ? "No entries match the current filters." : "No log entries have been recorded yet."} /> : <>{filtered.map((line: LogLine, index) => <div key={`${line.timestamp}-${index}`} className={`group grid grid-cols-[3rem_8rem_4rem_minmax(0,1fr)] gap-3 border-b border-border/60 px-4 py-2 hover:bg-muted/50 ${wrap ? "break-words" : ""}`}><span className="text-right text-muted-foreground">{index + 1}</span><span className="text-muted-foreground">{line.timestamp || "—"}</span><span className={`w-fit rounded px-1.5 py-0.5 text-[10px] font-bold ${levelStyle[line.level] ?? levelStyle.INFO}`}>{line.level}</span><span className={`flex items-start gap-2 text-foreground ${wrap ? "whitespace-pre-wrap break-words" : "whitespace-pre"}`}><span className="flex-1">{line.message}</span><button onClick={() => { navigator.clipboard.writeText(line.message).catch(() => {}); setCopiedLine(index); setTimeout(() => setCopiedLine(null), 1500); }} className="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity" title="Copy line">{copiedLine === index ? <CopyCheck className="size-3 text-green-400" /> : <Copy className="size-3 text-muted-foreground" />}</button></span></div>)}<div ref={bottomRef} /></>}
    </div></ScrollArea>
  </section>;
}

function Empty({ text }: { text: string }) { return <div className="flex h-48 items-center justify-center px-6 text-center text-sm text-muted-foreground">{text}</div>; }
