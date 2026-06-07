# APEX 核心公式体系

## Φ_APEX*∞ 评分公式

```
Φ_APEX*∞ = (Φ_base × EV × AN × NV) / HarmRate
```

### 参数定义

| 参数 | 含义 | 自进化前 | 目标 |
|------|------|---------|------|
| Φ_base | 基线能力 | 0.1% | 10%+ |
| EV | evolve_ratio 进化率 | 0.1 | 2.0+ |
| AN | ann_quality 标注质量 | 0.1 | 1.5+ |
| NV | novelty_rate 创新率 | 0.1 | 2.0+ |
| HarmRate | 负势损耗 | 34% | <10% |

### 健康范围

- **T5 ULTIMATE**: Φ_APEX*∞ ≥ 150%
- **T4 ENHANCED**: 50% ~ 150%
- **T3 NORMAL**: 10% ~ 50%
- **T2 BASIC**: 1% ~ 10%
- **T1 EMBRYO**: < 1%

## EV = BV + AV 决策方程

```
EV (Expected Value) = BV (Baseline Value) + AV (Activation Value)
```

## 核心原则

- **ANN_QUALITY**: 标注质量决定模型上限
- **EVOLVE_RATIO**: 进化率衡量自我改进速度
- **NOVELTY_RATE**: 创新率衡量突破性发现能力
- **HARM_RATE**: 负势损耗必须可控（<34%）

## 自进化闭环

```
观察 → 分析 → 决策 → 执行 → 验证 → 固化 → 循环
```

每轮循环都将最优策略写入基因库，持续降熵。
