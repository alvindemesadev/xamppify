import { useEffect, type ReactNode } from "react";
import { useUiStore } from "@/stores/ui-store";
import { Sidebar } from "./Sidebar";
import { TopBar } from "./TopBar";

interface AppShellProps {
  children: ReactNode;
}

export function AppShell({ children }: AppShellProps) {
  const sidebarOpen = useUiStore((s) => s.sidebarOpen);
  const theme = useUiStore((s) => s.theme);
  const compactMode = useUiStore((s) => s.compactMode);

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, [theme]);

  return (
    <div className="flex h-screen flex-col overflow-hidden bg-background">
      <div className="flex min-h-0 flex-1">
        <Sidebar collapsed={!sidebarOpen} />
        <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
          <TopBar />
          <main className={`app-main-scroll-region min-h-0 flex-1 overflow-y-auto overscroll-contain ${compactMode ? "p-2" : ""}`}>{children}</main>
        </div>
      </div>
    </div>
  );
}
