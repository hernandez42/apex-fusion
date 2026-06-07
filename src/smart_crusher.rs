//! SmartCrusher层 - JSON结构化压缩
//! 移植自 Headroom 的 SmartCrusher 策略
//! 
//! 核心思路：
//! - 检测JSON结构
//! - 识别重复键值
//! - 数组元素重复用 "Nx" 标记
//! - 嵌套对象展开后重新紧凑

use regex::Regex;

/// 尝试将内容识别为JSON并压缩
pub fn try_compress_json(content: &str) -> Option<String> {
    let trimmed = content.trim();
    
    // 快速检查是否是JSON（以{或[开头）
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        return None;
    }

    // 尝试解析
    let parsed: serde_json::Value = serde_json::from_str(trimmed).ok()?;
    
    // 压缩JSON
    let compressed = compress_json_value(&parsed)?;
    let result = serde_json::to_string(&compressed).ok()?;
    
    // 只有当压缩后更短时才返回
    if result.len() < trimmed.len() {
        Some(result)
    } else {
        None
    }
}

/// 压缩JSON值
pub fn compress_json_value(value: &serde_json::Value) -> Option<serde_json::Value> {
    match value {
        serde_json::Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            for (k, v) in obj {
                if let Some(cv) = compress_json_value(v) {
                    new_obj.insert(k.clone(), cv);
                }
            }
            Some(serde_json::Value::Object(new_obj))
        }
        serde_json::Value::Array(arr) => {
            // 检测重复元素
            let compressed: Vec<serde_json::Value> = arr.iter().filter_map(|v| compress_json_value(v)).collect();
            
            // 寻找连续重复
            let mut result: Vec<serde_json::Value> = Vec::new();
            let mut i = 0;
            while i < compressed.len() {
                let mut count = 1;
                while i + count < compressed.len() && compressed[i] == compressed[i + count] {
                    count += 1;
                }
                if count > 1 {
                    // 压缩重复：用标记替代
                    let rep = serde_json::json!({"_repeat": count, "item": compressed[i]});
                    result.push(rep);
                    i += count;
                } else {
                    result.push(compressed[i].clone());
                    i += 1;
                }
            }
            
            // 如果结果明显更短，返回压缩版
            if result.len() < compressed.len() {
                Some(serde_json::Value::Array(result))
            } else {
                Some(serde_json::Value::Array(compressed))
            }
        }
        serde_json::Value::String(s) => {
            // 短字符串不压缩
            if s.len() < 50 {
                Some(serde_json::Value::String(s.clone()))
            } else {
                // 长字符串压缩空白
                let compressed = compress_whitespace(s);
                if compressed.len() < s.len() {
                    Some(serde_json::Value::String(compressed))
                } else {
                    Some(serde_json::Value::String(s.clone()))
                }
            }
        }
        _ => Some(value.clone()),
    }
}

/// 压缩空白字符（保持语义）
fn compress_whitespace(s: &str) -> String {
    let ws_re = Regex::new(r"\s+").unwrap();
    ws_re.replace_all(s, " ").to_string()
}

/// 压缩JSON（公开API）
pub fn compress_json(content: &str) -> String {
    try_compress_json(content).unwrap_or_else(|| content.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_repeat_detection() {
        let json = r#"[1,1,1,2,3,3]"#;
        let result = try_compress_json(json);
        assert!(result.is_some());
    }

    #[test]
    fn test_non_json() {
        let text = "this is not json";
        assert!(try_compress_json(text).is_none());
    }

    #[test]
    fn test_whitespace_compression() {
        let s = "hello    world\n\n\nfoo";
        let compressed = compress_whitespace(s);
        assert!(compressed.len() < s.len());
    }
}
