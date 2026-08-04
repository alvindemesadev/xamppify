import { PanelLeftClose, PanelLeft } from "lucide-react";
import { useUiStore } from "@/stores/ui-store";

export function TopBar() {
  const { sidebarOpen, toggleSidebar } = useUiStore();

  return (
    <header className="flex h-12 items-center gap-2 border-b border-border px-4">
      <button
        onClick={toggleSidebar}
        className="flex size-8 items-center justify-center rounded-md text-muted-foreground transition-[background-color,color,transform] duration-200 hover:bg-surface-hover hover:text-foreground active:scale-95"
        title={sidebarOpen ? "Close sidebar" : "Open sidebar"}
      >
        {sidebarOpen ? <PanelLeftClose className="size-4 transition-transform duration-200" /> : <PanelLeft className="size-4 transition-transform duration-200" />}
      </button>
      <span className="text-sm font-medium">Workspace</span>
      <span className="ml-auto hidden truncate rounded-md bg-muted px-2.5 py-1.5 text-xs text-muted-foreground sm:inline max-w-48">Local XAMPP workspace</span>
    </header>
  );
}
