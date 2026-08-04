import { Sun, Moon } from "lucide-react";
import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { useUiStore } from "@/stores/ui-store";
import { useUpdaterStore } from "@/stores/updater-store";

export default function Settings() {
  const { theme, setTheme, compactMode, toggleCompactMode, resetOnboarding } = useUiStore();
  const { checking, installing, lastResult, error, checkNow } = useUpdaterStore();
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(null));
  }, []);

  return (
    <div className="p-6">
      <h1 className="text-2xl font-semibold">Settings</h1>
      <div className="mt-6 max-w-lg space-y-5">
        <section className="rounded-lg border border-border p-4">
          <h2 className="text-sm font-medium">Theme</h2>
          <div className="mt-3 flex gap-2">
            {(["dark", "light"] as const).map((option) => (
              <button
                key={option}
                onClick={() => setTheme(option)}
                className={`flex items-center gap-1.5 h-8 rounded px-3 text-sm capitalize ${theme === option ? "bg-primary text-primary-foreground" : "bg-secondary text-secondary-foreground"}`}
              >
                {option === "dark" ? <Moon className="size-4" /> : <Sun className="size-4" />}
                {option}
              </button>
            ))}
          </div>
        </section>
        <section className="rounded-lg border border-border p-4">
          <h2 className="text-sm font-medium">Layout</h2>
          <div className="mt-3 flex items-center gap-3">
            <span className="text-sm text-muted-foreground">Compact monitoring mode</span>
            <button
              onClick={toggleCompactMode}
              className={`relative h-6 w-11 rounded-full transition-colors ${compactMode ? "bg-primary" : "bg-border"}`}
            >
              <span className={`absolute top-0.5 block h-5 w-5 rounded-full bg-white transition-transform ${compactMode ? "translate-x-5" : "translate-x-0.5"}`} />
            </button>
          </div>
        </section>
        <section className="rounded-lg border border-border p-4 text-sm text-muted-foreground">
          <h2 className="font-medium text-foreground">XAMPP location</h2>
          <p className="mt-2">Set the <code>XAMPP_HOME</code> environment variable before launching the app to use an installation other than <code>C:\\xampp</code>.</p>
        </section>
        <section className="rounded-lg border border-border p-4 text-sm">
          <h2 className="font-medium">Setup checks</h2>
          <p className="mt-2 text-muted-foreground">Review XAMPP, OpenSSL, log, and htdocs availability again.</p>
          <button onClick={resetOnboarding} className="mt-3 h-8 rounded bg-secondary px-3 text-sm text-secondary-foreground">Run setup checks</button>
        </section>
        <section className="rounded-lg border border-border p-4 text-sm">
          <h2 className="font-medium">Updates</h2>
          <p className="mt-2 text-muted-foreground">
            Version {version ?? "…"} — updates are downloaded from GitHub Releases and installed automatically.
          </p>
          <div className="mt-3 flex items-center gap-3">
            <button
              onClick={() => checkNow({ notify: true })}
              disabled={checking || installing}
              className="h-8 rounded bg-secondary px-3 text-sm text-secondary-foreground disabled:opacity-50"
            >
              {checking ? "Checking…" : "Check for updates"}
            </button>
            {lastResult === "none" && !checking && (
              <span className="text-xs text-muted-foreground">You're up to date</span>
            )}
            {lastResult === "error" && !checking && error && (
              <span className="text-xs text-red-600 dark:text-red-400">Check failed</span>
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
