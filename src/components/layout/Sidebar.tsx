import { useEffect } from "react";
import { NavLink } from "react-router-dom";
import { Boxes, ScrollText, Folder, Database, FileCode, ShieldCheck, Settings, HardDrive, ArrowRightFromLine, Sun, Moon } from "lucide-react";
import { cn } from "@/lib/utils";
import { useCertExpiry } from "@/hooks/use-cert-expiry";
import { useUiStore } from "@/stores/ui-store";

const navItems = [
  { to: "/", icon: Boxes, label: "Deployments" },
  { to: "/logs", icon: ScrollText, label: "Logs" },
  { to: "/files", icon: Folder, label: "Files" },
  { to: "/database", icon: Database, label: "Database" },
  { to: "/config", icon: FileCode, label: "Config" },
  { to: "/ssl", icon: ShieldCheck, label: "SSL" },
  { to: "/file-sync", icon: ArrowRightFromLine, label: "File Sync" },
  { to: "/performance", icon: HardDrive, label: "Performance" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export function Sidebar({ collapsed = false }: { collapsed?: boolean }) {
  const expiringCerts = useCertExpiry();
  const { theme, setTheme } = useUiStore();

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
  }, [theme]);

  return (
    <aside aria-hidden={collapsed} className={cn("group/sidebar flex shrink-0 flex-col gap-1 overflow-hidden border-r bg-surface transition-[width,padding,border-color] duration-300 ease-[cubic-bezier(.22,1,.36,1)]", collapsed ? "w-0 border-transparent px-0 py-3" : "w-52 border-border p-3")}>
      <p className={cn("whitespace-nowrap px-2 pb-2 text-xs font-medium uppercase tracking-wider text-muted-foreground transition-all duration-200", collapsed ? "translate-x-2 opacity-0" : "translate-x-0 opacity-100")}>Workspace</p>
      {navItems.map(({ to, icon: Icon, label }) => (
        <NavLink
          key={to}
          to={to}
          tabIndex={collapsed ? -1 : undefined}
          className={({ isActive }) =>
            cn(
              "flex h-9 min-w-0 items-center gap-3 rounded-md px-2 text-sm text-muted-foreground transition-[background-color,color,transform] duration-200 hover:bg-surface-hover hover:text-foreground",
              collapsed && "translate-x-2 opacity-0 pointer-events-none",
              isActive && "bg-surface-hover text-foreground"
            )
          }
          title={label}
        >
          <Icon className="size-4" />
          <span>{label}</span>
          {to === "/ssl" && expiringCerts > 0 && (
            <span className="ml-auto flex size-5 items-center justify-center rounded-full bg-red-500 text-[10px] font-bold text-white">
              {expiringCerts}
            </span>
          )}
        </NavLink>
      ))}
      <div className="mt-auto border-t border-border pt-2">
        <button
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
          tabIndex={collapsed ? -1 : undefined}
          className={cn(
            "flex h-9 min-w-0 w-full items-center gap-3 rounded-md px-2 text-sm text-muted-foreground transition-[background-color,color,transform] duration-200 hover:bg-surface-hover hover:text-foreground",
            collapsed && "translate-x-2 opacity-0 pointer-events-none"
          )}
          title={theme === "dark" ? "Switch to light mode" : "Switch to dark mode"}
        >
          {theme === "dark" ? <Sun className="size-4" /> : <Moon className="size-4" />}
          <span>{theme === "dark" ? "Light mode" : "Dark mode"}</span>
        </button>
      </div>
    </aside>
  );
}
