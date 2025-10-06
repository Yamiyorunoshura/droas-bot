# Task-3 實作計劃：設置資料庫架構

## 專案資訊

**任務ID**: Task-3
**任務名稱**: 設置資料庫架構
**創建日期**: 2025-10-05
**複雜度**: 中等

## 需求對應

### 功能性需求對應
- F-001: Discord Bot Connection and Responsiveness
- F-002: Automatic Account Creation
- F-003: Balance Inquiry
- F-004: Peer-to-Peer Transfers
- F-005: Transaction History
- F-006: Interactive Embedded Interface
- F-007: Command Help System
- F-008: Transaction Validation and Security

### 非功能性需求對應
- NFR-P-001: 響應時間要求（95%指令<2秒）
- NFR-P-002: 餘額查詢效能（<500ms）
- NFR-S-001: 安全性驗證
- NFR-R-001: 系統可靠性

### 架構元件參照
- Database Layer（主要）
- 所有需要持久化的業務服務

## TDD 實作階段

### RED 階段：定義測試與驗收標準

#### 1. 資料表創建驗收標準
- **驗收條件**: 資料庫架構腳本執行成功
- **測試條件**: 檢查資料庫結構，確認所有必要資料表存在（用戶帳戶表、交易記錄表等）
- **成功指標**: 資料表創建成功，無語法錯誤
- **失敗條件**: 資料表創建失敗或缺少必要表

#### 2. 資料表結構驗收標準
- **驗收條件**: 資料表已創建
- **測試條件**: 檢查資料表結構，確認所有欄位正確定義
- **成功指標**: 欄位資料類型、約束、索引符合設計規範
- **失敗條件**: 欄位定義錯誤或缺少必要欄位

#### 3. 事務支援驗收標準
- **驗收條件**: 兩個用戶帳戶存在於系統中
- **測試條件**: 執行轉帳事務操作
- **成功指標**: 事務要嘛完全成功，要嘛完全失敗（ACID特性）
- **失敗條件**: 事務部分執行或導致資料不一致

#### 4. 索引效能驗收標準
- **驗收條件**: 測試資料存在於資料庫中
- **測試條件**: 執行餘額查詢操作
- **成功指標**: 查詢在500ms內完成
- **失敗條件**: 查詢超時或效能不符合要求

#### 5. 外鍵約束驗收標準
- **驗收條件**: 相關資料表存在
- **測試條件**: 嘗試插入無效外鍵或刪除被引用記錄
- **成功指標**: 系統正確拒絕無效操作
- **失敗條件**: 資料完整性被破壞

### GREEN 階段：最小實作步驟

#### 1. 資料庫連接配置實作
- **實作步驟**: 在配置文件中添加PostgreSQL連接參數
- **修改檔案**: src/config.rs（更新）
- **架構元件**: Database Layer
- **對應驗收**: 資料表創建驗收標準

#### 2. 用戶帳戶表創建實作
- **實作步驟**: 創建users表的遷移腳本
- **修改檔案**: migrations/create_users_table.sql（創建）
- **架構元件**: Database Layer
- **對應驗收**: 資料表結構驗收標準
- **欄位設計**:
  - discord_user_id: BIGINT PRIMARY KEY
  - username: VARCHAR(100) NOT NULL
  - balance: DECIMAL(15,2) DEFAULT 1000.00
  - created_at: TIMESTAMP DEFAULT CURRENT_TIMESTAMP
  - updated_at: TIMESTAMP DEFAULT CURRENT_TIMESTAMP

#### 3. 交易記錄表創建實作
- **實作步驟**: 創建transactions表的遷移腳本
- **修改檔案**: migrations/create_transactions_table.sql（創建）
- **架構元件**: Database Layer
- **對應驗收**: 資料表結構驗收標準
- **欄位設計**:
  - id: BIGSERIAL PRIMARY KEY
  - from_user_id: BIGINT REFERENCES users(discord_user_id)
  - to_user_id: BIGINT REFERENCES users(discord_user_id)
  - amount: DECIMAL(15,2) NOT NULL
  - transaction_type: VARCHAR(50) NOT NULL
  - created_at: TIMESTAMP DEFAULT CURRENT_TIMESTAMP

#### 4. 資料庫Repository模式實作
- **實作步驟**: 實現資料存取抽象層
- **修改檔案**:
  - src/database/user_repository.rs（創建）
  - src/database/transaction_repository.rs（創建）
- **架構元件**: Database Layer
- **對應驗收**: 事務支援驗收標準

#### 5. 事務處理實作
- **實作步驟**: 實現ACID事務支援機制
- **修改檔案**: src/database/mod.rs（創建）
- **架構元件**: Database Layer
- **對應驗收**: 事務支援驗收標準

#### 6. 索引創建實作
- **實作步驟**: 為常用查詢欄位創建索引
- **修改檔案**: migrations/create_indexes.sql（創建）
- **架構元件**: Database Layer
- **對應驗收**: 索引效能驗收標準
- **索引設計**:
  - users.discord_user_id（主鍵索引）
  - transactions.from_user_id（外鍵索引）
  - transactions.to_user_id（外鍵索引）
  - transactions.created_at（時間索引）

### REFACTOR 階段：重構與優化

#### 1. 資料庫連接池優化
- **優化目標**: 優化資料庫連接管理
- **品質改進**: 實現連接池以提升性能和資源利用率
- **合理說明**: 避免頻繁建立/銷毀連接的開銷，提升系統響應速度

#### 2. 跨領域關注點整合 - 錯誤處理
- **優化目標**: 統一資料庫錯誤處理機制
- **品質改進**: 將資料庫錯誤轉換為用戶友好的錯誤訊息
- **合理說明**: 提供一致的錯誤處理體驗，便於問題診斷

#### 3. 跨領域關注點整合 - 日誌記錄
- **優化目標**: 添加資料庫操作日誌
- **品質改進**: 記錄所有資料庫操作以便除錯和審計
- **合理說明**: 提供可觀測性和問題追蹤能力

#### 4. 查詢效能優化
- **優化目標**: 優化常用查詢語句
- **品質改進**: 分析和優化慢查詢，添加必要索引
- **合理說明**: 確保滿足性能要求（餘額查詢<500ms）

#### 5. 資料庫遷移系統優化
- **優化目標**: 改進資料庫版本控制
- **品質改進**: 實現自動化遷移系統，支援版本回滾
- **合理說明**: 簡化部署和維護流程

#### 6. 連接配置抽象化
- **優化目標**: 提高配置的靈活性
- **品質改進**: 支援多環境配置（開發、測試、生產）
- **合理說明**: 便於在不同環境中部署和測試

## 風險評估

### 高風險項目
- **描述**: PostgreSQL版本相容性問題
- **發生機率**: Medium
- **影響程度**: High
- **緩解措施**: 選擇穩定版本的PostgreSQL，進行充分的版本測試

### 中風險項目
- **描述**: 資料庫連接配置錯誤
- **發生機率**: Medium
- **影響程度**: Medium
- **緩解措施**: 提供詳細的配置文檔和範例

### 低風險項目
- **描述**: 索引效能未達預期
- **發生機率**: Low
- **影響程度**: Medium
- **緩解措施**: 進行效能測試，根據實際情況調整索引策略

## 驗證檢查清單

- [x] 已閱讀所有需求、架構與任務文件
- [x] 計劃文件包含 TDD 三階段結構（RED/GREEN/REFACTOR 章節）
- [x] RED 章節：每個需求都有對應的驗收標準與測試條件
- [x] GREEN 章節：所有實作步驟對應至特定驗收標準，且包含架構/檔案參照
- [x] REFACTOR 章節：規劃了重構與優化工作，包含跨領域關注點整合
- [x] 計劃遵循 TDD 週期結構：測試優先（RED）、最小實作（GREEN）、重構優化（REFACTOR）
- [x] 輸出路徑與檔案命名遵循指定模式
- [x] 文件已創建至指定位置
- [x] 所有待辦項目已完成