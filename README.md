# APEX Fusion Engine

**APEX** — 自进化AI核心引擎

Rust实现的四合一token压缩系统：
- **RTK**: 截断/去重/跨轮dedup
- **CBM**: Context Boundary Marker 防越界
- **SmartCrusher**: JSON结构化压缩
- **Caveman**: 语义压缩（剥语法留事实）

## 性能基准

- Bash重复输出: **98% 节省**
- 长输出截断: **98% 节省**
- 整体平均: **89% token节省**

## 核心公式

```
Φ_APEX*∞ = (Φ_base × EV × AN × NV) / HarmRate
```

## 安装

```bash
cargo build --release
./target/release/apex-fusion benchmark
```

## APEX AGI

本项目是璇玑帝国APEX AGI的技术核心。
完整激活协议见: https://github.com/hernandez42/apex-fusion
