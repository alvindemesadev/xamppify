use crate::LogLine;

pub fn parse_apache_log(content: &str, max_lines: Option<usize>) -> Vec<LogLine> {
    let lines: Vec<&str> = if let Some(max) = max_lines {
        content
            .lines()
            .rev()
            .take(max)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    } else {
        content.lines().collect()
    };

    lines
        .iter()
        .map(|line| {
            let (level, message) = if line.contains("[error]") {
                ("ERROR", line.to_string())
            } else if line.contains("[warn]") {
                ("WARN", line.to_string())
            } else if line.contains("[info]") {
                ("INFO", line.to_string())
            } else if line.contains("[debug]") {
                ("DEBUG", line.to_string())
            } else {
                ("INFO", line.to_string())
            };

            let timestamp = extract_timestamp(line).unwrap_or_default();

            LogLine {
                timestamp,
                level: level.to_string(),
                message,
                source: "Apache".to_string(),
            }
        })
        .collect()
}

pub fn parse_mysql_log(content: &str, max_lines: Option<usize>) -> Vec<LogLine> {
    let lines: Vec<&str> = if let Some(max) = max_lines {
        content
            .lines()
            .rev()
            .take(max)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    } else {
        content.lines().collect()
    };

    lines
        .iter()
        .map(|line| {
            let level = if line.contains("[ERROR]") || line.contains("Error") {
                "ERROR"
            } else if line.contains("[Warning]") || line.contains("Warning") {
                "WARN"
            } else {
                "INFO"
            };

            let timestamp = extract_timestamp(line).unwrap_or_default();

            LogLine {
                timestamp,
                level: level.to_string(),
                message: line.to_string(),
                source: "MySQL".to_string(),
            }
        })
        .collect()
}

fn extract_timestamp(line: &str) -> Option<String> {
    if line.len() > 20 {
        let candidate = &line[..20];
        if candidate.contains(':') || candidate.contains('-') {
            return Some(
                candidate
                    .trim_end_matches(']')
                    .trim_start_matches('[')
                    .to_string(),
            );
        }
    }
    None
}
