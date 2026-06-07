//! APEX Fusion - RTK + CBM + Headroom + Caveman 四合一压缩引擎
//! 
//! 融合来源：
//! - RTK (signal1project/rtk) - 截断/去重/跨轮dedup
//! - CBM - Context Boundary Marker 防越界标记
//! - Headroom (vishvacyber/Headroom-AI-Context-Compression) - SmartCrusher JSON压缩
//! - Caveman (wilpel/caveman-compression) - 剥语法留事实

# APEX Fusion

RTK + CBM + SmartCrusher + Caveman 四合一token压缩引擎。

## 编译

```bash
cargo build --release
```

## 运行Benchmark

```bash
cargo run --release -- benchmark
```

## 架构

- `src/rtk.rs` - 行截断 + 连续重复行去重
- `src/cbm.rs` - 边界标记防越界
- `src/smart_crusher.rs` - JSON结构压缩
- `src/caveman.rs` - 语义压缩(剥语法留事实)
