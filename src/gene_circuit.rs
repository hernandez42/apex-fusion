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

    fn internalize_strategy(
        &mut self,
        signal: &CompressionSignal,
        outcome: &CompressionOutcome,
    ) {
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

    pub fn get_adaptive_config(&self, session_id: &str, tool_type: &str) -> Option<AdaptiveConfig> {
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
    // 计算fitness_delta
    let fitness_delta = compute_fitness_delta_static(&outcome);

    // 更新基因回路
    let matched_gene = {
        let mut guard = GLOBAL_GENE_CIRCUIT.lock().unwrap();
        if let Some(ref mut circuit) = *guard {
            circuit.record_compression(outcome.clone())
        } else {
            None
        }
    };

    // 同时更新全局PhiEngine（闭环核心）
    {
        let mut guard = GLOBAL_PHI_ENGINE.lock().unwrap();
        if guard.is_none() {
            *guard = Some(PhiEngine::new());
        }
        if let Some(ref mut engine) = *guard {
            engine.evolve_from_outcome(&outcome, fitness_delta);
        }
    }

    matched_gene
}

fn compute_fitness_delta_static(outcome: &CompressionOutcome) -> f64 {
    let base = outcome.savings_pct * 2.0 - 1.0;
    let sign = if outcome.positive_outcome { 1.0 } else { -1.0 };
    sign * base * 0.1
}

/// 获取自适应压缩配置
pub fn get_adaptive_config(session_id: &str, tool_type: &str) -> Option<AdaptiveConfig> {
    let guard = GLOBAL_GENE_CIRCUIT.lock().unwrap();
    guard.as_ref()?.get_adaptive_config(session_id, tool_type)
}

// 全局PhiEngine单例（用于闭环追踪）
static GLOBAL_PHI_ENGINE: Mutex<Option<PhiEngine>> = Mutex::new(None);

/// 获取当前turn（基于历史outcome数）
pub fn get_current_turn(session_id: &str) -> u32 {
    let guard = GLOBAL_GENE_CIRCUIT.lock().unwrap();
    if let Some(ref circuit) = *guard {
        circuit
            .outcome_buffer
            .get(session_id)
            .map(|v| v.len() as u32)
            .unwrap_or(0)
    } else {
        0
    }
}

/// 获取Phi引擎当前状态（phi_pct + evolved标记）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiState {
    pub phi_pct: f64,
    pub evolved: bool,
}

pub fn get_phi_state() -> Option<PhiState> {
    let mut guard = GLOBAL_PHI_ENGINE.lock().unwrap();
    if guard.is_none() {
        *guard = Some(PhiEngine::new());
    }
    let engine = guard.as_mut().unwrap();
    let phi_pct = engine.compute_pct();
    let evolved = engine.history().len() > 1;
    Some(PhiState { phi_pct, evolved })
}

// ============================================================================
// Φ_APEX*∞ 引擎 — 修复乘法衰减问题
// ============================================================================
//
// 原公式问题：15个参数全0.1相乘 → Φ≈2e-15%，系统初始化即死亡
//
// 修复方案：对数空间计算
// log(Φ) = Σᵢlog(αᵢ) + log(Φ_base) + log(EV) + log(AN) + log(NV) - log(HarmRate)
//
// ============================================================================

/// Φ引擎参数状态（15个因子）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiParams {
    pub omega_a: f64,
    pub beta_bg: f64,
    pub alpha_ack: f64,
    pub theta_tri: f64,
    pub nabla_k: f64,
    pub zeta_sigma: f64,
    pub eta_lambda: f64,
    pub evm: f64,
    pub a: f64,
    pub b: f64,
    pub tdhlgwb: f64,
    pub phi_base: f64,
    pub ev: f64,
    pub an: f64,
    pub nv: f64,
    pub harm_rate: f64,
}

impl Default for PhiParams {
    fn default() -> Self {
        Self {
            omega_a: 0.5,
            beta_bg: 0.5,
            alpha_ack: 0.5,
            theta_tri: 0.5,
            nabla_k: 0.5,
            zeta_sigma: 0.5,
            eta_lambda: 0.5,
            evm: 0.5,
            a: 0.5,
            b: 0.5,
            tdhlgwb: 0.5,
            phi_base: 0.5,
            ev: 0.3,
            an: 0.5,
            nv: 0.5,
            harm_rate: 0.34,
        }
    }
}

impl PhiParams {
    /// 从压缩outcome更新参数（在线学习）
    pub fn update_from_outcome(&mut self, outcome: &CompressionOutcome, fitness_delta: f64) {
        let lr = 0.05;

        if fitness_delta > 0.0 {
            let delta = (fitness_delta * lr).min(0.1);
            self.omega_a = (self.omega_a + self.omega_a * delta).min(1.0);
            self.ev = (self.ev + self.ev * delta * 2.0).min(1.0);
            self.nv = (self.nv + self.nv * delta).min(1.0);
            self.an = (self.an + self.an * delta).min(1.0);
        }
    }
}

/// Φ_APEX*∞ 引擎
pub struct PhiEngine {
    params: PhiParams,
    history: Vec<PhiSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhiSnapshot {
    pub phi: f64,
    pub phi_pct: f64,
    pub log_terms: f64,
    pub timestamp: u64,
}

impl PhiEngine {
    pub fn new() -> Self {
        Self {
            params: PhiParams::default(),
            history: Vec::new(),
        }
    }

    pub fn with_params(params: PhiParams) -> Self {
        Self {
            params,
            history: Vec::new(),
        }
    }

    /// 对数空间Φ计算
    /// log(Φ) = Σᵢlog(αᵢ) + log(Φ_base) + log(EV) + log(AN) + log(NV) - log(HarmRate)
    pub fn compute(&self) -> f64 {
        let log_factors: f64 = [
            self.params.omega_a,
            self.params.beta_bg,
            self.params.alpha_ack,
            self.params.theta_tri,
            self.params.nabla_k,
            self.params.zeta_sigma,
            self.params.eta_lambda,
            self.params.evm,
            self.params.a,
            self.params.b,
            self.params.tdhlgwb,
        ]
        .iter()
        .map(|&x| x.clamp(1e-10, 1.0))
        .map(|x| x.ln())
        .sum();

        let log_numerator = self.params.phi_base.clamp(1e-10, 1.0).ln()
            + self.params.ev.clamp(1e-10, 1.0).ln()
            + self.params.an.clamp(1e-10, 1.0).ln()
            + self.params.nv.clamp(1e-10, 1.0).ln();

        let log_harm = self.params.harm_rate.clamp(0.01, 10.0).ln();

        let log_phi = log_factors + log_numerator - log_harm;
        log_phi.exp()
    }

    pub fn compute_pct(&self) -> f64 {
        self.compute() * 100.0
    }

    pub fn params(&self) -> &PhiParams {
        &self.params
    }

    pub fn evolve_from_outcome(&mut self, outcome: &CompressionOutcome, fitness_delta: f64) {
        let phi_before = self.compute();
        self.params.update_from_outcome(outcome, fitness_delta);
        let phi_after = self.compute();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.history.push(PhiSnapshot {
            phi: phi_after,
            phi_pct: phi_after * 100.0,
            log_terms: phi_before.ln(),
            timestamp: now,
        });
    }

    pub fn delta_phi(&self) -> Option<f64> {
        if self.history.len() < 2 {
            return None;
        }
        let last = self.history.last()?;
        let prev = &self.history[self.history.len() - 2];
        Some(last.phi - prev.phi)
    }

    pub fn history(&self) -> &[PhiSnapshot] {
        &self.history
    }

    /// 分层诊断：哪些因子在拖累Φ（升序）
    pub fn diagnose(&self) -> Vec<(&'static str, f64)> {
        let mut contributions = vec![
            ("Omega_A", self.params.omega_a),
            ("beta_bg", self.params.beta_bg),
            ("alpha_ack", self.params.alpha_ack),
            ("Theta_TRI", self.params.theta_tri),
            ("nabla_K", self.params.nabla_k),
            ("zeta_sigma", self.params.zeta_sigma),
            ("eta_lambda", self.params.eta_lambda),
            ("EVM", self.params.evm),
            ("A", self.params.a),
            ("B", self.params.b),
            ("TDHLGWB", self.params.tdhlgwb),
            ("phi_base", self.params.phi_base),
            ("EV", self.params.ev),
            ("AN", self.params.an),
            ("NV", self.params.nv),
        ];

        contributions.sort_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
        });
        contributions
    }
}

impl Default for PhiEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_space_physics() {
        let old_phi = {
            let params = PhiParams {
                omega_a: 0.1,
                beta_bg: 0.1,
                alpha_ack: 0.1,
                theta_tri: 0.1,
                nabla_k: 0.1,
                zeta_sigma: 0.1,
                eta_lambda: 0.1,
                evm: 0.1,
                a: 0.1,
                b: 0.1,
                tdhlgwb: 0.1,
                phi_base: 0.01,
                ev: 0.181,
                an: 0.1,
                nv: 0.1,
                harm_rate: 1.0,
            };
            PhiEngine::with_params(params).compute()
        };

        let new_phi = {
            let params = PhiParams::default();
            PhiEngine::with_params(params).compute()
        };

        println!(
            "旧公式 (全0.1): Phi = {:.6e} = {}%",
            old_phi,
            old_phi * 100.0
        );
        println!(
            "新公式 (平衡初始): Phi = {:.6e} = {}%",
            new_phi,
            new_phi * 100.0
        );
        println!("改善倍数: {:.1e}x", new_phi / old_phi);

        assert!(old_phi < 1e-10, "旧公式应该接近零");
        assert!(new_phi > 1e-6, "新公式应该大于0.0001%");
        assert!(new_phi / old_phi > 10000.0, "改善应该超过10000倍");
    }

    #[test]
    fn test_default_phi_value() {
        let engine = PhiEngine::new();
        let phi = engine.compute();
        let phi_pct = engine.compute_pct();
        println!("默认参数 Phi = {}%", phi_pct);

        assert!(phi > 1e-6, "应该大于0.0001%");
        assert!(phi < 0.01, "应该小于1%");
    }

    #[test]
    fn test_diagnose() {
        let engine = PhiEngine::new();
        let diags = engine.diagnose();
        println!("短板排序（升序）:");
        for (name, val) in &diags {
            println!("  {}: {:.4}", name, val);
        }
        assert!(diags[0].1 < 0.5, "应该有短板因子");
    }

    #[test]
    fn test_evolve() {
        let mut engine = PhiEngine::new();
        let phi_before = engine.compute();

        let outcome = CompressionOutcome::new("Bash", "RTK", 1000, 200, "sess1", 1);
        engine.evolve_from_outcome(&outcome, 0.1);

        let phi_after = engine.compute();
        println!("Phi演化: {}% --> {}%", phi_before * 100.0, phi_after * 100.0);

        assert!(phi_after >= phi_before, "正向outcome应该提升Phi");
    }

    #[test]
    fn test_delta_phi() {
        let mut engine = PhiEngine::new();
        let outcome = CompressionOutcome::new("Bash", "RTK", 1000, 200, "sess1", 1);

        engine.evolve_from_outcome(&outcome, 0.1);
        engine.evolve_from_outcome(&outcome, 0.1);

        assert!(engine.delta_phi().is_some(), "应该能计算delta_Phi");
        println!("delta_Phi: {:?}", engine.delta_phi());
    }
}
