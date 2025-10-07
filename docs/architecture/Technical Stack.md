## Technical Stack


**frontend**: Discord 嵌入消息和交互組件 (Serenity 0.12)
**backend**: Rust + Serenity 框架 + Tokio 異步運行時
**database**: PostgreSQL 16.x (ACID 合規)
**infrastructure**: Redis 8.x 快取 (可選，支援記憶體快取降級)

### External Services
- **Discord API**: v2+, Gateway Intents: GUILD_MESSAGES, MESSAGE_CONTENT, Serenity 0.12
- **PostgreSQL**: 16.x, 主資料庫，ACID 交易支援
- **Redis**: 8.x, 快取層，熱數據存取優化

### Development Tools
- Cargo (Rust 包管理器)
- PostgreSQL 資料庫遷移工具
- Redis 命令行工具
- tracing + tracing-subscriber (日誌系統)
- Prometheus 指標收集

*source_refs: ["Cargo.toml:6-38", "docs/architecture/Technical Stack.md", "src/config.rs"]*

