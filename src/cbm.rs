//! CBM层 - Context Boundary Marker 防越界

pub fn insert_markers(content: &str, tool_type: &str) -> String {
    let marker_start = format!("[CBM:{} START]", tool_type);
    let marker_end = format!("[CBM:{} END]", tool_type);
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 10 { return content.to_string(); }
    let (head_count, tail_count) = (3, 3);
    let total = lines.len();
    if total <= head_count + tail_count + 1 { return content.to_string(); }
    let head = &lines[..head_count];
    let tail = &lines[total - tail_count..];
    let mut result = head.join("\n");
    result.push_str(&format!("\n{}\n[CBM: {} lines truncated, {} total]\n{}", marker_start, total - head_count - tail_count, total, marker_end));
    result.push('\n');
    result.push_str(&tail.join("\n"));
    result
}
pub fn insert_boundary_markers(content: &str, tool_type: &str) -> String { insert_markers(content, tool_type) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_marker_insertion() {
        let lines: Vec<String> = (0..50).map(|i| format!("line{}", i)).collect();
        let result = insert_markers(&lines.join("\n"), "Read");
        assert!(result.contains("[CBM:Read START]"));
        assert!(result.contains("[CBM:Read END]"));
    }
    #[test] fn test_short_content() {
        let result = insert_markers("short\ncontent", "Bash");
        assert_eq!(result, "short\ncontent");
    }
}
