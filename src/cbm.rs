//! CBM层 - Context Boundary Marker
//! 防止压缩内容越界污染上下文

use regex::Regex;

/// 插入边界标记
pub fn insert_markers(content: &str, tool_type: &str) -> String {
    let marker_start = format!("[CBM:{} START]", tool_type);
    let marker_end = format!("[CBM:{} END]", tool_type);
    
    // 对于长内容，在关键位置插入标记
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 10 {
        return content.to_string();
    }

    // 保留头尾，中间插入标记
    let head_count = 3;
    let tail_count = 3;
    let total = lines.len();
    
    if total <= head_count + tail_count + 1 {
        return content.to_string();
    }

    let head = &lines[..head_count];
    let tail = &lines[total - tail_count..];
    
    let mut result = head.join("\n");
    result.push_str(&format!("\n{}\n", marker_start));
    result.push_str(&format!("[CBM: {} lines truncated, {} total]\n", total - head_count - tail_count, total));
    result.push_str(&marker_end);
    result.push('\n');
    result.push_str(&tail.join("\n"));
    
    result
}

/// 插入边界标记（公开API）
pub fn insert_boundary_markers(content: &str, tool_type: &str) -> String {
    insert_markers(content, tool_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marker_insertion() {
        let lines: Vec<String> = (0..50).map(|i| format!("line{}", i)).collect();
        let content = lines.join("\n");
        let result = insert_markers(&content, "Read");
        assert!(result.contains("[CBM:Read START]"));
        assert!(result.contains("[CBM:Read END]"));
    }

    #[test]
    fn test_short_content() {
        let content = "short\ncontent";
        let result = insert_markers(content, "Bash");
        assert_eq!(result, content);
    }
}
