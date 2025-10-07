# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 專案概述

DROAS Discord Economy Bot 是一個基於 Rust 的 Discord 機器人，提供虛擬經濟系統功能。這是一個單體架構應用程式，遵循分層設計原則，清晰分離 Discord API 整合、業務邏輯和資料持久化。

## 技術堆疊

- **語言**: Rust
- **Discord 框架**: Serenity (Discord API v2+)
- **資料庫**: PostgreSQL (ACID 合規)
- **快取**: Redis
- **監控**: Prometheus
- **架構模式**: 單體應用程式、Repository 模式、分層架構

## 常用開發命令

### 建置和運行
```bash
cargo build --release    # 編譯發布版本
cargo run               # 運行開發版本
./target/release/droas-bot  # 運行發布版本
```

### 測試和檢查
```bash
cargo test              # 運行所有測試
cargo test specific_test_name  # 運行特定測試
cargo fmt               # 格式化代碼
cargo clippy -- -D warnings  # 檢查代碼風格
```

### 資料庫操作
```bash
# 資料庫遷移在程式啟動時自動執行
# 如需手動操作，可使用：
psql -d postgres://localhost/droas  # 連接資料庫
```

## 開發環境設置

開發前需要確保以下環境已正確配置：

1. **Rust 工具鏈**: Rust 1.88.0+ 和 Cargo 1.88.0+
2. **PostgreSQL**: 版本 16.x，用於 ACID 合規的資料持久化
3. **Redis**: 版本 8.x，用於快取層性能優化
4. **Discord Bot Token**: 需要從 Discord Developer Portal 獲取

**環境驗證命令**:
```bash
rustc --version    # 確認 Rust 版本
cargo --version    # 確認 Cargo 版本
psql --version     # 確認 PostgreSQL 客戶端
redis-cli --version # 確認 Redis 客戶端
cargo build        # 編譯項目驗證依賴
```

## 專案架構

### 核心架構元件

1. **Discord API Gateway**: 處理 Discord API 連接和事件監聽
2. **Command Router**: 解析命令並路由到適當服務
3. **User Account Service**: 管理用戶帳戶創建和驗證
4. **Balance Service**: 處理餘額查詢和更新
5. **Transfer Service**: 管理點對點轉帳
6. **Transaction Service**: 記錄交易歷史
7. **Message/UI Service**: 構建 Discord 嵌入消息
8. **Security/Validation Service**: 提供身份驗證和輸入驗證
9. **Database Layer**: 處理資料持久化 (Repository 模式)
10. **Cache Layer**: 提供 Redis 快取功能
11. **Monitoring/Metrics Service**: 收集性能指標
12. **Error Handling Framework**: 集中式錯誤處理

### 模組結構

```
src/
├── main.rs                 # 主程式入口，初始化所有服務
├── lib.rs                  # 庫入口，導出公共模組
├── config.rs               # 配置管理 (環境變數、資料庫、快取配置)
├── database/               # 資料庫層
│   ├── mod.rs             # 資料庫模組入口和遷移
│   ├── user_repository.rs # 用戶資料操作
│   ├── balance_repository.rs # 餘額資料操作
│   └── transaction_repository.rs # 交易資料操作
├── services/               # 業務邏輯層
│   ├── mod.rs             # 服務模組入口
│   ├── user_account_service.rs # 用戶帳戶管理
│   ├── balance_service.rs # 餘額業務邏輯
│   ├── transfer_service.rs # 轉帳業務邏輯
│   ├── transaction_service.rs # 交易歷史服務
│   ├── message_service.rs # Discord 消息構建
│   ├── security_service.rs # 安全驗證服務
│   └── monitoring_service.rs # 監控服務
├── discord_gateway/        # Discord API 整合
│   ├── mod.rs             # Gateway 模組入口
│   ├── command_parser.rs  # 命令解析
│   ├── command_registry.rs # 命令註冊
│   └── service_router.rs  # 服務路由
├── command_router.rs       # 命令路由核心
├── cache/                  # 快取層
│   └── mod.rs             # 快取抽象和 Redis/記憶體實現
├── styles/                 # UI 樣式
│   └── mod.rs             # Discord 嵌入消息樣式
├── error.rs                # 錯誤處理定義
├── logging.rs              # 日誌系統配置
├── health.rs               # 健康檢查
└── metrics.rs              # 指標收集
```

### 資料流設計

主要資料流程：
```
Discord Event → API Gateway → Command Router → Security Validation → Business Service → Cache/Database → Message Service → Discord Response
```

### 服務初始化流程

主程式按以下順序初始化：
1. 日誌系統
2. 配置載入 (從環境變數)
3. 資料庫連接池 (PostgreSQL)
4. 快取服務 (Redis，失敗則降級到記憶體)
5. 倉儲層 (UserRepository, BalanceRepository, TransactionRepository)
6. 業務服務 (UserAccountService, BalanceService, TransferService 等)
7. 監控服務 (健康檢查、指標收集)
8. Discord Gateway (注入所有服務)
9. 啟動監控 HTTP 服務器 (端口 8080)
10. 啟動 Discord 客戶端

## 開發指引

### 遵循的架構原則
- 單一職責原則，元件邊界清晰
- Repository 模式實現資料存取抽象
- ACID 合規資料庫確保交易完整性
- 快取層優化性能
- 集中式安全性和驗證框架
- 全面的監控和錯誤處理

### 環境配置

專案使用 `.env` 文件配置環境變數。必要變數：
- `DISCORD_TOKEN`: Discord 機器人令牌
- `DATABASE_URL`: PostgreSQL 連接字符串

可選變數：
- `REDIS_URL`: Redis 連接字符串
- `RUST_LOG`: 日誌級別 (debug/info/warn/error)
- `DROAS_MONITORING_PORT`: 監控服務端口 (預設 8080)

### 測試架構

測試位於 `tests/` 目錄，包含：
- 單元測試：測試各服務的獨立功能
- 整合測試：測試服務間協作
- 性能測試：驗證響應時間和並發能力
- 負載測試：模擬高負載場景

運行特定測試：
```bash
cargo test balance_service_test
cargo test integration_test -- --ignored
```

## 項目目標

### 主要目標
1. **提供完整的虛擬經濟系統**: 支援帳戶管理、轉帳交易、歷史查詢
2. **確保高性能和可靠性**: 滿足嚴格的響應時間和正常運行時間要求
3. **提供優秀的用戶體驗**: 使用交互式 Discord 嵌入消息界面
4. **確保安全性**: 實現完整的身份驗證和交易驗證機制

### 成功標準
- 所有核心功能按需求規範實現
- 性能指標達到非功能需求要求
- 100% 的交易通過安全驗證
- 用戶界面友好且直觀

## 文檔索引

### 架構文檔 (`docs/architecture/`)
- `Project Metadata.md` - 專案元數據、版本、狀態和更新日期 (v0.2.0, 2025-10-08)
- `Executive Summary.md` - 執行摘要、關鍵設計原則和當前實作狀態，包含管理員功能實現
- `Technical Stack.md` - 技術堆疊詳細信息，包含外部服務和開發工具 (Serenity 0.12, PostgreSQL 16.x, Redis 8.x)
- `System Architecture.md` - 系統架構元件詳細描述，包含 Admin Service 和 Admin Audit Service
- `API Documentation.md` - 內部 API 和外部 API 文檔，包含管理員 API 端點和審計 API
- `Requirements Traceability.md` - 功能和非功能需求完整追溯，包含 F-009 到 F-012 管理員功能需求
- `Architecture Decisions.md` - 11個關鍵架構決策記錄 (ADR-001 到 ADR-011)，包含管理員相關決策
- `Cross-Cutting Concerns.md` - 安全性、性能、可靠性和可觀測性關注點，包含三重驗證機制
- `Deployment Architecture.md` - 部署環境、基礎設施、CI/CD 和擴展策略
- `Architecture Quality.md` - 架構優勢、限制和技術債務分析，基於 v0.2.0 實際實現
- `Architecture Diagram.md` - 系統架構視覺化圖表 (Mermaid 格式)，包含完整組件關係
- `Source References.md` - 所有源文檔的完整參考列表，包含需求、代碼和測試結果