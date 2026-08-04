use serde::Serialize;
use std::path::Path;

const MAX_RESULTS: usize = 300;
const MAX_FILE_BYTES: u64 = 10 * 1024 * 1024;
const SKIP_DIRS: [&str; 4] = [".git", "node_modules", "vendor", "dist"];

#[derive(Debug, Clone, Serialize)]
pub struct SearchMatch {
    pub path: String,
    pub line_number: u64,
    pub line: String,
}

pub async fn search_htdocs(query: String, literal: bool) -> Result<Vec<SearchMatch>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    let pattern = if literal {
        regex::escape(query)
    } else {
        query.to_string()
    };
    let matcher = grep::regex::RegexMatcherBuilder::new()
        .case_smart(true)
        .build(&pattern)
        .map_err(|e| format!("Invalid search pattern: {e}"))?;

    let root = crate::paths::canonical_xampp_root()?.join("htdocs");
    if !root.is_dir() {
        return Err("The XAMPP htdocs folder is not available".to_string());
    }

    let root_for_task = root.clone();
    tokio::task::spawn_blocking(move || walk(&root_for_task, &matcher))
        .await
        .map_err(|e| format!("Search task failed: {e}"))?
}

fn walk(dir: &Path, matcher: &grep::regex::RegexMatcher) -> Result<Vec<SearchMatch>, String> {
    let mut results = Vec::new();
    walk_recursive(dir, matcher, &mut results)?;
    Ok(results)
}

fn walk_recursive(
    dir: &Path,
    matcher: &grep::regex::RegexMatcher,
    results: &mut Vec<SearchMatch>,
) -> Result<(), String> {
    if results.len() >= MAX_RESULTS {
        return Ok(());
    }
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read {}: {e}", dir.display()))?;
    for entry in entries {
        if results.len() >= MAX_RESULTS {
            return Ok(());
        }
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {e}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to inspect {}: {e}", path.display()))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            walk_recursive(&path, matcher, results)?;
        } else if file_type.is_file() {
            let len = entry
                .metadata()
                .map_err(|e| format!("Failed to inspect {}: {e}", path.display()))?
                .len();
            if len == 0 || len > MAX_FILE_BYTES {
                continue;
            }
            let file = std::fs::File::open(&path)
                .map_err(|e| format!("Failed to open {}: {e}", path.display()))?;
            let mut searcher = grep::searcher::Searcher::new();
            let mut sink = CollectSink {
                path: path.to_string_lossy().to_string(),
                results,
            };
            searcher
                .search_reader(matcher, file, &mut sink)
                .map_err(|e| format!("Failed to search {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

struct CollectSink<'a> {
    path: String,
    results: &'a mut Vec<SearchMatch>,
}

#[derive(Debug)]
struct SearchSinkError(String);

impl std::fmt::Display for SearchSinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl grep::searcher::SinkError for SearchSinkError {
    fn error_message<T: std::fmt::Display>(message: T) -> Self {
        SearchSinkError(message.to_string())
    }
}

impl<'a> grep::searcher::Sink for CollectSink<'a> {
    type Error = SearchSinkError;

    fn matched(
        &mut self,
        _searcher: &grep::searcher::Searcher,
        mat: &grep::searcher::SinkMatch<'_>,
    ) -> Result<bool, Self::Error> {
        self.results.push(SearchMatch {
            path: self.path.clone(),
            line_number: mat.line_number().unwrap_or(0),
            line: String::from_utf8_lossy(mat.bytes()).trim_end().to_string(),
        });
        Ok(self.results.len() < MAX_RESULTS)
    }
}
