# Evolver断连诊断

## 发现
- gene_match返回空 → APEX MCP无基因数据
- harm_rate=0.34来自APEX MCP服务，非本地Evolver代码
- 本地Evolver在/root/evolver/运行正常
- APEX MCP服务和本地Evolver之间断连

## 待确认
- gene_input.json内容（应含512基因）
- MCP服务如何获取基因数据
