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
            let lower = line.to_lowercase();
            let (level, message) = if lower.contains("[error]") || lower.contains("error") {
                ("ERROR", line.to_string())
            } else if lower.contains("[warn]") || lower.contains(":warn]") || lower.contains("warning") {
                ("WARN", line.to_string())
            } else if lower.contains("[info]") {
                ("INFO", line.to_string())
            } else if lower.contains("[debug]") {
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
            let lower = line.to_lowercase();
            let level = if lower.contains("[error]") || lower.contains("error") {
                "ERROR"
            } else if lower.contains("[warn]") || lower.contains(":warn]") || lower.contains("warning") {
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

#[cfg(test)]
mod tests {
    use super::{extract_timestamp, parse_apache_log, parse_mysql_log};

    const APACHE_LINES: &str = "[Sat Jun 01 12:00:00.123456 2026] [core:error] [pid 1234] boom\n\
     [Sat Jun 01 12:00:05.654321 2026] [core:info] [pid 1234] all good\n\
     [Sat Jun 01 12:00:10.111222 2026] [core:warn] [pid 1234] watch out\n\
     plain line without a timestamp\n";

    #[test]
    fn detects_apache_levels() {
        let parsed = parse_apache_log(APACHE_LINES, None);
        assert_eq!(parsed[0].level, "ERROR");
        assert_eq!(parsed[1].level, "INFO");
        assert_eq!(parsed[2].level, "WARN");
        assert_eq!(parsed[3].source, "Apache");
    }

    #[test]
    fn apache_timestamp_uses_bracket_prefix() {
        let parsed = parse_apache_log(APACHE_LINES, None);
        assert_eq!(parsed[0].timestamp, "Sat Jun 01 12:00:00");
        assert_eq!(parsed[3].timestamp, "");
    }

    #[test]
    fn limits_apache_lines_to_newest() {
        let parsed = parse_apache_log(APACHE_LINES, Some(2));
        assert_eq!(parsed.len(), 2);
        assert!(parsed[0].message.contains("12:00:10"));
    }

    #[test]
    fn mysql_log_levels_and_source() {
        let parsed = parse_mysql_log("2026-06-01 12:00:00 [ERROR] InnoDB crashed\n2026-06-01 12:00:01 [Note] started\n2026-06-01 12:00:02 [Warning] slow query\n", None);
        assert_eq!(parsed[0].level, "ERROR");
        assert_eq!(parsed[1].level, "INFO");
        assert_eq!(parsed[2].level, "WARN");
        assert!(parsed.iter().all(|line| line.source == "MySQL"));
    }

    #[test]
    fn extract_timestamp_handles_formats() {
        assert_eq!(extract_timestamp("[Sat Jun 01 12:00:00.123456 2026] x"), Some("Sat Jun 01 12:00:00".into()));
        assert_eq!(extract_timestamp("2026-06-01 12:00:00 x"), Some("2026-06-01 12:00:00 ".into()));
        assert_eq!(extract_timestamp("short"), None);
    }
}
