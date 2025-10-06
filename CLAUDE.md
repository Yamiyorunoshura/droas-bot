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

### 資料流設計

主要資料流程：
```
Discord Event → API Gateway → Command Router → Security Validation → Business Service → Cache/Database → Message Service → Discord Response
```

## 開發指引

### 遵循的架構原則
- 單一職責原則，元件邊界清晰
- Repository 模式實現資料存取抽象
- ACID 合規資料庫確保交易完整性
- 快取層優化性能
- 集中式安全性和驗證框架
- 全面的監控和錯誤處理

### 開發優先級

根據 docs/epic.md 的建議開發順序：

1. **第一階段（基礎設置）**: Discord API 連接、命令路由器、資料庫架構
2. **第二階段（帳戶管理）**: 自動帳戶創建、用戶驗證機制
3. **第三階段（核心功能）**: 餘額查詢、點對點轉帳、轉帳驗證、快取層
4. **第四階段（用戶界面）**: 嵌入消息模板、交互按鈕功能
5. **第五階段（擴展功能）**: 交易歷史、歷史查詢、幫助系統
6. **第六階段（監控優化）**: 監控系統、錯誤處理、性能擴展優化

### 命令模式

所有 Discord 命令應遵循：
- `!balance` - 查詢餘額
- `!transfer @user amount` - 轉帳給指定用戶
- `!history` - 查看交易歷史
- `!help` - 顯示幫助信息

### 性能要求

- 95% 的命令需在 2 秒內響應
- 餘額查詢需在 500ms 內完成
- 支援 1000+ 並發用戶
- 系統正常運行時間需達 99.5%

### 安全要求

- 100% 的交易必須通過 Discord 用戶 ID 進行身份驗證
- 所有用戶輸入必須驗證和清理
- 阻止自我轉帳和無效交易
- 實現適當的錯誤處理和用戶友好的錯誤消息

## 測試策略

每個功能應遵循 TDD 週期：測試定義 → 實現功能 → 重構優化。確保所有交易操作具有原子性並通過適當的驗證檢查。

## 部署注意事項

- 確保 Discord Bot Token 安全存儲
- 資料庫連接字符串配置
- Redis 連接設置
- Prometheus 指標端點配置
- 遵循 Discord 服務條款

## 需求概述

### 核心功能需求 (F-IDs)
- **F-001**: Discord Bot 連接和響應性 - 機器人連接 Discord API 並在 2 秒內響應命令
- **F-002**: 自動帳戶創建 - 新用戶首次命令時自動創建帳戶並獲得 1000 幣
- **F-003**: 餘額查詢 - 使用嵌入消息界面查詢帳戶餘額
- **F-004**: 點對點轉帳 - 用戶間轉帳虛擬貨幣，包含確認機制
- **F-005**: 交易歷史 - 查看最近 10 筆交易記錄
- **F-006**: 交互式嵌入界面 - 所有機器人響應使用 Discord 嵌入消息
- **F-007**: 命令幫助系統 - 提供所有可用命令的幫助信息
- **F-008**: 交易驗證和安全 - 防止無效交易和未授權操作

### 非功能需求 (NFR-IDs)
- **NFR-P-001**: 響應時間 - 95% 命令在 2 秒內響應
- **NFR-P-002**: 資料庫性能 - 餘額查詢在 500ms 內完成
- **NFR-S-001**: 交易身份驗證 - 100% 交易通過 Discord 用戶 ID 驗證
- **NFR-S-002**: 輸入驗證 - 所有用戶輸入都經過驗證和清理
- **NFR-R-001**: 系統正常運行時間 - 99.5% 正常運行時間
- **NFR-U-001**: 錯誤消息 - 90% 錯誤消息提供可行的指導
- **NFR-SC-001**: 並發用戶 - 支援 1000+ 並發用戶

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
- `Project Metadata.md` - 專案元數據、狀態、完成日期
- `專案概述.md` - 專案概述、狀態、完成日期
- `結論概述.md` - 最終架構總結和關鍵權衡決策
- `系統架構元件.md` - 12個核心架構元件的職責和介面定義
- `資料流設計.md` - 主要資料流程、轉帳交易流程、帳戶創建流程
- `架構決策記錄 (ADR).md` - 5個關鍵架構決策及其理由
- `跨領域關注點.md` - 安全性、性能、可靠性等跨領域關注點
- `架構圖表.md` - 系統架構視覺化圖表
- `需求追溯矩陣.md` - 需求到架構元件的追溯關係
- `實作元件.md` - 實際實作的元件詳細信息、設計偏差和變更理由
- `實際技術堆疊.md` - 最終實際使用的前端、後端、資料庫、基礎設施和外部服務
- `關鍵決策記錄.md` - 6個關鍵架構決策的決策、理由、結果和經驗
- `架構質量.md` - 架構優勢、已知限制和技術債務
- `源參考.md` - 所有源文檔的完整參考列表
- `Source References.md` - 統一的源文檔參考格式

### 需求文檔 (`docs/requirements/`)
- `Project Overview.md` - 專案名稱和描述
- `Functional Requirements.md` - 8個功能需求及驗收標準
- `Non-Functional Requirements.md` - 7個非功能需求及測試方法
- `Constraints.md` - 技術約束、商業約束、法規約束
- `Assumptions and Risks.md` - 專案假設和風險評估

### 任務文檔
- `docs/epic.md` - 完整開發任務清單，包含審查狀態和依賴關係