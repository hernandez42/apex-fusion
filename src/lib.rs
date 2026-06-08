//! APEX Fusion - RTK + CBM + Headroom + Caveman + GeneCircuit 五合一压缩引擎
//!
//! 融合来源：
//! - RTK (signal1project/rtk) - 截断/去重/跨轮dedup
//! - CBM - Context Boundary Marker 防越界标记
//! - Headroom (vishvacyber/Headroom-AI-Context-Compression) - SmartCrusher JSON压缩
//! - Caveman (wilpel/caveman-compression) - 剥语法留事实
//! - APEX GeneCircuit - 基因回路自进化闭环（KnowledgeSmith + RTDMD融合）

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

mod rtk;
mod cbm;
mod smart_crusher;
mod caveman;
pub mod gene_circuit;

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
        "Bash" => rtk::compress_bash_internal(
            &current,
            config.rtk.bash_max,
            config.rtk.bash_head,
            config.rtk.bash_tail,
        ),
        "Grep" => rtk::truncate_lines(&current, config.rtk.grep_max),
        "Glob" => rtk::compress_glob_internal(&current, config.rtk.glob_max),
        "WebFetch" | "WebSearch" => rtk::truncate_lines(&current, config.rtk.web_max),
        _ => current,
    };
    if current != content {
        layers.push("RTK");
    }

    // Layer 2: CBM 边界标记
    if config.enable_cbm && current.len() > 200 {
        let marked = cbm::insert_boundary_markers(&current, tool_type);
        if marked != current {
            current = marked;
            layers.push("CBM");
        }
    }

    // Layer 3: SmartCrusher JSON压缩
    if config.enable_smart_crusher {
        let compressed = smart_crusher::compress_json(&current);
        if compressed != current {
            current = compressed;
            layers.push("SmartCrusher");
        }
    }

    // Layer 4: Caveman 语义压缩
    if config.enable_caveman && current.len() > 500 {
        let compressed = caveman::compress_text(&current, 2);
        if compressed != current && compressed.len() < current.len() {
            current = compressed;
            layers.push("Caveman");
        }
    }

    CompressionResult::new(original, current, layers)
}

/// 会话状态（跨轮去重用）
#[derive(Debug, Default)]
pub struct SessionState {
    pub turn: u32,
    pub reads: HashMap<String, ReadState>,
    pub bash_hashes: HashMap<String, BashState>,
}

#[derive(Debug)]
pub struct ReadState {
    pub hash: String,
    pub turn: u32,
}

#[derive(Debug)]
pub struct BashState {
    pub hash: String,
    pub turn: u32,
}

impl SessionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advance_turn(&mut self) {
        self.turn += 1;
    }

    pub fn update_bash(&mut self, command: &str) -> bool {
        let hash = format!("{:x}", md5::compute(command.as_bytes()));
        if let Some(state) = self.bash_hashes.get(&hash) {
            if state.turn < self.turn {
                self.bash_hashes.insert(
                    hash,
                    BashState {
                        hash,
                        turn: self.turn,
                    },
                );
                return true;
            }
            return false;
        }
        self.bash_hashes.insert(
            hash,
            BashState {
                hash,
                turn: self.turn,
            },
        );
        true
    }

    pub fn update_read(&mut self, session_id: &str, file_path: &str, content: &str) {
        let key = format!("{}:{}", session_id, file_path);
        let hash = format!("{:x}", md5::compute(content.as_bytes()));
        self.reads.insert(
            key,
            ReadState {
                hash,
                turn: self.turn,
            },
        );
    }
}

// ============================================================================
// CLI入口
// ============================================================================

use std::io::{self, Read, Write};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let trimmed = input.trim();

    // 检测工具类型（从环境变量或内容推断）
    let tool_type = std::env::var("TOOL_TYPE").unwrap_or_else(|_| {
        if trimmed.contains("cargo") || trimmed.contains("npm") || trimmed.contains("pip") {
            "Bash".to_string()
        } else if trimmed.contains("fn main") || trimmed.contains("pub fn") || trimmed.ends_with(".rs") {
            "Read".to_string()
        } else {
            "Unknown".to_string()
        }
    });

    let session_id = std::env::var("SESSION_ID").unwrap_or_else(|_| "default".to_string());

    let result = fuse_compress(
        trimmed,
        &tool_type,
        &session_id,
        None,
        &FusionConfig::default(),
    );

    // 输出原始结果（单行JSON）
    let output = serde_json::json!({
        "original_len": result.original_len,
        "compressed_len": result.compressed_len,
        "savings_pct": result.savings_pct,
        "layers": result.layers_applied,
        "compressed": result.content,
    });

    println!("{}", output);
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fusion_basic() {
        let input = "line1\nline2\nline3\n";
        let result = fuse_compress(input, "Bash", "test", None, &FusionConfig::default());
        assert!(result.original_len > result.compressed_len);
    }

    #[test]
    fn test_smart_crusher_json() {
        let input = r#"{"a":1,"a":1,"a":1,"b":2}"#;
        let result = fuse_compress(input, "Bash", "test", None, &FusionConfig::default());
        assert!(result.layers_applied.contains(&"SmartCrusher".to_string()));
    }
}
