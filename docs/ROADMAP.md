# Improvement Roadmap

A prioritized collection of suggested fixes, improvements, and new features for Xamppify.

## Quick fixes (small, concrete)

### 1. Inconsistent Apache port
`src-tauri/src/service/mod.rs:104` hardcodes port 80 for Apache, but `paths::apache_port()` reads `httpd.conf` and deployments use the real port. If Apache runs on 8888, the Services panel shows the wrong port.

### 2. Performance polling spawns PowerShell every 5s
`src-tauri/src/performance/mod.rs:15` spawns a `powershell.exe` process 12 times per minute. Replace with native Win32 calls (`GetSystemTimes`, `GlobalMemoryStatusEx`, `GetDiskFreeSpaceEx` via `windows-sys`, which is already a dependency). Faster and no process churn.

### 3. Test coverage
Only 8 tests exist, all deployment-name validation. The robocopy output parsing, log parser, INI parser, and `build_unc_path` are pure functions — cheap to unit test and would catch regressions.

### 4. Stale bundles
Old `XAMPP LAN Manager_0.1.x` installers clutter `target\release\bundle\`. Worth cleaning periodically.

## Reliability & quality

### 5. App-level logs to a rotating file
`tracing` currently only goes to stdout, invisible in a GUI app. Add `tracing-appender` and write a `logs/xamppify.log` in app data — makes user bug reports 10x more useful.

### 6. `thiserror` error enum
Replace the `Result<_, String>` soup in commands with typed errors — better messages, no stringly-typed failures.

### 7. Code-split the frontend
The single 524 kB JS bundle (161 kB gzip) would shrink with lazy-loaded routes; also improves startup.

## UX improvements

### 8. Command palette actions
It only navigates pages today. Add actions: toggle theme/compact, open a deployment in the file browser, quick sync, start/stop Apache.

### 9. Dashboard service strip
Show Apache/MySQL running state with start/stop buttons right on the Dashboard instead of requiring navigation.

### 10. Config editor safety
"Test Apache config" (`httpd -t` via XAMPP's bundled Apache) before restart, and auto-backup a file before saving.

### 11. Logs upgrades
Regex search, pause streaming, word-wrap toggle.

### 12. File browser
Image preview pane, drag-and-drop upload, and "Recycle Bin instead of permanent delete" (deployments too).

## New features (bigger wins)

### 13. Auto-update
`tauri-plugin-updater` + GitHub Releases — you already distribute installers; this is the single most valuable addition for a desktop app.

### 14. Remember MySQL credentials in Windows Credential Manager
Via keyring — secure (never in a config file), and removes the password friction every session.

### 15. Deployment backups as .zip
The old Backups page was removed due to a mysqldump engine bug; a zip-based version avoids that entirely.

### 16. Search across htdocs
`ripgrep` crate — fast, respects `.gitignore`.

### 17. Scheduled sync
Interval-based robocopy for the File Sync page.

## Top 5 priorities

1. Auto-update (13)
2. Native performance metrics (2)
3. App log file + thiserror (5–6)
4. Dashboard service controls (9)
5. Command palette actions (8)
