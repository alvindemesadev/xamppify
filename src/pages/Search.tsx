import { useState } from "react";
import { useMutation } from "@tanstack/react-query";
import { FileText, Search as SearchIcon } from "lucide-react";
import { useNavigate } from "react-router-dom";
import { searchHtdocs } from "@/lib/ipc";
import type { SearchMatch } from "@/lib/ipc";
import { PageHeader } from "@/components/layout/PageHeader";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { toast } from "sonner";

export default function SearchPage() {
  const [query, setQuery] = useState("");
  const [literal, setLiteral] = useState(true);
  const navigate = useNavigate();
  const searchMutation = useMutation({
    mutationFn: () => searchHtdocs(query, literal),
    onError: (reason: Error) => toast.error(reason.message),
  });
  const results = searchMutation.data ?? [];
  const grouped = groupByPath(results);

  return (
    <div className="flex h-full min-w-0 flex-col p-6">
      <PageHeader
        title="Search"
        description="Search file contents inside htdocs using ripgrep."
        actions={<span className="rounded-md bg-muted px-2.5 py-1.5 font-mono text-xs text-muted-foreground">{results.length === 300 ? "300+ matches" : `${results.length} match(es)`}</span>}
      />
      <div className="mb-4 flex flex-wrap items-center gap-2 rounded-xl border border-border bg-card p-3 shadow-sm">
        <label className="relative min-w-64 flex-1">
          <SearchIcon className="absolute left-3 top-2.5 size-4 text-muted-foreground" />
          <Input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => event.key === "Enter" && searchMutation.mutate()}
            placeholder="Search htdocs contents…"
            className="pl-9"
            aria-label="Search htdocs contents"
            autoFocus
          />
        </label>
        <Button onClick={() => searchMutation.mutate()} disabled={!query.trim() || searchMutation.isPending}>
          {searchMutation.isPending ? "Searching…" : "Search"}
        </Button>
        <label className="flex cursor-pointer items-center gap-2 text-sm text-muted-foreground">
          <input type="checkbox" checked={literal} onChange={(event) => setLiteral(event.target.checked)} className="size-4 accent-primary" />
          Literal text
        </label>
      </div>
      {searchMutation.isPending ? (
        <div className="flex flex-1 items-center justify-center rounded-xl border border-border bg-card text-sm text-muted-foreground">Searching htdocs…</div>
      ) : searchMutation.error ? (
        <div className="flex flex-1 items-center justify-center rounded-xl border border-border bg-card text-sm text-destructive">Search failed: {String(searchMutation.error)}</div>
      ) : query.trim() && results.length === 0 ? (
        <div className="flex flex-1 items-center justify-center rounded-xl border border-border bg-card text-sm text-muted-foreground">No matches for “{query.trim()}”.</div>
      ) : results.length === 0 ? (
        <div className="flex flex-1 items-center justify-center rounded-xl border border-border bg-card text-sm text-muted-foreground">Enter a search term to scan htdocs.</div>
      ) : (
        <ScrollArea className="min-h-0 flex-1 rounded-xl border border-border bg-card shadow-sm">
          <div className="p-2">
            {grouped.map(({ path, matches }) => (
              <div key={path} className="mb-1">
                <p className="rounded-md bg-muted/50 px-3 py-1.5 font-mono text-xs text-muted-foreground" title={path}>{path}</p>
                {matches.map((match, index) => (
                  <button key={`${match.path}-${match.line_number}-${index}`} onClick={() => navigate("/files", { state: { filePath: match.path } })} className="group flex w-full items-start gap-3 rounded-md px-3 py-1.5 text-left hover:bg-muted">
                    <FileText className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
                    <span className="w-10 shrink-0 text-right font-mono text-xs text-muted-foreground">{match.line_number}</span>
                    <span className="min-w-0 flex-1 truncate font-mono text-xs text-foreground group-hover:text-primary" title={match.line}>{match.line}</span>
                  </button>
                ))}
              </div>
            ))}
          </div>
        </ScrollArea>
      )}
    </div>
  );
}

function groupByPath(matches: SearchMatch[]): { path: string; matches: SearchMatch[] }[] {
  const groups = new Map<string, SearchMatch[]>();
  for (const match of matches) {
    const list = groups.get(match.path) ?? [];
    list.push(match);
    groups.set(match.path, list);
  }
  return [...groups.entries()].map(([path, list]) => ({ path, matches: list }));
}
