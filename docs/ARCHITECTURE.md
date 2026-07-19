# Architecture

## Overview

Xamppify is a Tauri v2 desktop application. React renders the interface; Rust owns privileged work such as filesystem access, database connections, service control, certificate parsing, process execution, and system monitoring.

```
React pages and components
        │ invoke()
        ▼
Tauri command modules
        ▼
Rust domain modules ──► XAMPP files, services, MySQL, OpenSSL, WMI, robocopy
```

## Front end

`src/App.tsx` owns the route map and global providers. The main pages are:

- `Dashboard.tsx` — local deployments in `htdocs`, import/create/hide/delete actions, local + network URLs.
- `FileBrowser.tsx` — workspace file management and text editing.
- `Logs.tsx` — real-time Apache and MySQL/MariaDB log viewing via filesystem watcher.
- `DatabaseManager.tsx` — MySQL connection, browsing, queries, and export.
- `ConfigEditor.tsx` — known XAMPP configuration files with CodeMirror editor.
- `SslManager.tsx` — certificates, keys, and self-signed generation with expiring-cert badge.

- `FileSync.tsx` — robocopy-based file sync to remote machines.
- `Performance.tsx` — real-time CPU/memory/disk gauges via WMI.
- `Settings.tsx` — theme, compact mode, XAMPP location, setup checks.
- `CommandPalette.tsx` — Ctrl+K / Cmd+K global keyboard command palette for page navigation.

### Shared components and hooks

- `useHotkeys` hook — register global keyboard shortcuts (used by command palette).
- `useCertExpiry` hook — checks SSL certificate expiry dates; drives the sidebar badge.
- `Skeleton` component — loading placeholder shimmer.
- `CodeEditor` component — CodeMirror wrapper with auto language detection and one-dark theme.
- `PageHeader` component — consistent page title/description/actions layout.

React Query invalidates data after mutations so the file browser, deployments, and detail views stay in sync. Zustand stores UI preferences such as theme, sidebar state, compact mode, and onboarding completion.

## Rust backend

`src-tauri/src/lib.rs` registers plugins, tray behavior, state, and the command handler. `src-tauri/src/commands/` defines the boundary exposed to the front end. Domain modules contain the implementation:

- `deployment/` — `htdocs` project lifecycle; generates URLs using the detected Apache port and LAN IP.
- `file_browser/` — constrained local file operations.
- `paths.rs` — XAMPP-root discovery, canonical path checks, safe child-process helpers, Apache port detection (`apache_port()`), and LAN IP detection (`local_ip()`).
- `config_editor/` — known configuration discovery and INI parsing.
- `database/` — MySQL connection and query operations.
- `log/` — Apache and MySQL/MariaDB log tailing and **real-time filesystem watching** via `notify` crate.
- `ssl_manager/` — certificate discovery, inspection, and self-signed creation.
- `service/` — local service commands (`sc.exe`) and remote status checks.
- `discovery/` — mDNS, port scanning, heartbeat tracking, and manual machine entries.
- `file_sync/` — `robocopy` wrapper for `/MIR` sync to a remote UNC path.
- `performance/` — WMI query execution via PowerShell for CPU, memory, disk, and uptime metrics.

## Filesystem boundary

The XAMPP root is `XAMPP_HOME` when defined, otherwise `C:\xampp`. Mutating commands canonicalize and validate paths against that root before they read, write, rename, upload, or delete. Deployment names are validated before a directory is created below `htdocs`.

The deployed URL is derived automatically:
- **Apache port**: read from the `Listen` directive in `{XAMPP_HOME}/apache/conf/httpd.conf` (handles `Listen 80`, `Listen 0.0.0.0:8888`, `Listen [::]:8888`).
- **LAN IP**: detected via a UDP socket trick (no data sent) that resolves the primary non-loopback IPv4 address.
- Port 80 is omitted from URLs; non-standard ports are included.

## Process execution

All child processes (robocopy, mysqldump, PowerShell, OpenSSL, `sc.exe`) execute with `CREATE_NO_WINDOW` flags so user actions do not flash a Command Prompt window. OpenSSL receives a standard Windows path rather than an extended-length `\\?\` path to remain compatible with XAMPP's bundled build.

## Packaging

`src-tauri/tauri.conf.json` defines the Xamppify product name, Windows application identifier, installer metadata, and native window configuration. `pnpm tauri build` produces both NSIS and MSI artifacts.
