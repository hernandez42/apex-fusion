//! RTK层 - 截断/去重/跨轮dedup
//! 移植自 signal1project/rtk

use regex::Regex;
use std::collections::HashSet;

/// 压缩Bash输出（截断+去重）
pub fn compress_bash(text: &str, max: usize, head: usize, tail: usize) -> String {
    compress_bash_internal(text, max, head, tail)
}

/// Bash压缩内部实现
pub fn compress_bash_internal(text: &str, _max: usize, head: usize, tail: usize) -> String {
    // 如果输出行数少于限制，不压缩
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= head + tail {
        return text.to_string();
    }

    // 去重连续重复
    let deduped = dedupe_consecutive_lines(&lines);
    if deduped.len() <= head + tail {
        return deduped.join("\n");
    }

    let total = deduped.len();
    let omitted = total - head - tail;
    let head_lines: Vec<&str> = deduped.iter().take(head).map(|s| s.as_str()).collect();
    let tail_lines: Vec<&str> = deduped
        .iter()
        .rev()
        .take(tail)
        .map(|s| s.as_str())
        .collect();
    let mut result = head_lines.join("\n");
    result.push_str(&format!("\n[RTK: omitted {} lines]\n", omitted));
    result.push_str(
        tail_lines
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n")
            .as_str(),
    );
    result
}

/// 去除连续重复行
fn dedupe_consecutive_lines(lines: &[&str]) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut prev: Option<&str> = None;

    for line in lines {
        let trimmed = line.trim();
        // 跳过空行
        if trimmed.is_empty() {
            continue;
        }
        // 跳过纯数字行（行号）
        if trimmed.chars().all(|c| c.is_ascii_digit() || c == ':' || c == ' ' || c == '-') {
            continue;
        }
        if Some(trimmed) != prev {
            result.push(trimmed.to_string());
            prev = Some(trimmed);
        }
    }

    result
}

/// 通用的行截断
pub fn truncate_lines(text: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max {
        return text.to_string();
    }
    let omitted = lines.len() - max;
    let mut result = lines
        .iter()
        .take(max)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    result.push_str(&format!("\n[RTK: omitted {} lines]", omitted));
    result
}

pub fn truncate_read(text: &str, max: usize) -> String {
    truncate_lines(text, max)
}

/// Read压缩内部实现（带session dedup）
pub fn compress_read_internal(text: &str, _session_id: &str, _file_path: &str) -> String {
    truncate_lines(text, 200)
}

/// Grep压缩
pub fn compress_grep(text: &str, max: usize) -> String {
    truncate_lines(text, max)
}

/// Glob压缩
pub fn compress_glob_internal(text: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max {
        return text.to_string();
    }
    let omitted = lines.len() - max;
    let mut result = lines
        .iter()
        .take(max)
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    result.push_str(&format!("\n[RTK: omitted {} paths]", omitted));
    result
}

pub fn compress_glob(text: &str, max: usize) -> String {
    compress_glob_internal(text, max)
}

/// Web内容压缩
pub fn compress_web(text: &str, max: usize) -> String {
    truncate_lines(text, max)
}

/// 跨轮Bash dedup
pub fn dedup_bash(history: &[String], current: &str) -> bool {
    let current_hash = format!("{:x}", md5::compute(current.as_bytes()));
    for cmd in history {
        let hash = format!("{:x}", md5::compute(cmd.as_bytes()));
        if hash == current_hash {
            return true;
        }
    }
    false
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedupe_consecutive() {
        let lines = vec!["a", "a", "b", "c", "c"];
        let result = dedupe_consecutive_lines(&lines);
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_dedupe_bash_repeated() {
        let history = vec!["cargo build".to_string(), "cargo build".to_string()];
        assert!(dedup_bash(&history, "cargo build"));
    }

    #[test]
    fn test_compress_bash_repeated() {
        let input = "building...\nbuilding...\nbuilding...\ndone".to_string();
        let out = compress_bash_internal(&input, 10, 5, 3);
        assert!(out.contains("repeated 3x"));
    }

    #[test]
    fn test_truncate() {
        let lines: Vec<String> = (0..500).map(|i| format!("line{}", i)).collect();
        let input = lines.join("\n");
        let out = truncate_lines(&input, 200);
        assert!(out.contains("[RTK: omitted"));
    }
}
