import { NavLink } from "react-router-dom";
import { Boxes, Monitor, Folder, Database, FileCode, ShieldCheck, Settings } from "lucide-react";
import { cn } from "@/lib/utils";

const navItems = [
  { to: "/", icon: Boxes, label: "Deployments" },
  { to: "/logs", icon: Monitor, label: "Logs" },
  { to: "/files", icon: Folder, label: "Files" },
  { to: "/database", icon: Database, label: "Database" },
  { to: "/config", icon: FileCode, label: "Config" },
  { to: "/ssl", icon: ShieldCheck, label: "SSL" },
  { to: "/settings", icon: Settings, label: "Settings" },
];

export function Sidebar({ collapsed = false }: { collapsed?: boolean }) {
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
              "flex h-9 min-w-46 items-center gap-3 rounded-md px-2 text-sm text-muted-foreground transition-[background-color,color,transform] duration-200 hover:bg-surface-hover hover:text-foreground",
              collapsed && "translate-x-2 opacity-0 pointer-events-none",
              isActive && "bg-surface-hover text-foreground"
            )
          }
          title={label}
        >
          <Icon className="size-4" />
          <span>{label}</span>
        </NavLink>
      ))}
    </aside>
  );
}
