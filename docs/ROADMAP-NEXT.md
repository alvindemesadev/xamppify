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

## Priority order

1. **Auto-detect Apache port everywhere** (2a, 2b) — bugfix, fast win
2. **Framework selection on create/import** (1a, 1b) — core UX improvement
3. **Auto-detect existing framework on import** (1c)
4. **Virtual host auto-generation** (3a, 3b)
5. **Deployment labels and organization** (7a, 7b)
6. **Composer/NPM integration** (5a)
7. **SSL per deployment** (6a, 6b)
8. **Everything else**
