import { useUiStore } from "@/stores/ui-store";

export default function Settings() {
  const { theme, setTheme, resetOnboarding } = useUiStore();

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
                className={`h-8 rounded px-3 text-sm capitalize ${theme === option ? "bg-primary text-primary-foreground" : "bg-secondary text-secondary-foreground"}`}
              >
                {option}
              </button>
            ))}
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
      </div>
    </div>
  );
}
