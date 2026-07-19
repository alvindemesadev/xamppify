# Architecture

## Overview

Xamppify is a Tauri v2 desktop application. React renders the interface; Rust owns privileged work such as filesystem access, database connections, service control, certificate parsing, and process execution.

```
React pages and components
        │ invoke()
        ▼
Tauri command modules
        ▼
Rust domain modules ──► XAMPP files, services, MySQL, OpenSSL
```

## Front end

`src/App.tsx` owns the route map and global providers. The main pages are:

- `Dashboard.tsx` — local deployments in `htdocs`, import/create/hide/delete actions.
- `FileBrowser.tsx` — workspace file management and text editing.
- `Logs.tsx` — Apache and MySQL/MariaDB log viewing.
- `DatabaseManager.tsx` — MySQL connection, browsing, queries, and export.
- `ConfigEditor.tsx` — known XAMPP configuration files.
- `SslManager.tsx` — certificates, keys, and self-signed generation.
- `Settings.tsx` — local application preferences.

React Query invalidates data after mutations so the file browser, deployments, and detail views stay in sync. Zustand stores UI preferences such as theme and onboarding completion.

## Rust backend

`src-tauri/src/lib.rs` registers plugins, tray behavior, state, and the command handler. `src-tauri/src/commands/` defines the boundary exposed to the front end. Domain modules contain the implementation:

- `deployment/` — `htdocs` project lifecycle.
- `file_browser/` — constrained local file operations.
- `paths.rs` — XAMPP-root discovery, canonical path checks, safe child-process helpers.
- `config_editor/` — known configuration discovery and INI parsing.
- `database/` — MySQL connection and query operations.
- `log/` — Apache and MySQL/MariaDB log tailing and parsing.
- `ssl_manager/` — certificate discovery, inspection, and self-signed creation.
- `service/` — local service commands and remote status checks.
- `discovery/` — mDNS, port scanning, heartbeat tracking, and manual machine entries.

## Filesystem boundary

The XAMPP root is `XAMPP_HOME` when defined, otherwise `C:\xampp`. Mutating commands canonicalize and validate paths against that root before they read, write, rename, upload, or delete. Deployment names are validated before a directory is created below `htdocs`.

## Process execution

OpenSSL and Windows service commands run with hidden creation flags so user actions do not flash a Command Prompt window. OpenSSL receives a standard Windows path rather than an extended-length `\\?\` path to remain compatible with XAMPP's bundled build.

## Packaging

`src-tauri/tauri.conf.json` defines the Xamppify product name, Windows application identifier, installer metadata, and native window configuration. `pnpm tauri build` produces both NSIS and MSI artifacts.
