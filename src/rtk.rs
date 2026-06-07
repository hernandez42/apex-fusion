//! RTK层 - 截断/去重/跨轮dedup
//! 移植自 signal1project/rtk

use regex::Regex;

/// 压缩bash输出（去连续重复行+截断）
pub fn compress_bash_internal(text: &str, max: usize, head: usize, tail: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return text.to_string();
    }

    // 去连续重复行
    let mut deduped: Vec<String> = Vec::new();
    let mut run_count = 1;
    for i in 1..lines.len() {
        if lines[i] == lines[i - 1] {
            run_count += 1;
        } else {
            if run_count > 1 {
                deduped.push(format!("[RTK: previous line repeated {}x]", run_count));
            }
            deduped.push(lines[i - 1].to_string());
            run_count = 1;
        }
    }
    if run_count > 1 {
        deduped.push(format!("[RTK: previous line repeated {}x]", run_count));
    }
    deduped.push(lines.last().unwrap().to_string());

    // 截断头尾
    let total = deduped.len();
    if total <= head + tail {
        return deduped.join("\n");
    }
    let omitted = total - head - tail;
    let head_lines: Vec<&str> = deduped.iter().take(head).map(|s| s.as_str()).collect();
    let tail_lines: Vec<&str> = deduped.iter().rev().take(tail).map(|s| s.as_str()).collect();
    let mut result = head_lines.join("\n");
    result.push_str(&format!("\n[RTK: omitted {} lines]\n", omitted));
    result.push_str(tail_lines.into_iter().rev().collect::<Vec<_>>().join("\n").as_str());
    result
}

/// 压缩bash输出（公开API）
pub fn compress_bash(text: &str) -> String {
    compress_bash_internal(text, 150, 80, 50)
}

/// 通用行截断
pub fn truncate_lines(text: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max {
        return text.to_string();
    }
    let omitted = lines.len() - max;
    let mut result = lines.iter().take(max).map(|s| s.to_string()).collect::<Vec<_>>().join("\n");
    result.push_str(&format!("\n[RTK: omitted {} lines]", omitted));
    result
}

/// 截断read输出
pub fn truncate_read(text: &str, max: usize) -> String {
    truncate_lines(text, max)
}

/// 内部read压缩（带跨轮dedup）
pub fn compress_read_internal(text: &str, session_id: &str, file_path: &str) -> String {
    // 注：跨轮dedup需要外部状态，这里只做截断
    truncate_lines(text, 200)
}

/// read压缩（公开API）
pub fn compress_read(text: &str) -> String {
    truncate_lines(text, 200)
}

/// grep压缩
pub fn compress_grep(text: &str) -> String {
    truncate_lines(text, 80)
}

/// glob压缩
pub fn compress_glob_internal(text: &str, max: usize) -> String {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() <= max {
        return text.to_string();
    }
    let omitted = lines.len() - max;
    let mut result = lines.iter().take(max).map(|s| s.to_string()).collect::<Vec<_>>().join("\n");
    result.push_str(&format!("\n[RTK: omitted {} paths]", omitted));
    result
}

/// glob压缩（公开API）
pub fn compress_glob(text: &str) -> String {
    compress_glob_internal(text, 150)
}

/// web压缩
pub fn compress_web(text: &str) -> String {
    truncate_lines(text, 300)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup() {
        let input = "a\na\na\nb\nc";
        let out = compress_bash_internal(input, 10, 5, 3);
        assert!(out.contains("repeated 3x"));
    }

    #[test]
    fn test_truncate() {
        let lines: Vec<&str> = (0..500).map(|i| format!("line{}", i)).collect();
        let input = lines.join("\n");
        let out = truncate_lines(&input, 200);
        assert!(out.contains("[RTK: omitted"));
    }
}
