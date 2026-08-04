// Framework-specific starter templates for new deployments.

pub const FRAMEWORKS: &[&str] = &["html", "php", "laravel", "wordpress", "react", "node"];

pub fn framework_label(framework: &str) -> &'static str {
    match framework {
        "html" => "HTML website",
        "php" => "PHP website",
        "laravel" => "Laravel",
        "wordpress" => "WordPress",
        "react" => "React (Vite)",
        "node" => "Node.js",
        _ => "Custom",
    }
}

fn title_case(name: &str) -> String {
    name.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns a list of (relative_path, content) files for the chosen framework.
pub fn template_files(name: &str, framework: &str) -> Vec<(String, String)> {
    let title = title_case(name);
    match framework {
        "php" => vec![
            ("index.php".to_string(), php_index(&title)),
            ("assets/styles.css".to_string(), stylesheet()),
            ("assets/app.js".to_string(), javascript()),
            (".gitignore".to_string(), ".DS_Store\nThumbs.db\n".to_string()),
        ],
        "laravel" => vec![
            ("index.php".to_string(), laravel_index(name)),
            ("public/.gitkeep".to_string(), String::new()),
            (".env.example".to_string(), laravel_env(name)),
            (".gitignore".to_string(), "/vendor\n/node_modules\n/.env\n/storage/logs/*.log\n/public/storage\n".to_string()),
            ("composer.json".to_string(), format!(
                "{{\n  \"name\": \"xamppify/{}\",\n  \"type\": \"project\",\n  \"require\": {{}}\n}}\n", name
            )),
        ],
        "wordpress" => vec![
            ("wp-config-sample.php".to_string(), wordpress_config(name)),
            (".htaccess".to_string(), wordpress_htaccess()),
            ("index.php".to_string(), wordpress_index()),
            (".gitignore".to_string(), "/wp-content/uploads\n*.log\n".to_string()),
        ],
        "react" => vec![
            ("index.html".to_string(), react_html(&title)),
            ("src/main.jsx".to_string(), react_main(&title)),
            ("src/App.jsx".to_string(), react_app()),
            ("src/App.css".to_string(), react_css()),
            ("vite.config.js".to_string(), react_vite_config()),
            ("package.json".to_string(), react_package(name)),
            (".gitignore".to_string(), "node_modules\ndist\n.env\n".to_string()),
        ],
        "node" => vec![
            ("server.js".to_string(), node_server(&title)),
            ("package.json".to_string(), node_package(name)),
            (".gitignore".to_string(), "node_modules\n.env\n".to_string()),
        ],
        _ => vec![
            ("index.html".to_string(), html_index(&title)),
            ("assets/styles.css".to_string(), stylesheet()),
            ("assets/app.js".to_string(), javascript()),
            (".gitignore".to_string(), ".DS_Store\nThumbs.db\n".to_string()),
        ],
    }
}

fn html_index(title: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>{title}</title>\n  <link rel=\"stylesheet\" href=\"assets/styles.css\">\n</head>\n<body>\n  <main class=\"hero\">\n    <p class=\"eyebrow\">Local XAMPP deployment</p>\n    <h1>{title}</h1>\n    <p>Edit <code>index.html</code>, <code>assets/styles.css</code>, and <code>assets/app.js</code> from Xamppify.</p>\n    <button id=\"hello-button\">Test JavaScript</button>\n  </main>\n  <script src=\"assets/app.js\"></script>\n</body>\n</html>\n"
    )
}

fn php_index(title: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n  <title>{title}</title>\n  <link rel=\"stylesheet\" href=\"assets/styles.css\">\n</head>\n<body>\n  <main class=\"hero\">\n    <p class=\"eyebrow\">Local XAMPP deployment</p>\n    <h1><?php echo '{title}'; ?></h1>\n    <p>This page is powered by PHP. Edit <code>index.php</code> to get started.</p>\n    <button id=\"hello-button\">Test JavaScript</button>\n  </main>\n  <script src=\"assets/app.js\"></script>\n</body>\n</html>\n"
    )
}

fn laravel_index(name: &str) -> String {
    format!(
        "<?php\n// Laravel-style starter for {name}.\n// In a full Laravel install, index.php lives in public/ and\n// delegates to the framework bootstrap. Use a Composer workflow here.\nrequire __DIR__ . '/../vendor/autoload.php';\n\necho '<h1>{}</h1>';\n",
        title_case(name)
    )
}

fn laravel_env(name: &str) -> String {
    format!(
        "APP_NAME=\"{}\"\nAPP_ENV=local\nAPP_KEY=\nAPP_DEBUG=true\nAPP_URL=http://localhost/{name}/\n\nDB_CONNECTION=mysql\nDB_HOST=127.0.0.1\nDB_PORT=3306\nDB_DATABASE={name}_db\nDB_USERNAME=root\nDB_PASSWORD=\n",
        title_case(name)
    )
}

fn wordpress_config(name: &str) -> String {
    format!(
        "<?php\n// WordPress-style starter for {name}.\n// Copy to wp-config.php once you upload a full WordPress install.\ndefine( 'DB_NAME', '{name}_db' );\ndefine( 'DB_USER', 'root' );\ndefine( 'DB_PASSWORD', '' );\ndefine( 'DB_HOST', '127.0.0.1' );\ndefine( 'DB_CHARSET', 'utf8mb4' );\ndefine( 'DB_COLLATE', '' );\n$table_prefix = 'wp_';\ndefine( 'WP_DEBUG', true );\nif ( ! defined( 'ABSPATH' ) ) {{ define( 'ABSPATH', __DIR__ . '/' ); }}\nrequire_once ABSPATH . 'wp-settings.php';\n"
    )
}

fn wordpress_index() -> String {
    "<?php\n// Short and sweet.\ndefine( 'WP_USE_THEMES', true );\nrequire( __DIR__ . '/wp-blog-header.php' );\n".to_string()
}

fn wordpress_htaccess() -> String {
    "# BEGIN WordPress\nRewriteEngine On\nRewriteBase /\nRewriteRule ^index\\.php$ - [L]\nRewriteCond %{REQUEST_FILENAME} !-f\nRewriteCond %{REQUEST_FILENAME} !-d\nRewriteRule . /index.php [L]\n# END WordPress\n".to_string()
}

fn react_html(title: &str) -> String {
    format!(
        "<!doctype html>\n<html lang=\"en\">\n  <head>\n    <meta charset=\"UTF-8\" />\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />\n    <title>{title}</title>\n  </head>\n  <body>\n    <div id=\"root\"></div>\n    <script type=\"module\" src=\"/src/main.jsx\"></script>\n  </body>\n</html>\n"
    )
}

fn react_main(title: &str) -> String {
    format!(
        "import React from 'react';\nimport ReactDOM from 'react-dom/client';\nimport App from './App';\nimport './App.css';\n\nReactDOM.createRoot(document.getElementById('root')).render(\n  <React.StrictMode>\n    <App title=\"{title}\" />\n  </React.StrictMode>\n);\n"
    )
}

fn react_app() -> String {
    "function App({ title }) {\n  return (\n    <main className=\"hero\">\n      <p className=\"eyebrow\">Local XAMPP deployment</p>\n      <h1>{title}</h1>\n      <p>Run <code>npm install</code> and <code>npm run dev</code> from Xamppify to start this React app.</p>\n    </main>\n  );\n}\n\nexport default App;\n".to_string()
}

fn react_css() -> String {
    "* { box-sizing: border-box; }\nbody { margin: 0; min-height: 100vh; display: grid; place-items: center; background: #0f172a; color: #e2e8f0; font-family: system-ui, sans-serif; }\n.hero { max-width: 42rem; padding: 3rem; border: 1px solid #334155; border-radius: 1rem; background: #172033; }\n.eyebrow { color: #38bdf8; font-weight: 700; text-transform: uppercase; letter-spacing: .08em; font-size: .75rem; }\n".to_string()
}

fn react_vite_config() -> String {
    "import { defineConfig } from 'vite';\nimport react from '@vitejs/plugin-react';\n\nexport default defineConfig({\n  plugins: [react()],\n  server: {\n    port: 5173,\n    strictPort: true,\n  },\n});\n".to_string()
}

fn react_package(name: &str) -> String {
    format!(
        "{{\n  \"name\": \"{name}\",\n  \"private\": true,\n  \"version\": \"0.1.0\",\n  \"type\": \"module\",\n  \"scripts\": {{\n    \"dev\": \"vite\",\n    \"build\": \"vite build\",\n    \"preview\": \"vite preview\"\n  }},\n  \"dependencies\": {{\n    \"react\": \"^18.3.1\",\n    \"react-dom\": \"^18.3.1\"\n  }},\n  \"devDependencies\": {{\n    \"@vitejs/plugin-react\": \"^4.3.4\",\n    \"vite\": \"^5.4.11\"\n  }}\n}}\n"
    )
}

fn node_server(title: &str) -> String {
    format!(
        "const http = require('http');\n\nconst PORT = process.env.PORT || 3000;\n\nconst server = http.createServer((req, res) => {{\n  res.writeHead(200, {{ 'Content-Type': 'text/html; charset=utf-8' }});\n  res.end(`<h1>{title}</h1><p>Node.js server is running on port ${{PORT}}.</p>`);\n}});\n\nserver.listen(PORT, () => {{\n  console.log(`Server running at http://localhost:${{PORT}}`);\n}});\n"
    )
}

fn node_package(name: &str) -> String {
    format!(
        "{{\n  \"name\": \"{name}\",\n  \"version\": \"0.1.0\",\n  \"main\": \"server.js\",\n  \"scripts\": {{\n    \"start\": \"node server.js\",\n    \"dev\": \"node server.js\"\n  }}\n}}\n"
    )
}

fn stylesheet() -> String {
    "* { box-sizing: border-box; }\nbody { margin: 0; min-height: 100vh; display: grid; place-items: center; background: #0f172a; color: #e2e8f0; font-family: system-ui, sans-serif; }\n.hero { max-width: 42rem; padding: 3rem; border: 1px solid #334155; border-radius: 1rem; background: #172033; }\n.eyebrow { color: #38bdf8; font-weight: 700; text-transform: uppercase; letter-spacing: .08em; font-size: .75rem; }\nh1 { margin: .25rem 0 1rem; font-size: clamp(2rem, 8vw, 4rem); }\nbutton { border: 0; border-radius: .5rem; padding: .75rem 1rem; background: #38bdf8; color: #082f49; font-weight: 700; cursor: pointer; }\n".to_string()
}

fn javascript() -> String {
    "document.querySelector('#hello-button')?.addEventListener('click', () => alert('Your deployment is working.'));\n".to_string()
}

/// Detects the framework of an existing project folder.
pub fn detect_framework(root: &std::path::Path) -> String {
    if root.join("wp-config.php").is_file() || root.join("wp-config-sample.php").is_file() {
        return "wordpress".into();
    }
    if root.join("artisan").is_file() && root.join("composer.json").is_file() {
        return "laravel".into();
    }
    if root.join("composer.json").is_file() {
        if let Ok(content) = std::fs::read_to_string(root.join("composer.json")) {
            if content.contains("\"laravel/framework\"") {
                return "laravel".into();
            }
        }
        return "php".into();
    }
    if root.join("vite.config.js").is_file() || root.join("vite.config.ts").is_file() {
        return "react".into();
    }
    if root.join("package.json").is_file() {
        if let Ok(content) = std::fs::read_to_string(root.join("package.json")) {
            if content.contains("\"react\"") {
                return "react".into();
            }
        }
        return "node".into();
    }
    if root.join("index.php").is_file() {
        return "php".into();
    }
    if root.join("index.html").is_file() {
        return "html".into();
    }
    "custom".into()
}
