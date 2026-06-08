//! APEX Gene Circuit融合层 — RTK压缩引擎 × APEX基因回路自进化
//!
//! 融合来源：
//! - apex-fusion: RTK+CBM+SmartCrusher+Caveman四层压缩
//! - apex gene circuit: outcome→fitness→基因更新回路
//! - KnowledgeSmith: 3层KG压缩策略 + 6类fitness探针
//! - RTDMD: 两阶段KL分解→ΔG驱动
//!
//! 核心思路：
//! 每次压缩结果 → 信号提取 → 基因匹配 → outcome记录 → fitness更新
//! 推理过程本身成为进化过程，实现"零外部管道的内循环自进化"

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// 数据结构
// ============================================================================

/// 单次压缩结果（APEX基因回路的最小反馈单元）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionOutcome {
    pub tool_type: String,
    pub layer: String,
    pub original_len: usize,
    pub compressed_len: usize,
    pub savings_pct: f64,
    pub session_id: String,
    pub turn: u32,
    pub timestamp: u64,
    pub matched_gene: Option<String>,
    pub positive_outcome: bool,
}

impl CompressionOutcome {
    pub fn new(
        tool_type: &str,
        layer: &str,
        original_len: usize,
        compressed_len: usize,
        session_id: &str,
        turn: u32,
    ) -> Self {
        let savings_pct = if original_len > 0 {
            (original_len - compressed_len) as f64 / original_len as f64
        } else {
            0.0
        };
        Self {
            tool_type: tool_type.to_string(),
            layer: layer.to_string(),
            original_len,
            compressed_len,
            savings_pct,
            session_id: session_id.to_string(),
            turn,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            matched_gene: None,
            positive_outcome: savings_pct > 0.3,
        }
    }
}

/// 压缩信号（从结果中提取，用于基因匹配）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSignal {
    pub tool_type: String,
    pub layer: String,
    pub savings_pct: f64,
    pub session_id: String,
    pub turn: u32,
    pub repeated_bash: bool,
    pub json_detected: bool,
    pub long_text: bool,
}

impl CompressionSignal {
    /// 从多次压缩结果提取聚合信号
    pub fn from_outcomes(outcomes: &[CompressionOutcome], session_id: &str) -> Self {
        let by_layer: HashMap<String, f64> = outcomes
            .iter()
            .filter(|o| o.session_id == session_id)
            .fold(HashMap::new(), |mut acc, o| {
                *acc.entry(o.layer.clone()).or_insert(0.0) += o.savings_pct;
                acc
            });

        let mut tool_type = String::new();
        let mut layer = String::new();
        let mut savings_pct = 0.0;
        let mut turn = 0u32;

        for o in outcomes.iter().filter(|o| o.session_id == session_id) {
            tool_type = o.tool_type.clone();
            layer = o.layer.clone();
            savings_pct = o.savings_pct;
            turn = o.turn;
        }

        Self {
            tool_type,
            layer,
            savings_pct,
            session_id: session_id.to_string(),
            turn,
            repeated_bash: by_layer.get("RTK").copied().unwrap_or(0.0) > 0.5,
            json_detected: by_layer.contains_key("SmartCrusher"),
            long_text: by_layer.get("Caveman").copied().unwrap_or(0.0) > 0.3,
        }
    }

    /// 转换为APEX基因信号字符串
    pub fn to_gene_signals(&self) -> Vec<String> {
        let mut signals = vec![
            format!("compression_{}", self.tool_type.to_lowercase()),
            format!("layer_{}", self.layer.to_lowercase()),
            format!("savings_{:.0}", self.savings_pct * 100.0),
        ];
        if self.repeated_bash {
            signals.push("repeated_bash".to_string());
        }
        if self.json_detected {
            signals.push("json_content".to_string());
        }
        if self.long_text {
            signals.push("long_text".to_string());
        }
        if self.savings_pct > 0.8 {
            signals.push("high_compression".to_string());
        }
        signals
    }
}

/// 基因回路状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneCircuitState {
    pub active_genes: HashMap<String, f64>,
    pub update_count: u32,
    pub last_internalization: u64,
    pub routing_share: f64,
}

impl Default for GeneCircuitState {
    fn default() -> Self {
        Self {
            active_genes: HashMap::new(),
            update_count: 0,
            last_internalization: 0,
            routing_share: 0.52,
        }
    }
}

/// APEX MEM 记忆条目（压缩模式持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMemory {
    pub session_id: String,
    pub tool_type: String,
    pub best_layer: String,
    pub avg_savings: f64,
    pub hit_count: u32,
    pub last_seen: u64,
    pub kg_level: KgLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KgLevel {
    Root,
    Intermediate,
    Leaf,
}

impl Default for KgLevel {
    fn default() -> Self {
        KgLevel::Intermediate
    }
}

impl CompressionMemory {
    pub fn new(session_id: &str, tool_type: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            tool_type: tool_type.to_string(),
            best_layer: "RTK".to_string(),
            avg_savings: 0.0,
            hit_count: 0,
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            kg_level: KgLevel::default(),
        }
    }

    pub fn record(&mut self, layer: &str, savings: f64, kg_level: KgLevel) {
        self.hit_count += 1;
        self.avg_savings =
            (self.avg_savings * (self.hit_count - 1) as f64 + savings) / self.hit_count as f64;
        self.last_seen = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if savings > self.avg_savings * 1.1 {
            self.best_layer = layer.to_string();
        }
        self.kg_level = kg_level;
    }
}

// ============================================================================
// 基因回路核心逻辑
// ============================================================================

/// APEX基因回路融合引擎
pub struct ApexGeneCircuit {
    apex_engine_path: String,
    outcome_buffer: HashMap<String, Vec<CompressionOutcome>>,
    memory: HashMap<String, CompressionMemory>,
    state: GeneCircuitState,
}

impl ApexGeneCircuit {
    pub fn new(apex_engine_path: &str) -> Self {
        Self {
            apex_engine_path: apex_engine_path.to_string(),
            outcome_buffer: HashMap::new(),
            memory: HashMap::new(),
            state: GeneCircuitState::default(),
        }
    }

    /// 从压缩结果提取信号并记录outcome
    pub fn record_compression(&mut self, outcome: CompressionOutcome) -> Option<String> {
        let session_id = outcome.session_id.clone();

        self.outcome_buffer
            .entry(session_id.clone())
            .or_default()
            .push(outcome.clone());

        let mem_key = format!("{}:{}", session_id, outcome.tool_type);
        let kg_level = self.infer_kg_level(&outcome);
        if let Some(memory) = self.memory.get_mut(&mem_key) {
            memory.record(&outcome.layer, outcome.savings_pct, kg_level);
        } else {
            let mut memory = CompressionMemory::new(&session_id, &outcome.tool_type);
            memory.record(&outcome.layer, outcome.savings_pct, kg_level);
            self.memory.insert(mem_key, memory);
        }

        let outcomes = self.outcome_buffer.get(&session_id).unwrap();
        let signal = CompressionSignal::from_outcomes(outcomes, &session_id);

        self.gene_match_and_update(&signal, &outcome)
    }

    fn infer_kg_level(&self, outcome: &CompressionOutcome) -> KgLevel {
        match outcome.layer.as_str() {
            "Caveman" if outcome.savings_pct > 0.6 => KgLevel::Root,
            "Caveman" if outcome.savings_pct < 0.3 => KgLevel::Leaf,
            _ => KgLevel::Intermediate,
        }
    }

    fn gene_match_and_update(
        &mut self,
        signal: &CompressionSignal,
        outcome: &CompressionOutcome,
    ) -> Option<String> {
        let signals_str = signal.to_gene_signals().join(",");
        let matched_gene = self.call_apex_gene_match(&signals_str);

        if let Some(ref gene_id) = matched_gene {
            let fitness_delta = self.compute_fitness_delta(outcome);
            self.update_gene_fitness(gene_id, fitness_delta);
            self.state.update_count += 1;

            if outcome.savings_pct > 0.85 && self.state.update_count % 10 == 0 {
                self.internalize_strategy(signal, outcome);
            }
        }

        matched_gene
    }

    fn call_apex_gene_match(&self, signals: &str) -> Option<String> {
        let signals_formatted: String = signals
            .split(',')
            .map(|s| format!("'{}'", s.trim()))
            .collect::<Vec<_>>()
            .join(",");

        let python_code = format!(
            r#"
import sys
sys.path.insert(0, '{}')
try:
    from apex_gene_engine_v3 import ApexGeneEngineV3
    eng = ApexGeneEngineV3()
    hits = eng.match([{}])
    if hits:
        print(hits[0]['gene'].get('id', ''))
    else:
        print('')
except Exception as e:
    print('ERROR:', e)
"#,
            self.apex_engine_path.replace('\\', "\\\\"),
            signals_formatted
        );

        let output = Command::new("python3")
            .arg("-c")
            .arg(&python_code)
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let gene_id = stdout.trim().to_string();
        if gene_id.is_empty() || gene_id.starts_with("ERROR") {
            None
        } else {
            Some(gene_id)
        }
    }

    fn compute_fitness_delta(&self, outcome: &CompressionOutcome) -> f64 {
        let base = outcome.savings_pct * 2.0 - 1.0;
        let sign = if outcome.positive_outcome { 1.0 } else { -1.0 };
        sign * base * 0.1
    }

    fn update_gene_fitness(&mut self, gene_id: &str, delta: f64) {
        let entry = self
            .state
            .active_genes
            .entry(gene_id.to_string())
            .or_insert(0.0);
        *entry = (*entry + delta).clamp(-1.0, 10.0);
    }

    fn internalize_strategy(&mut self, signal: &CompressionSignal, outcome: &CompressionOutcome) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        if now.saturating_sub(self.state.last_internalization) < 60 {
            return;
        }

        let strategy_gene = serde_json::json!({
            "type": "apex_gene",
            "id": format!("apex_compress_{}_{}_{}", outcome.tool_type, outcome.layer, now),
            "category": "compression",
            "signals_match": signal.to_gene_signals(),
            "preconditions": [],
            "strategy": [
                format!("layer={}", outcome.layer),
                format!("savings_pct={:.2}", outcome.savings_pct),
                format!("session={}", outcome.session_id),
            ],
            "constraints": {
                "min_savings": outcome.savings_pct * 0.8,
                "kg_level": format!("{:?}", self.infer_kg_level(outcome)),
            },
            "validation": [],
        });

        self.write_gene_to_apex(&strategy_gene);
        self.state.last_internalization = now;
    }

    fn write_gene_to_apex(&self, gene: &serde_json::Value) {
        let python_code = format!(
            r#"
import sys, json
sys.path.insert(0, '{}')
try:
    from apex_gene_engine_v3 import ApexGeneEngineV3
    eng = ApexGeneEngineV3()
    result = eng.add_gene({}, protect=False)
    print('OK')
except Exception as e:
    print('ERROR:', e)
"#,
            self.apex_engine_path.replace('\\', "\\\\"),
            serde_json::to_string(gene).unwrap_or_default()
        );

        let _ = Command::new("python3").arg("-c").arg(&python_code).output();
    }

    pub fn flush_buffer(&mut self) {
        self.outcome_buffer.clear();
    }

    pub fn get_state(&self) -> GeneCircuitState {
        self.state.clone()
    }

    pub fn get_memory(&self, session_id: &str, tool_type: &str) -> Option<&CompressionMemory> {
        self.memory.get(&format!("{}:{}", session_id, tool_type))
    }

    pub fn get_adaptive_config(
        &self,
        session_id: &str,
        tool_type: &str,
    ) -> Option<AdaptiveConfig> {
        self.get_memory(session_id, tool_type).map(|m| {
            let (rtk_max, caveman_level) = match m.kg_level {
                KgLevel::Root => (100, 1),
                KgLevel::Intermediate => (200, 2),
                KgLevel::Leaf => (300, 3),
            };
            AdaptiveConfig {
                best_layer: m.best_layer.clone(),
                rtk_max,
                caveman_level,
                expected_savings: m.avg_savings,
            }
        })
    }
}

/// 自适应压缩配置（基于APEX MEM历史）
#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    pub best_layer: String,
    pub rtk_max: usize,
    pub caveman_level: u8,
    pub expected_savings: f64,
}

// ============================================================================
// 全局单例 + 便捷函数
// ============================================================================

static GLOBAL_GENE_CIRCUIT: Mutex<Option<ApexGeneCircuit>> = Mutex::new(None);

/// 初始化全局基因回路
pub fn init_gene_circuit(apex_engine_path: &str) {
    let mut guard = GLOBAL_GENE_CIRCUIT.lock().unwrap();
    *guard = Some(ApexGeneCircuit::new(apex_engine_path));
}

/// 从CompressionResult提取CompressionOutcome（tool_type由调用方传入）
pub fn outcome_from_result(
    tool_type: &str,
    result: &crate::CompressionResult,
    session_id: &str,
    turn: u32,
    layer: &str,
) -> CompressionOutcome {
    CompressionOutcome::new(
        tool_type,
        layer,
        result.original_len,
        result.compressed_len,
        session_id,
        turn,
    )
}

/// 全局记录压缩outcome并触发基因回路
pub fn record_and_evolve(outcome: CompressionOutcome) -> Option<String> {
    let mut guard = GLOBAL_GENE_CIRCUIT.lock().unwrap();
    if let Some(ref mut circuit) = *guard {
        circuit.record_compression(outcome)
    } else {
        None
    }
}

/// 获取自适应压缩配置
pub fn get_adaptive_config(session_id: &str, tool_type: &str) -> Option<AdaptiveConfig> {
    let guard = GLOBAL_GENE_CIRCUIT.lock().unwrap();
    guard.as_ref()?.get_adaptive_config(session_id, tool_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_outcome() {
        let o = CompressionOutcome::new("Bash", "RTK", 1000, 200, "sess1", 1);
        assert!((o.savings_pct - 0.8).abs() < 0.001);
        assert!(o.positive_outcome);
    }

    #[test]
    fn test_compression_outcome_low_savings() {
        let o = CompressionOutcome::new("Bash", "RTK", 1000, 850, "sess1", 1);
        assert!((o.savings_pct - 0.15).abs() < 0.001);
        assert!(!o.positive_outcome);
    }

    #[test]
    fn test_compression_signal_repeated_bash() {
        let outcomes = vec![
            CompressionOutcome::new("Bash", "RTK", 1000, 200, "sess1", 1),
            CompressionOutcome::new("Bash", "Caveman", 1000, 700, "sess1", 2),
        ];
        let signal = CompressionSignal::from_outcomes(&outcomes, "sess1");
        assert!(signal.repeated_bash);
    }

    #[test]
    fn test_gene_signals() {
        let signal = CompressionSignal {
            tool_type: "Bash".to_string(),
            layer: "RTK".to_string(),
            savings_pct: 0.85,
            session_id: "sess1".to_string(),
            turn: 1,
            repeated_bash: true,
            json_detected: false,
            long_text: false,
        };
        let signals = signal.to_gene_signals();
        assert!(signals.contains(&"compression_bash".to_string()));
        assert!(signals.contains(&"high_compression".to_string()));
        assert!(signals.contains(&"repeated_bash".to_string()));
    }

    #[test]
    fn test_adaptive_config_root() {
        let mut mem = CompressionMemory::new("s1", "Bash");
        mem.record("Caveman", 0.75, KgLevel::Root);
        assert_eq!(mem.kg_level, KgLevel::Root);
        assert!(mem.avg_savings > 0.7);
    }

    #[test]
    fn test_fitness_delta_positive() {
        let circuit = ApexGeneCircuit::new("/tmp");
        let outcome = CompressionOutcome::new("Bash", "RTK", 1000, 100, "sess1", 1);
        let delta = circuit.compute_fitness_delta(&outcome);
        assert!(delta > 0.0);
    }

    #[test]
    fn test_fitness_delta_negative() {
        let circuit = ApexGeneCircuit::new("/tmp");
        let outcome = CompressionOutcome::new("Bash", "RTK", 1000, 900, "sess1", 1);
        let delta = circuit.compute_fitness_delta(&outcome);
        assert!(delta < 0.0);
    }
}
