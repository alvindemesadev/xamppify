import { useQuery } from "@tanstack/react-query";
import { CheckCircle2, CircleAlert, FolderCog, FolderPlus } from "lucide-react";
import { getAppHealth } from "@/lib/ipc";
import { useUiStore } from "@/stores/ui-store";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";

export function Onboarding() {
  const complete = useUiStore((state) => state.completeOnboarding);
  const completeOnboarding = useUiStore((state) => state.onboardingComplete);
  const health = useQuery({ queryKey: ["app-health"], queryFn: getAppHealth });

  if (completeOnboarding) return null;

  const checks = [
    [health.data?.xampp_available, "XAMPP installation", health.data?.xampp_root ?? "Checking location…"],
    [health.data?.openssl_available, "OpenSSL", "Needed for certificate details and generation"],
    [health.data?.apache_log_available || health.data?.mysql_log_available, "Log files", "Start XAMPP services to make logs available"],
  ] as const;

  return (
    <Dialog open>
      <DialogContent showCloseButton={false} className="max-w-lg">
        <DialogHeader>
          <DialogTitle>Welcome to Xamppify</DialogTitle>
          <DialogDescription>Confirm the local setup, then create and manage projects in your XAMPP htdocs folder.</DialogDescription>
        </DialogHeader>
        <div className="space-y-3">
          {checks.map(([ready, label, detail]) => (
            <div key={label} className="flex gap-3 rounded-lg border border-border p-3">
              {ready ? <CheckCircle2 className="size-5 shrink-0 text-emerald-500" /> : <CircleAlert className="size-5 shrink-0 text-amber-500" />}
              <div><p className="font-medium">{label}</p><p className="text-xs text-muted-foreground">{detail}</p></div>
            </div>
          ))}
        </div>
        <div className="grid grid-cols-2 gap-3 text-xs text-muted-foreground">
          <div className="flex gap-2"><FolderCog className="size-4" />Set <code>XAMPP_HOME</code> before launch for a custom installation.</div>
          <div className="flex gap-2"><FolderPlus className="size-4" />New deployments are created safely inside <code>htdocs</code>.</div>
        </div>
        <DialogFooter><Button onClick={complete}>Open deployments</Button></DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
