//! SmartCrusher层 - JSON结构化压缩

use regex::Regex;

pub fn try_compress_json(content: &str) -> Option<String> {
    let trimmed = content.trim();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') { return None; }
    let parsed: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    let compressed = compress_json_value(&parsed)?;
    let result = serde_json::to_string(&compressed).ok()?;
    if result.len() < trimmed.len() { Some(result) } else { None }
}

pub fn compress_json_value(value: &serde_json::Value) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                if let Some(cv) = compress_json_value(v) { new_obj.insert(k.clone(), cv); }
            }
            Some(serde_json::Value::Object(new_obj))
        }
        serde_json::Value::Array(arr) => {
            let compressed: Vec<serde_json::Value> = arr.iter().filter_map(|v| compress_json_value(v)).collect();
            let mut result: Vec<serde_json::Value> = Vec::new();
            let mut i = 0;
            while i < compressed.len() {
                let mut count = 1;
                while i + count < compressed.len() && compressed[i] == compressed[i + count] { count += 1; }
                if count > 1 {
                    result.push(serde_json::json!({"_repeat": count, "item": compressed[i]}));
                    i += count;
                } else { result.push(compressed[i].clone()); i += 1; }
            }
            if result.len() < compressed.len() { Some(serde_json::Value::Array(result)) }
            else { Some(serde_json::Value::Array(compressed)) }
        }
        serde_json::Value::String(s) => {
            if s.len() < 50 { Some(serde_json::Value::String(s.clone())) }
            else {
                let compressed = compress_whitespace(s);
                if compressed.len() < s.len() { Some(serde_json::Value::String(compressed)) }
                else { Some(serde_json::Value::String(s.clone())) }
            }
        }
        _ => Some(value.clone()),
    }
}

fn compress_whitespace(s: &str) -> String {
    let ws_re = Regex::new(r"\s+").unwrap();
    ws_re.replace_all(s, " ").to_string()
}
pub fn compress_json(content: &str) -> String { try_compress_json(content).unwrap_or_else(|| content.to_string()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_json_repeat_detection() { assert!(try_compress_json(r#"[1,1,1,2,3,3]"#).is_some()); }
    #[test] fn test_non_json() { assert!(try_compress_json("this is not json").is_none()); }
    #[test] fn test_whitespace_compression() { assert!(compress_whitespace("hello    world\n\n\nfoo").len() < 20); }
}
