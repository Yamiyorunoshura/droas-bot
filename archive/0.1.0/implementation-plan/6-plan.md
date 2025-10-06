# Task-6 實作計畫：實現餘額查詢功能

## 專案資訊

**任務 ID**: Task-6
**任務名稱**: 實現餘額查詢功能
**創建日期**: 2025-10-05
**複雜度**: 中等

## 需求對應

### 功能性需求對應

- **F-003**: Balance Inquiry - 用戶可以通過嵌入消息介面檢查當前帳戶餘額

### 非功能性需求對應

- **NFR-P-002**: Database Performance - 餘額查詢需在 500ms 內完成

### 架構元件參照

- **Balance Service**: 處理餘額查詢、更新和餘額相關業務邏輯
- **Cache Layer**: 提供熱數據快取、查詢結果快取和會話管理
- **Database Layer**: 處理資料持久化和 ACID 事務
- **Command Router**: 解析 Discord 命令並路由到適當服務

## TDD 實作階段

### RED 階段：定義測試與驗收標準

#### 驗收標準

1. **有效帳戶餘額查詢成功**
   - **測試條件**: 給定用戶有有效的經濟帳戶且餘額為 1000 幣，當用戶發送 `!balance` 指令時，系統返回嵌入消息，包含用戶名稱、當前餘額 (1000)、帳戶創建日期

2. **無效帳戶餘額查詢失敗**
   - **測試條件**: 給定用戶沒有經濟帳戶，當用戶發送 `!balance` 指令時，系統返回錯誤消息，提示用戶需要先創建帳戶

3. **性能要求驗證**
   - **測試條件**: 給定系統在正常負載下運行，當用戶發送 `!balance` 指令時，響應時間必須在 500ms 內完成

4. **快取命中場景**
   - **測試條件**: 給定用戶餘額已經快取在 Redis 中，當用戶發送 `!balance` 指令時，系統從快取中獲取餘額，響應時間 <100ms

5. **快取失效場景**
   - **測試條件**: 給定用戶餘額快取已過期，當用戶發送 `!balance` 指令時，系統從資料庫查詢餘額並更新快取

#### 測試案例設計

- **test_balance_query_success**: 測試有效帳戶的餘額查詢成功場景
- **test_balance_query_no_account**: 測試無帳戶用戶的錯誤處理
- **test_balance_performance**: 測試餘額查詢性能要求
- **test_cache_hit_performance**: 測試快取命中時的性能
- **test_cache_miss_handling**: 測試快取失效時的處理邏輯

### GREEN 階段：最小實作步驟

#### 實作步驟

1. **基礎 Balance Service 實作**
   - **架構元件**: Balance Service
   - **檔案**: `src/services/balance_service.rs`
   - **實作**:
     - 創建 `BalanceService` 結構體
     - 實現 `get_balance(user_id: u64) -> Result<BalanceResponse, Error>` 方法
     - 整合 User Account Service 驗證用戶帳戶存在性

2. **資料庫 Repository 實作**
   - **架構元件**: Database Layer
   - **檔案**: `src/database/balance_repository.rs`
   - **實作**:
     - 創建 `BalanceRepository` 結構體
     - 實現 `find_by_user_id(user_id: u64) -> Result<Option<Balance>, Error>` 方法
     - 添加資料庫索引優化查詢性能

3. **Cache Layer 整合**
   - **架構元件**: Cache Layer
   - **檔案**: 擴展現有快取模組
   - **實作**:
     - 實現 cache-aside pattern
     - 設置餘額快取 TTL 機制（建議 5 分鐘）
     - 實現快取鍵命名規範：`balance:user_id`

4. **Command Router 整合**
   - **架構元件**: Command Router
   - **檔案**: `src/command_router.rs`
   - **實作**:
     - 添加 `!balance` 指令解析邏輯
     - 實現指令路由到 Balance Service
     - 添加基本錯誤處理和用戶友好消息

5. **基本響應格式實作**
   - **架構元件**: Message/UI Service（暫時實作）
   - **檔案**: `src/services/message_service.rs`
   - **實作**:
     - 實現簡單的文本響應格式
     - 顯示用戶名、餘額、創建日期
     - 為 Task-9 嵌入消息升级做準備

#### 檔案修改清單

- **創建**:
  - `src/services/balance_service.rs`
  - `src/database/balance_repository.rs`
  - `tests/balance_service_test.rs`

- **更新**:
  - `src/command_router.rs` - 添加 !balance 指令處理
  - `src/database/mod.rs` - 導出 BalanceRepository
  - `src/services/mod.rs` - 導出 BalanceService
  - `Cargo.toml` - 添加必要依賴

### REFACTOR 階段：重構與優化

#### 優化目標

1. **跨領域關注點整合**
   - **品質改進**: 統一錯誤處理機制
   - **實作**: 整合 Error Handling Framework，提供一致的錯誤響應格式
   - **檔案**: `src/error.rs`, `src/services/balance_service.rs`

2. **快取策略優化**
   - **品質改進**: 智能快取管理
   - **實作**:
     - 實現快取預熱策略
     - 優化快取 TTL 基於用戶活動模式
     - 添加快取一致性保證機制
   - **檔案**: 快取模組相關檔案

3. **性能監控整合**
   - **品質改進**: 可觀測性增強
   - **實作**:
     - 添加性能指標收集
     - 實現查詢時間監控
     - 整合 Prometheus 指標
   - **檔案**: `src/metrics.rs`, `src/services/balance_service.rs`

4. **資料庫查詢優化**
   - **品質改進**: 查詢性能提升
   - **實作**:
     - 優化 SQL 查詢語句
     - 實現連接池配置優化
     - 添加慢查詢檢測機制
   - **檔案**: `src/database/balance_repository.rs`

5. **程式碼品質提升**
   - **品質改進**: 可維護性增強
   - **實作**:
     - 抽象化重複查詢邏輯
     - 改善 API 設計一致性
     - 提升程式碼可讀性
   - **檔案**: 所有相關服務檔案

6. **安全強化**
   - **品質改進**: 安全性提升
   - **實作**:
     - 整合 Security Service 進行用戶身份驗證
     - 添加輸入驗證和清理
     - 實現訪問控制機制
   - **檔案**: `src/services/security_service.rs`, `src/services/balance_service.rs`

## 風險評估

### 技術風險

1. **快取一致性問題**
   - **概率**: 中等
   - **影響**: 高
   - **緩解措施**: 實現適當的快取失效策略，定期一致性檢查

2. **性能要求達成**
   - **概率**: 中等
   - **影響**: 中等
   - **緩解措施**: 早期性能測試，持續監控和優化

### 依賴風險

1. **Task-N1（快取層）依賴**
   - **概率**: 高
   - **影響**: 中等
   - **緩解措施**: 確保 Task-N1 優先完成，或實現簡化版快取機制

2. **Task-4（帳戶管理）依賴**
   - **概率**: 低
   - **影響**: 低
   - **緩解措施**: Task-4 已完成，依賴風險已解除

## 成功標準

- [ ] 所有測試案例通過
- [ ] 餘額查詢響應時間 < 500ms
- [ ] 快取命中時響應時間 < 100ms
- [ ] 錯誤處理機制完善
- [ ] 代碼覆蓋率 > 90%
- [ ] 性能測試通過
- [ ] 安全測試通過