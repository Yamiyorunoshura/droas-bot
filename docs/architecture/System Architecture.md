## System Architecture


### Components

#### Discord API Gateway
**responsibility**: 處理 Discord API 連接、事件監聽和初始命令路由
**technology**: Rust + Serenity 框架
**interfaces**:
- DiscordEvent → CommandRouter
- HTTP 監控端點 (/health, /metrics)
**dependencies**: Command Router, Message/UI Service, UI Component Factory

*source_refs: ["src/main.rs:88-140", "src/discord_gateway/mod.rs:200-244"]*

#### Command Router
**responsibility**: 解析 Discord 命令、路由到適當服務、格式化響應
**technology**: Rust 命令模式實作 + 依賴注入
**interfaces**:
- DiscordCommand → ServiceResponse
- 服務配置介面 (with_user_account_service 等)
**dependencies**: All Business Services, Security Service

*source_refs: ["src/main.rs:288-322", "src/command_router.rs:15-50"]*

#### User Account Service
**responsibility**: 管理用戶帳戶創建、驗證和用戶相關操作
**technology**: Rust + PostgreSQL + UserRepository
**interfaces**:
- DiscordUser → Account
- 自動帳戶創建
**dependencies**: Database Layer, Cache Layer

*source_refs: ["src/main.rs:228-301", "src/services/user_account_service.rs"]*

#### Balance Service
**responsibility**: 處理餘額查詢、更新和餘額相關業務邏輯
**technology**: Rust + Redis 快取 + BalanceRepository
**interfaces**:
- AccountId → Balance
- 快取支援的餘額查詢
- 管理員餘額調整介面
**dependencies**: Database Layer, Cache Layer, Security Service

*source_refs: ["src/services/balance_service.rs", "archive/0.2.4/prd-dev-notes.md:168-189"]*

#### Transfer Service
**responsibility**: 管理點對點轉帳，包含驗證和原子操作
**technology**: Rust + ACID 事務 + Security Service
**interfaces**:
- TransferRequest → TransferResult
- 轉帳驗證和安全檢查
**dependencies**: Balance Service, Security Service, Database Layer

*source_refs: ["src/services/transfer_service.rs:174-247"]*

#### Transaction Service
**responsibility**: 記錄交易歷史、提供審計軌跡和歷史查詢
**technology**: Rust + Repository 模式 + TransactionRepository
**interfaces**:
- TransactionQuery → TransactionHistory
- 歷史查詢和格式化
- 管理員審計記錄介面
**dependencies**: Database Layer

*source_refs: ["src/database/transaction_repository.rs:379-384"]*

#### Admin Service
**responsibility**: 管理員身份驗證、權限檢查和管理員操作協調
**technology**: Rust + 三重驗證機制 + Discord 原生權限
**interfaces**:
- AdminOperation → OperationResult
- 管理員權限驗證
- 動態管理員列表管理
**dependencies**: Security Service, Database Layer, Admin Audit Service

*source_refs: ["src/services/admin_service.rs", "archive/0.2.4/prd-dev-notes.md:106-124"]*

#### Admin Audit Service
**responsibility**: 記錄管理員操作、提供審計查詢和統計
**technology**: Rust + 專用審計表 + TransactionRepository
**interfaces**:
- AdminAuditRecord → AuditResult
- 審計歷史查詢
- 審計統計分析
**dependencies**: Database Layer

*source_refs: ["src/services/admin_audit_service.rs", "archive/0.2.4/prd-dev-notes.md:124-135"]*

#### Message/UI Service
**responsibility**: 構建 Discord 嵌入消息和管理交互組件
**technology**: Discord 嵌入消息構建器
**interfaces**:
- ServiceData → DiscordEmbed
- 交互式消息格式化
**dependencies**: Discord API Gateway

*source_refs: ["src/services/message_service.rs"]*

#### Help Service
**responsibility**: 提供命令幫助系統和使用指南
**technology**: Rust + 動態命令註冊
**interfaces**:
- !help [command] → 幫助信息
- 命令分類和描述
**dependencies**: Command Router

*source_refs: ["src/services/help_service.rs"]*

#### Security/Validation Service
**responsibility**: 提供身份驗證、輸入驗證和安全檢查
**technology**: Rust 輸入驗證 + 身份驗證 + 異常檢測
**interfaces**:
- UserInput → ValidationResult
- 轉帳驗證和安全檢查
- 管理員權限驗證
- 異常操作模式檢測
**dependencies**: Cache Layer, Database Layer

*source_refs: ["src/services/security_service.rs", "archive/0.2.4/prd-dev-notes.md:136-142"]*

#### Monitoring Service
**responsibility**: 提供健康檢查、指標收集和監控端點
**technology**: Rust + Warp HTTP 框架 + Prometheus 指標
**interfaces**:
- HTTP Endpoints (/health, /metrics)
- 系統健康狀態檢查
**dependencies**: Database Layer, All Services

*source_refs: ["src/services/monitoring_service.rs:49-74"]*

### Data Flow

#### 主要資料流程
```
Discord Event → API Gateway → Command Router → Security Validation → Business Service → Cache/Database → Message Service → Discord Response
```

#### 轉帳交易流程
```
Transfer Command → Security Validation → Transfer Service → Dual Balance Check → Database Transaction → Notification → Audit Log
```

#### 帳戶創建流程
```
New User Command → User Account Service → Account Creation → Initial Balance → Welcome Message → Cache Update
```

#### 餘額查詢流程（含快取）
```
Balance Command → Security Validation → Balance Service → Cache Check → Database Query (if needed) → Cache Update → Response
```

#### 管理員操作流程
```
Admin Command → Discord Permission Check → Admin Service → Security Validation → Operation Execution → Audit Log → Response
```

*source_refs: ["docs/architecture/System Architecture.md:146-192"]*

### Integration Points

#### Discord API 整合
- Gateway Intents: GUILD_MESSAGES, MESSAGE_CONTENT, GUILD_MESSAGE_REACTIONS, DIRECT_MESSAGES, GUILD_MEMBERS
- 命令解析: !balance, !transfer, !history, !help, !adjust_balance, !admin_history
- 交互組件: 嵌入消息、按鈕、表單
- 原生權限: 伺服器擁有者、Administrator、MANAGE_GUILD

#### 資料庫整合
- ACID 事務確保轉帳原子性
- 自動資料庫遷移
- 連接池管理 (idle_timeout: 10m, max_lifetime: 30m)
- 專用審計表 (admin_audit)

#### 快取整合
- Redis 快取 + 記憶體快取降級
- TTL 管理 (預設 5 分鐘)
- 快取預熱和失效策略

#### 監控整合
- HTTP 監控端點 (預設端口 8080)
- Prometheus 指標收集
- 健康檢查和系統狀態監控
- SLA 追蹤 (100ms 響應時間要求)

*source_refs: ["docs/architecture/System Architecture.md:170-192"]*

