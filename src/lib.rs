//! APEX Fusion - RTK + CBM + Headroom + Caveman 四合一压缩引擎
//! 
//! 融合来源：
//! - RTK (signal1project/rtk) - 截断/去重/跨轮dedup
//! - CBM - Context Boundary Marker 防越界标记
//! - Headroom (vishvacyber/Headroom-AI-Context-Compression) - SmartCrusher JSON压缩
//! - Caveman (wilpel/caveman-compression) - 剥语法留事实

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

mod rtk;
mod cbm;
mod smart_crusher;
mod caveman;

pub use rtk::{compress_bash, compress_read, compress_grep, compress_glob, compress_web};
pub use cbm::insert_boundary_markers;
pub use smart_crusher::compress_json;
pub use caveman::compress_text;

/// 融合压缩配置
#[derive(Debug, Clone)]
pub struct FusionConfig {
    /// RTK截断配置
    pub rtk: RtkConfig,
    /// 是否启用CBM标记
    pub enable_cbm: bool,
    /// 是否启用SmartCrusher
    pub enable_smart_crusher: bool,
    /// 是否启用Caveman语法剥离
    pub enable_caveman: bool,
}

#[derive(Debug, Clone)]
pub struct RtkConfig {
    pub read_max: usize,
    pub bash_max: usize,
    pub bash_head: usize,
    pub bash_tail: usize,
    pub grep_max: usize,
    pub glob_max: usize,
    pub web_max: usize,
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            rtk: RtkConfig::default(),
            enable_cbm: true,
            enable_smart_crusher: true,
            enable_caveman: true,
        }
    }
}

impl Default for RtkConfig {
    fn default() -> Self {
        Self {
            read_max: 200,
            bash_max: 150,
            bash_head: 80,
            bash_tail: 50,
            grep_max: 80,
            glob_max: 150,
            web_max: 300,
        }
    }
}

/// 融合压缩结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionResult {
    pub original_len: usize,
    pub compressed_len: usize,
    pub savings_pct: f64,
    pub layers_applied: Vec<String>,
    pub content: String,
}

impl CompressionResult {
    pub fn new(original: &str, compressed: String, layers: Vec<&str>) -> Self {
        let original_len = original.len();
        let compressed_len = compressed.len();
        let savings_pct = if original_len > 0 {
            (original_len - compressed_len) as f64 / original_len as f64 * 100.0
        } else {
            0.0
        };
        Self {
            original_len,
            compressed_len,
            savings_pct,
            layers_applied: layers.into_iter().map(|s| s.to_string()).collect(),
            content: compressed,
        }
    }
}

/// 主融合压缩函数
pub fn fuse_compress(
    content: &str,
    tool_type: &str,
    session_id: &str,
    file_path: Option<&str>,
    config: &FusionConfig,
) -> CompressionResult {
    let original = content;
    let mut layers: Vec<&str> = Vec::new();
    let mut current = content.to_string();

    // Layer 1: RTK 截断+去重
    current = match tool_type {
        "Read" => {
            if let Some(path) = file_path {
                rtk::compress_read_internal(&current, session_id, path)
            } else {
                rtk::truncate_lines(&current, config.rtk.read_max)
            }
        }
        "Bash" => rtk::compress_bash_internal(&current, config.rtk.bash_max, config.rtk.bash_head, config.rtk.bash_tail),
        "Grep" => rtk::truncate_lines(&current, config.rtk.grep_max),
        "Glob" => rtk::compress_glob_internal(&current, config.rtk.glob_max),
        "WebFetch" | "WebSearch" => rtk::truncate_lines(&current, config.rtk.web_max),
        _ => current,
    };
    if current != content {
        layers.push("RTK");
    }

    // Layer 2: CBM 边界标记（仅对文本内容）
    if config.enable_cbm && !current.is_empty() {
        let with_markers = cbm::insert_markers(&current, tool_type);
        if with_markers != current {
            current = with_markers;
            layers.push("CBM");
        }
    }

    // Layer 3: SmartCrusher JSON压缩
    if config.enable_smart_crusher && current.len() > 500 {
        if let Some(json_compressed) = smart_crusher::try_compress_json(&current) {
            if json_compressed.len() < current.len() {
                current = json_compressed;
                layers.push("SmartCrusher");
            }
        }
    }

    // Layer 4: Caveman 语法剥离（仅对长文本）
    if config.enable_caveman && current.len() > 200 {
        let caveman_compressed = caveman::compress(&current);
        if caveman_compressed.len() < current.len() {
            current = caveman_compressed;
            layers.push("Caveman");
        }
    }

    CompressionResult::new(original, current, layers)
}

/// 跨轮去重状态
pub struct SessionState {
    pub reads: HashMap<String, ReadState>,
    pub turn: u32,
}

#[derive(Debug, Clone)]
pub struct ReadState {
    pub hash: String,
    pub turn: u32,
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            reads: HashMap::new(),
            turn: 0,
        }
    }

    pub fn increment_turn(&mut self) {
        self.turn += 1;
    }

    pub fn check_read(&self, session_id: &str, file_path: &str) -> Option<String> {
        let key = format!("{}:{}", session_id, file_path);
        self.reads.get(&key).map(|s| format!("turn {}", s.turn))
    }

    pub fn update_read(&mut self, session_id: &str, file_path: &str, content: &str) {
        let key = format!("{}:{}", session_id, file_path);
        let hash = format!("{:x}", md5::compute(content.as_bytes()));
        self.reads.insert(key, ReadState {
            hash,
            turn: self.turn,
        });
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtk_bash_dedup() {
        let output = "line1\nline1\nline1\nline2\nline3";
        let compressed = rtk::compress_bash_internal(output, 150, 80, 50);
        assert!(compressed.contains("[RTK:"));
    }

    #[test]
    fn test_caveman_compression() {
        let text = "The system is designed to optimize database performance by implementing efficient indexing strategies.";
        let compressed = caveman::compress(text);
        assert!(compressed.len() < text.len());
    }

    #[test]
    fn test_smart_crusher_json() {
        let json = r#"{"items": [{"id": 1, "name": "test"}, {"id": 2, "name": "test"}]}"#;
        if let Some(compressed) = smart_crusher::try_compress_json(json) {
            assert!(compressed.contains("2x"));
        }
    }
}
