import { useEffect, useState } from "react";
import { useMutation } from "@tanstack/react-query";
import {
  ArrowRightFromLine,
  CheckCircle2,
  XCircle,
  Activity,
  KeyRound,
  ShieldAlert,
  Copy,
  Check,
  ChevronDown,
  ChevronUp,
  Trash2,
} from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { PageHeader } from "@/components/layout/PageHeader";
import { syncToRemote, testRemoteConnection, getSyncHistory, clearSyncHistory } from "@/lib/ipc";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

type SyncLogItem = {
  result: "success" | "error";
  message: string;
  timestamp?: string;
};

type TestConnectionResult = {
  ping_ok: boolean;
  smb_port_ok: boolean;
  share_accessible: boolean;
  unc_path: string;
  message: string;
  suggestions: string[];
};

export default function FileSyncPage() {
  const [source, setSource] = useState("");
  const [dest, setDest] = useState("");
  const [remoteHost, setRemoteHost] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [showCredentials, setShowCredentials] = useState(false);
  const [logs, setLogs] = useState<SyncLogItem[]>([]);
  const [testResult, setTestResult] = useState<TestConnectionResult | null>(null);
  const [copied, setCopied] = useState(false);
  const [historyLoaded, setHistoryLoaded] = useState(false);

  const refreshHistory = () =>
    getSyncHistory()
      .then((entries) =>
        setLogs(
          entries.map((e) => ({
            result: e.result,
            message: e.message,
            timestamp: e.timestamp,
          }))
        )
      )
      .catch((e) => toast.error(`Failed to load sync history: ${String(e)}`));

  useEffect(() => {
    refreshHistory().finally(() => setHistoryLoaded(true));
  }, []);

  const clearHistoryMutation = useMutation({
    mutationFn: clearSyncHistory,
    onSuccess: () => {
      setLogs([]);
      toast.success("Sync history cleared");
    },
    onError: (e: unknown) => {
      toast.error(`Failed to clear sync history: ${String(e)}`);
    },
  });

  const testMutation = useMutation({
    mutationFn: () => testRemoteConnection(remoteHost, dest, username, password),
    onSuccess: (res) => {
      setTestResult(res);
      if (res.share_accessible) {
        toast.success("Remote connection successful!");
      } else {
        toast.error("Connection issue detected", { description: res.message });
      }
    },
    onError: (e: unknown) => {
      toast.error(`Connection test failed: ${String(e)}`);
    },
  });

  const syncMutation = useMutation({
    mutationFn: () => syncToRemote(source, dest, remoteHost, username, password),
    onSuccess: (res) => {
      if (res.success) {
        toast.success(`Synced ${res.files_copied} items`);
      } else {
        toast.error("Sync completed with errors");
      }
      refreshHistory();
    },
    onError: (e: unknown) => {
      const msg = String(e);
      toast.error(msg);
      setLogs((prev) => [...prev, { result: "error", message: msg }]);
    },
  });

  const handleCopyRegistryCommand = () => {
    const cmd = `reg add "HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Policies\\System" /v LocalAccountTokenFilterPolicy /t REG_DWORD /d 1 /f`;
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    toast.success("Copied PowerShell command to clipboard!");
    setTimeout(() => setCopied(false), 2000);
  };

  const hasError53 =
    logs.some(
      (l) =>
        l.result === "error" &&
        (l.message.includes("ERROR 53") || l.message.includes("network path was not found"))
    ) ||
    (testResult !== null && (!testResult.ping_ok || !testResult.share_accessible));

  return (
    <div className="flex h-full flex-col p-6 space-y-6 overflow-y-auto">
      <PageHeader
        title="File Sync"
        description="Sync XAMPP files to remote machines using robocopy over SMB (Windows only)."
      />

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[1fr_1fr]">
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Sync configuration</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-4">
              <label className="grid gap-1.5 text-sm">
                <span className="text-muted-foreground font-medium">Source path (Local)</span>
                <Input
                  value={source}
                  onChange={(e) => setSource(e.target.value)}
                  placeholder="C:\xampp\htdocs"
                />
              </label>

              <label className="grid gap-1.5 text-sm">
                <span className="text-muted-foreground font-medium">Remote host (IP address or Hostname)</span>
                <Input
                  value={remoteHost}
                  onChange={(e) => setRemoteHost(e.target.value)}
                  placeholder="192.168.2.170"
                />
              </label>

              <label className="grid gap-1.5 text-sm">
                <span className="text-muted-foreground font-medium">Destination path on remote host</span>
                <Input
                  value={dest}
                  onChange={(e) => setDest(e.target.value)}
                  placeholder="C:\xampp\htdocs\hris (or \\192.168.2.170\C$\xampp\htdocs\hris)"
                />
              </label>

              <div>
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="px-0 text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                  onClick={() => setShowCredentials(!showCredentials)}
                >
                  <KeyRound className="size-3.5" />
                  {showCredentials ? "Hide remote credentials" : "Add remote credentials (Optional for password-protected SMB)"}
                  {showCredentials ? <ChevronUp className="size-3.5" /> : <ChevronDown className="size-3.5" />}
                </Button>

                {showCredentials && (
                  <div className="mt-3 grid gap-3 rounded-lg border bg-muted/40 p-3">
                    <label className="grid gap-1 text-xs">
                      <span className="text-muted-foreground">Remote Username</span>
                      <Input
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}
                        placeholder="Administrator"
                        className="h-8 text-xs"
                      />
                    </label>
                    <label className="grid gap-1 text-xs">
                      <span className="text-muted-foreground">Remote Password</span>
                      <Input
                        type="password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        placeholder="••••••••"
                        className="h-8 text-xs"
                      />
                    </label>
                  </div>
                )}
              </div>

              <div className="flex gap-3 pt-2">
                <Button
                  variant="outline"
                  onClick={() => testMutation.mutate()}
                  disabled={testMutation.isPending || !remoteHost || !dest}
                  className="flex-1"
                >
                  <Activity className="size-4 mr-1.5" />
                  {testMutation.isPending ? "Testing..." : "Test connection"}
                </Button>

                <Button
                  onClick={() => syncMutation.mutate()}
                  disabled={syncMutation.isPending || !source || !remoteHost || !dest}
                  className="flex-1"
                >
                  <ArrowRightFromLine className="size-4 mr-1.5" />
                  {syncMutation.isPending ? "Syncing…" : "Sync now"}
                </Button>
              </div>
            </CardContent>
          </Card>

          {testResult && (
            <Card className="border-muted">
              <CardHeader className="py-3">
                <CardTitle className="text-sm font-semibold flex items-center justify-between">
                  <span>Connection Test Diagnostic</span>
                  <span className="text-xs font-mono font-normal text-muted-foreground">
                    {testResult.unc_path}
                  </span>
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3 text-xs">
                <div className="grid grid-cols-3 gap-2">
                  <div className={`flex items-center gap-1.5 rounded p-2 border ${testResult.ping_ok ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-600 dark:text-emerald-400" : "bg-red-500/10 border-red-500/20 text-red-600 dark:text-red-400"}`}>
                    {testResult.ping_ok ? <CheckCircle2 className="size-4" /> : <XCircle className="size-4" />}
                    <span>Ping: {testResult.ping_ok ? "OK" : "Failed"}</span>
                  </div>
                  <div className={`flex items-center gap-1.5 rounded p-2 border ${testResult.smb_port_ok ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-600 dark:text-emerald-400" : "bg-red-500/10 border-red-500/20 text-red-600 dark:text-red-400"}`}>
                    {testResult.smb_port_ok ? <CheckCircle2 className="size-4" /> : <XCircle className="size-4" />}
                    <span>SMB Port 445: {testResult.smb_port_ok ? "Open" : "Closed"}</span>
                  </div>
                  <div className={`flex items-center gap-1.5 rounded p-2 border ${testResult.share_accessible ? "bg-emerald-500/10 border-emerald-500/20 text-emerald-600 dark:text-emerald-400" : "bg-red-500/10 border-red-500/20 text-red-600 dark:text-red-400"}`}>
                    {testResult.share_accessible ? <CheckCircle2 className="size-4" /> : <XCircle className="size-4" />}
                    <span>Share Access: {testResult.share_accessible ? "OK" : "Failed"}</span>
                  </div>
                </div>

                {testResult.suggestions.length > 0 && (
                  <div className="space-y-1.5 bg-amber-500/10 border border-amber-500/20 rounded p-3 text-amber-800 dark:text-amber-300">
                    <p className="font-semibold flex items-center gap-1">
                      <ShieldAlert className="size-4 text-amber-500" />
                      Required Actions:
                    </p>
                    <ul className="list-disc list-inside space-y-1 pl-1">
                      {testResult.suggestions.map((s, idx) => (
                        <li key={idx}>{s}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </CardContent>
            </Card>
          )}
        </div>

        <div className="space-y-6">
          <Card>
            <CardHeader className="flex-row items-center justify-between space-y-0">
              <CardTitle>Sync history & Logs</CardTitle>
              <Button
                size="sm"
                variant="ghost"
                onClick={() => clearHistoryMutation.mutate()}
                disabled={clearHistoryMutation.isPending || logs.length === 0}
                className="text-xs text-muted-foreground hover:text-foreground"
              >
                <Trash2 className="size-3.5 mr-1" />
                Clear
              </Button>
            </CardHeader>
            <CardContent className="max-h-[450px] space-y-2 overflow-y-auto">
              {logs.length === 0 && (
                <p className="py-12 text-center text-sm text-muted-foreground">
                  {historyLoaded
                    ? "No sync operations yet. Configure the fields above and click \"Sync now\"."
                    : "Loading sync history…"}
                </p>
              )}
              {logs.map((log, i) => (
                <div
                  key={i}
                  className={`flex items-start gap-2 rounded-lg p-3 text-xs font-mono ${
                    log.result === "success"
                      ? "bg-emerald-500/10 text-emerald-700 dark:text-emerald-300 border border-emerald-500/20"
                      : "bg-red-500/10 text-red-700 dark:text-red-300 border border-red-500/20"
                  }`}
                >
                  {log.result === "success" ? (
                    <CheckCircle2 className="mt-0.5 size-4 shrink-0 text-emerald-500" />
                  ) : (
                    <XCircle className="mt-0.5 size-4 shrink-0 text-red-500" />
                  )}
                  <div className="overflow-x-auto whitespace-pre-wrap flex-1">
                    {log.timestamp && (
                      <div className="text-[10px] opacity-60 mb-0.5">{log.timestamp}</div>
                    )}
                    {log.message}
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>

          {hasError53 && (
            <Card className="border-amber-500/30 bg-amber-500/5">
              <CardHeader className="py-3">
                <CardTitle className="text-sm font-semibold flex items-center gap-2 text-amber-600 dark:text-amber-400">
                  <ShieldAlert className="size-4" />
                  How to Fix Error 53 (Network Path Not Found)
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-3 text-xs text-muted-foreground">
                <ol className="list-decimal list-inside space-y-1.5">
                  <li>
                    <strong className="text-foreground">Verify target IP is online:</strong> Ping{" "}
                    <code className="bg-muted px-1 rounded">{remoteHost || "192.168.2.170"}</code>.
                  </li>
                  <li>
                    <strong className="text-foreground">Enable File Sharing:</strong> On remote machine, set network profile to <strong>Private</strong> and allow File & Printer Sharing in Firewall.
                  </li>
                  <li>
                    <strong className="text-foreground">Enable Remote C$ Share (Workgroups):</strong> Run this command on the <strong>target machine</strong> in Admin PowerShell:
                  </li>
                </ol>
                <div className="flex items-center gap-2 bg-muted p-2 rounded border text-[11px] font-mono text-foreground">
                  <span className="flex-1 overflow-x-auto">
                    reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System" /v LocalAccountTokenFilterPolicy /t REG_DWORD /d 1 /f
                  </span>
                  <Button size="icon" variant="ghost" className="size-7 shrink-0" onClick={handleCopyRegistryCommand}>
                    {copied ? <Check className="size-3.5 text-emerald-500" /> : <Copy className="size-3.5" />}
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
