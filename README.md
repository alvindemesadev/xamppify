# Xamppify

> Your local PHP workspace, simplified.

Xamppify is a Windows desktop workspace for developing and managing projects in a local XAMPP installation. It brings deployments, files, configuration, logs, databases, and SSL certificates into one Tauri desktop application.

## What it does

- Creates HTML or PHP starter projects directly in `htdocs`.
- Imports an existing project folder without altering the original source folder.
- Opens project URLs in the default browser and provides one-click access to project files.
- Lets you hide non-project `htdocs` folders from the deployment grid without deleting them.
- Browses, edits, creates, renames, uploads, and deletes files and folders inside the configured XAMPP installation.
- Reads Apache and MySQL/MariaDB logs with filtering and clear unavailable-log states.
- Connects to MySQL, browses databases and tables, runs queries, and exports databases.
- Edits known Apache, PHP, MySQL, and phpMyAdmin configuration files.
- Lists certificate/key files, reads certificate metadata, and creates self-signed certificates.
- Starts, stops, restarts, and monitors local XAMPP services; the underlying commands run without showing a terminal window.
- Retains LAN-discovery support for compatible remote XAMPP machines.

## Requirements

- Windows 10 or Windows 11
- A local XAMPP installation (default: `C:\xampp`)
- Apache and/or MySQL running when using their respective sites, logs, or database features
- OpenSSL bundled with XAMPP for certificate inspection and generation

Xamppify is designed around a local XAMPP installation. It does not install, configure, or distribute XAMPP itself.

## Install

1. Download the latest `Xamppify_*_x64-setup.exe` release.
2. Close any running Xamppify window.
3. Run the installer, then launch **Xamppify** from the Start menu.
4. Confirm the installation checks on first launch.

The application uses the standard Windows title bar, including Windows 11 Snap Layouts on the maximize button.

### Custom XAMPP location

Set `XAMPP_HOME` before launching Xamppify when XAMPP is not installed in `C:\xampp`.

```powershell
[Environment]::SetEnvironmentVariable('XAMPP_HOME', 'D:\tools\xampp', 'User')
```

Restart Xamppify after changing the variable. The selected directory must be a valid XAMPP root.

## Using Xamppify

### Deployments

The **Deployments** page represents folders within `htdocs`.

- **New deployment** creates a safe, editable HTML or PHP starter project.
- **Import project** copies a selected source folder into `htdocs` and keeps its original source untouched.
- **Customize** opens that deployment in the file workspace.
- **Open site** opens its localhost URL in the default browser.
- **Hide** removes a folder from the grid only; it neither moves nor deletes files. Use **Hidden** to restore it.
- **Delete** permanently deletes the selected deployment folder after confirmation.

### Files

The **Files** workspace supports navigation, filtering, text editing, saving, creating, renaming, uploading, and deletion. Actions target the currently selected directory, and the folder tree/editor refresh after mutations.

### Database

Use **Database** to connect to MySQL, inspect databases and tables, run queries, and export a selected database. Treat write queries carefully: they change the connected database immediately.

### Config and SSL

The **Config** page exposes known local XAMPP configuration files and saves directly to the selected file. The **SSL** page shows available certificate and key files, displays certificate details, and can generate self-signed certificates. Restart Apache if a configuration or certificate change requires it.

### Logs and services

The **Logs** page reads Apache and MySQL/MariaDB logs when they are available. Missing-log messages usually mean the relevant service has not started yet, logging is disabled, or the XAMPP version stores the log under another name. Service controls show the current local service state.

## Safety model

Xamppify constrains its file, config, certificate, and deployment operations to the configured XAMPP root. Path traversal, absolute paths outside that root, and unsafe deployment names are rejected by the Rust backend.

File uploads are preflighted before copying. If an upload fails mid-operation, Xamppify removes files it created during that operation where possible. This is a safeguard, not a substitute for source control or backups.

## Development

### Prerequisites

- Node.js 20+ and pnpm
- Rust stable with the MSVC toolchain
- Visual Studio Build Tools with the Desktop development with C++ workload
- WebView2 Runtime (included with modern Windows)

### Run locally

```powershell
pnpm install
pnpm tauri dev
```

### Validate

```powershell
pnpm build
Set-Location src-tauri
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

### Build installers

```powershell
pnpm tauri build
```

Windows artifacts are written to:

- `src-tauri\target\release\bundle\nsis\`
- `src-tauri\target\release\bundle\msi\`

## Architecture

| Area | Technology | Responsibility |
| --- | --- | --- |
| Desktop shell | Tauri v2 + Rust | Native window, installer, commands, local system access |
| Front end | React 19 + TypeScript + Vite | Application UI and route-level pages |
| Styling | Tailwind CSS v4 + Radix/shadcn primitives | Theme-aware, accessible interface components |
| Client state | TanStack Query + Zustand | Server data, mutation refreshes, interface preferences |
| Local integration | Tauri plugins | File dialogs, safe URL opening, filesystem access |

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for module-level details, [docs/CONTRIBUTING.md](docs/CONTRIBUTING.md) for contribution guidance, and [docs/SECURITY.md](docs/SECURITY.md) for security reporting.

## License

Xamppify is licensed under the [MIT License](LICENSE).
