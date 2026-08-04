import { Download } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useUpdaterStore } from "@/stores/updater-store";

export function UpdateDialog() {
  const { update, checking, installing, progress, error, dismissUpdate, installUpdate } =
    useUpdaterStore();

  return (
    <Dialog
      open={update !== null && !checking}
      onOpenChange={(open) => {
        if (!open && !installing) dismissUpdate();
      }}
    >
      {update && (
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Update available: v{update.version}</DialogTitle>
            <DialogDescription className="pt-1">
              {update.body ? (
                <div className="max-h-48 overflow-y-auto whitespace-pre-wrap rounded border border-border bg-muted/50 p-3 text-xs font-mono">
                  {update.body}
                </div>
              ) : (
                "A new version of Xamppify is available."
              )}
            </DialogDescription>
          </DialogHeader>

          {installing && (
            <div className="space-y-1.5">
              <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                <div
                  className="h-full rounded-full bg-primary transition-all duration-200"
                  style={{ width: `${progress ?? 100}%` }}
                />
              </div>
              <p className="text-xs text-muted-foreground">
                {progress !== null && progress < 100
                  ? `Downloading… ${progress}%`
                  : "Installing…"}
              </p>
            </div>
          )}

          {error && <p className="text-xs text-red-600 dark:text-red-400">Update failed: {error}</p>}

          <DialogFooter>
            <Button variant="outline" onClick={dismissUpdate} disabled={installing}>
              Later
            </Button>
            <Button onClick={installUpdate} disabled={installing}>
              <Download className="size-4 mr-1.5" />
              {installing ? "Updating…" : "Download & Install"}
            </Button>
          </DialogFooter>
        </DialogContent>
      )}
    </Dialog>
  );
}
