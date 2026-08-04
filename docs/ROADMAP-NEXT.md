# Roadmap — Next Features

## 1. Framework-aware deployments

When creating or importing a deployment, let the user select a framework/stack. The app adjusts the
starter template, defaults, and deployment path accordingly.

### 1a. Framework selection on create / import
- Add a framework dropdown to the "New deployment" and "Import project" dialogs
- Options: HTML (default), PHP, Laravel, WordPress, React (Vite), Node.js
- The deployment card shows a framework badge (replaces or accompanies the current "Local" badge)

### 1b. Framework-specific starter templates
- **HTML**: same as today — `index.html`, `style.css`, `script.js`
- **PHP**: `index.php` with `phpinfo()` section, `.htaccess`
- **Laravel**: skeleton with `public/` as web root, `.env.example`, basic routes
- **WordPress**: `wp-config-sample.php`, `.htaccess`, basic theme structure
- **React**: Vite scaffold (`index.html`, `src/main.jsx`, `vite.config.js`, `package.json`)
- **Node.js**: `server.js` with Express boilerplate, `package.json`

### 1c. Auto-detect existing framework on import
- Scan the imported folder for known files (`composer.json` → Laravel/PHP, `package.json` → Node/React, `wp-config.php` → WordPress)
- Pre-select the framework in the import dialog
- Warn if the selected framework doesn't match detected files

### 1d. Framework-aware URL generation
- Laravel: deployment opens at `{deployment}/public/`
- React/Vite: offer to run `npm run dev` and link to `localhost:5173`
- Node.js: show the start command and expected port

### 1e. Per-deployment tools
- **Laravel**: quick-link to run `php artisan` commands, `.env` editor
- **WordPress**: quick-link to `wp-admin`, `wp-config.php` editor
- **React/Node**: `npm install` / `npm run dev` / `npm run build` buttons
- **All**: terminal panel scoped to the deployment directory

---

## 2. Auto-detect Apache port everywhere

Currently the app reads the `Listen` directive from `httpd.conf` once (`paths::apache_port()`), but
URLs rendered in the UI sometimes omit the port when it's non-standard.

### 2a. Fix all URL construction sites
Audit every place a `localhost` or network URL is built and ensure `apache_port()` is always included
when the port is not 80:
- `deployment/mod.rs` — deployment URL and network URL
- Dashboard `DeploymentCard` — "Open site" and copy buttons
- Any other place that constructs `http://...` without the port

### 2b. Live port monitoring
- Poll `httpd.conf` for changes to the `Listen` directive (or detect port changes via service check)
- If Apache restarts on a different port, automatically update all deployment URLs
- Show the current detected port in the Dashboard service strip

### 2c. Port conflict detection
- Before starting Apache, check if the configured port is already in use
- Warn the user and suggest an alternative port
- Show a "Ports in use" panel in Settings

### 2d. Per-service port display
- Show actual ports for Apache, MySQL, FileZilla in the Dashboard strip and MachineCards
- If MySQL runs on 3307 instead of 3306, reflect that in the Database Manager connection defaults

---

## 3. Deployment virtual hosts

### 3a. Auto-generate virtual host config
- On deployment creation, offer to add a `<VirtualHost>` entry to `httpd-vhosts.conf`
- Support custom `ServerName` (e.g., `myapp.test`)
- Offer to add the domain to the system `hosts` file (with admin prompt)

### 3b. Virtual host list in Dashboard
- Show each deployment's virtual host status (active/inactive)
- Toggle virtual hosts on/off without deleting the config
- "Open with virtual host" button on deployment cards

---

## 4. Deployment environment variables

### 4a. `.env` file editor per deployment
- Detect `.env` or `.env.example` in the deployment root
- Show a key-value editor (not raw text) with add/remove/edit
- Mask sensitive values (passwords, API keys)
- Offer a `.env.example` → `.env` copy on first setup

### 4b. PHP version per deployment
- If multiple PHP versions are installed, let the user pick one per deployment
- Sets the appropriate PHP binary for that deployment's scripts

---

## 5. Composer / NPM integration

### 5a. Dependency management
- "Install dependencies" button per deployment (runs `composer install` or `npm install`)
- Show outdated packages (composer/npm)
- Dependency update logs

### 5b. Quick terminal
- Embedded terminal panel scoped to the deployment directory
- Pre-loaded with common commands (`composer`, `npm`, `php`, `artisan`, `git`)
- Command history per deployment

---

## 6. SSL per deployment

### 6a. Generate certificate for a deployment
- Right-click or card action → "Enable HTTPS for this deployment"
- Auto-generates a self-signed cert with the deployment's domain/name
- Adds the SSL config snippet to `httpd-ssl.conf` or `httpd-vhosts.conf`

### 6b. SSL status badge
- Green lock on cards when HTTPS is configured
- Warning when cert is expired or self-signed

---

## 7. Deployment labels and organization

### 7a. Tags and labels
- Assign color-coded tags to deployments (e.g., "production", "staging", "client-project")
- Filter deployments by tag in the Dashboard

### 7b. Sort and group
- Sort by name, date modified, framework, last opened
- Group deployments by framework or tag

### 7c. Pin / favorite deployments
- Pin important deployments to the top of the list
- Quick-access from sidebar or top bar

---

## 8. Quick actions and shortcuts

### 8a. Right-click context menu on deployment cards
- Open site / Open network / Open in file browser / Edit config / Back up / Delete
- Copy URL (local / network)
- Reveal in File Explorer

### 8b. Global hotkeys
- `Ctrl+Shift+O` — quick-open a deployment
- `Ctrl+Shift+N` — new deployment
- `Ctrl+Shift+L` — open Logs

---

## 9. MySQL port auto-detect

The port work in section 2 is Apache-only. MySQL has the same problem — the Database Manager
defaults to 3306, but XAMPP can run MySQL on 3307 or another port.

### 9a. Detect MySQL port from my.ini
- Read the `port` / `bind-address` directives from `mysql\bin\my.ini`
- Pre-fill the Database Manager connection dialog with the detected port
- Fall back to 3306 if not found

### 9b. Reflect MySQL port everywhere
- Show the detected MySQL port in the Dashboard service strip and MachineCards
- Use it as the default in `mysqlConnect` when the user leaves the port empty

---

## 10. Per-deployment database provisioning

Ties framework selection (section 1) to a real workflow: when you create a Laravel/PHP project,
you almost always need a matching MySQL database.

### 10a. Create database on deployment
- In the "New deployment" dialog, offer "Create a matching MySQL database"
- Auto-names it after the deployment (e.g., `hris_db`)
- Sets `.env` / config file `DB_DATABASE` accordingly (framework-aware)

### 10b. Database management from the deployment card
- "Open database" action on cards → jumps to Database Manager connected to that database
- Show linked database name on the deployment card

---

## 11. Git import

### 11a. Clone a repository as a deployment
- Import dialog gets a "From Git" tab — paste a repo URL
- Backend runs `git clone` into `htdocs`, then applies the framework auto-detect (1c)
- Support private repos with a saved Git credential (Windows Credential Manager, like MySQL)

### 11b. Per-deployment Git panel
- Show branch, dirty status, last commit on the deployment card or a detail pane
- Quick actions: pull, status, open a Git GUI

---

## 12. Duplicate deployment

### 12a. Copy an existing deployment
- Card action → "Duplicate" — copies the folder to `{name}-copy` and creates a fresh DB name
- Useful for staging copies before risky changes

### 12b. Template deployments
- Save any deployment as a reusable template for the "New deployment" dialog

---

## 13. Per-deployment log filtering

### 13a. Filter Apache logs by deployment
- In the Log Viewer, add a deployment dropdown
- Filters access/error log lines to requests hitting that deployment's path
- Ties into framework awareness — Laravel projects have predictable URL patterns

### 13b. Deployment error badge
- If the last N lines of the Apache error log reference a deployment, show a small alert badge on its card

---

## 14. Backup restore

Complements the existing zip backup feature (roadmap item 15, done in v0.4.0).

### 14a. Restore from a backup
- Deployment card action → "Restore backup" — lists `deployment-backups\*.zip`
- Previews contents, restores to a new deployment name (never overwrites an existing folder)

### 14b. Auto-backup scheduling
- Optional per-deployment daily/weekly auto-backup
- Retain last N backups, prune older ones

---

## 15. Custom URL / domain per deployment

Ties into virtual hosts (section 3) but lighter-weight.

### 15a. Custom domain without editing config
- Deployment settings → set a custom `ServerName` like `myapp.test`
- App offers to update `httpd-vhosts.conf` + the Windows `hosts` file
- Card shows the friendly URL alongside the localhost URL

### 15b. URL alias
- Add multiple URLs (localhost + LAN IP + custom domain) per deployment
- All update automatically when the Apache port changes (section 2)

---

## Priority order

1. **Auto-detect Apache port everywhere** (2a, 2b) — bugfix, fast win
2. **Auto-detect MySQL port** (9a, 9b) — same pattern, quick win
3. **Framework selection on create/import** (1a, 1b) — core UX improvement
4. **Auto-detect existing framework on import** (1c)
5. **Per-deployment database provisioning** (10a, 10b) — pairs naturally with #1
6. **Git import** (11a, 11b)
7. **Virtual host auto-generation** (3a, 3b)
8. **Deployment labels and organization** (7a, 7b)
9. **Duplicate deployment** (12a, 12b)
10. **Composer/NPM integration** (5a)
11. **Per-deployment log filtering** (13a, 13b)
12. **Backup restore** (14a, 14b)
13. **SSL per deployment** (6a, 6b)
14. **Everything else**
