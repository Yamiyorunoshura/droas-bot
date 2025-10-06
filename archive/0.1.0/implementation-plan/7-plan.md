# Task-7 實作計畫：開發點對點轉帳功能

## 專案資訊

**task_id**: Task-7
**task_name**: 開發點對點轉帳
**created_date**: 2025-10-05

## 需求對應

**functional_ids**:
- F-004: Peer-to-Peer Transfers

**nonfunctional_ids**:
- NFR-P-001: 95% 指令在 2 秒內響應
- NFR-S-001: 100% 交易通過 Discord 用戶 ID 驗證

**architecture_refs**:
- Transfer Service (主要)
- Balance Service (相依)
- Security Service (相依)
- Database Layer (相依)
- Message/UI Service (相依)

## TDD 三階段實作計畫

### RED 階段：測試與驗收標準定義

#### 驗收標準

1. **成功轉帳驗收標準**
   - **criterion**: 發送者有足夠餘額且接收者有有效帳戶時，轉帳應成功完成
   - **test_condition**: 執行 !transfer @user amount 後，系統扣除發送者金額，增加到接收者帳戶，並向雙方發送確認通知

2. **餘額不足驗收標準**
   - **criterion**: 發送者餘額不足時，轉帳應被拒絕
   - **test_condition**: 執行轉帳時，系統返回餘額不足錯誤訊息，且不執行任何帳戶變更

3. **接收者不存在驗收標準**
   - **criterion**: 接收者帳戶不存在時，轉帳應被拒絕
   - **test_condition**: 執行轉帳時，系統返回無效接收者錯誤訊息，且不執行任何帳戶變更

4. **無效金額驗收標準**
   - **criterion**: 轉帳金額無效 (負數、零或非數字) 時，轉帳應被拒絕
   - **test_condition**: 執行轉帳時，系統返回無效金額錯誤訊息，且不執行任何帳戶變更

5. **交易原子性驗收標準**
   - **criterion**: 轉帳過程中發生系統錯誤時，所有變更應被回滾
   - **test_condition**: 模擬系統錯誤，確保帳戶餘額保持一致性

#### 測試案例

1. **test_name**: successful_transfer_test
   - **scenario**: 用戶A向用戶B轉帳100幣，雙方帳戶餘額正確更新
   - **expected_result**: A餘額減少100，B餘額增加100，雙方收到通知

2. **test_name**: insufficient_balance_test
   - **scenario**: 用戶A餘額50幣，嘗試轉帳100幣
   - **expected_result**: 返回餘額不足錯誤，A餘額保持50幣

3. **test_name**: invalid_recipient_test
   - **scenario**: 向不存在的用戶轉帳
   - **expected_result**: 返回無效接收者錯誤，發送者餘額不變

4. **test_name**: invalid_amount_test
   - **scenario**: 嘗試轉帳負數或零金額
   - **expected_result**: 返回無效金額錯誤，發送者餘額不變

5. **test_name**: atomic_transaction_test
   - **scenario**: 在轉帳過程中模擬系統故障
   - **expected_result**: 所有帳戶餘額保持原始狀態

### GREEN 階段：最小實作步驟

#### 實作步驟

1. **step**: 實作轉帳命令解析器
   - **files**:
     - src/discord_gateway/command_parser.rs (update)
   - **architecture_component**: Command Router
   - **description**: 解析 !transfer @user amount 命令格式，提取發送者、接收者和金額

2. **step**: 實作 Transfer Service 核心邏輯
   - **files**:
     - src/services/transfer_service.rs (create)
   - **architecture_component**: Transfer Service
   - **description**: 處理轉帳業務邏輯，協調餘額檢查、驗證和執行轉帳

3. **step**: 實作餘額驗證功能
   - **files**:
     - src/services/balance_service.rs (update)
   - **architecture_component**: Balance Service
   - **description**: 檢查用戶餘額是否足夠進行轉帳

4. **step**: 實作原子轉帳操作
   - **files**:
     - src/database/transaction_repository.rs (update)
   - **architecture_component**: Database Layer
   - **description**: 使用資料庫事務確保轉帳操作的原子性

5. **step**: 實作通知機制
   - **files**:
     - src/services/message_service.rs (update)
   - **architecture_component**: Message/UI Service
   - **description**: 向轉帳雙方發送轉帳成功或失敗的通知

6. **step**: 實作安全驗證整合
   - **files**:
     - src/services/security_service.rs (update)
   - **architecture_component**: Security Service
   - **description**: 驗證用戶身份和輸入參數有效性

#### 需要修改的檔案

- **path**: src/services/transfer_service.rs
  - **type**: source
  - **modification**: create

- **path**: src/discord_gateway/command_parser.rs
  - **type**: source
  - **modification**: update

- **path**: src/services/balance_service.rs
  - **type**: source
  - **modification**: update

- **path**: src/database/transaction_repository.rs
  - **type**: source
  - **modification**: update

- **path**: src/services/message_service.rs
  - **type**: source
  - **modification**: update

- **path**: tests/transfer_service_test.rs
  - **type**: test
  - **modification**: create

- **path**: tests/command_router_integration_test.rs
  - **type**: test
  - **modification**: update

### REFACTOR 階段：重構與優化

#### 優化目標

1. **target**: 程式碼重複消除
   - **quality_improvement**: 合併餘額檢查邏輯到統一的驗證服務，重構錯誤處理機制

2. **target**: 跨領域關注點整合
   - **quality_improvement**: 整合日誌記錄、統一安全驗證流程、加入交易指標收集

3. **target**: 性能優化
   - **quality_improvement**: 實作轉帳快取機制、優化資料庫事務、加入並發控制

4. **target**: 程式碼品質提升
   - **quality_improvement**: 提高可測試性、改善錯誤處理、加入型別安全檢查

5. **target**: 架構改善
   - **quality_improvement**: 確保服務介面一致性、優化依賴關係、加入交易重試機制

#### 品質改善措施

1. **improvement**: 實作統一的交易驗證介面
   - **rationale**: 避免重複的驗證邏輯，提高程式碼維護性

2. **improvement**: 加入轉帳操作的詳細日誌記錄
   - **rationale**: 支援問題排查和審計需求

3. **improvement**: 實作轉帳快取策略
   - **rationale**: 減少資料庫查詢次數，提升響應速度

4. **improvement**: 加入並發轉帳的鎖定機制
   - **rationale**: 防止競爭條件導致的餘額不一致

5. **improvement**: 優化錯誤訊息的用戶友好性
   - **rationale**: 提供更清晰的錯誤指導，改善用戶體驗

## 風險評估

### 風險項目

1. **description**: 交易原子性實作複雜度
   - **probability**: Medium
   - **impact**: High
   - **mitigation**: 使用成熟的資料庫事務機制，實作完整的回滾邏輯

2. **description**: 並發轉帳競爭條件
   - **probability**: Medium
   - **impact**: High
   - **mitigation**: 實作適當的鎖定機制和樂觀併發控制

3. **description**: 性能要求達成
   - **probability**: Low
   - **impact**: Medium
   - **mitigation**: 實作快取機制和資料庫查詢優化，進行性能測試

4. **description**: 與現有服務整合複雜度
   - **probability**: Low
   - **impact**: Medium
   - **mitigation**: 充分理解現有服務介面，實作清晰的服務間通訊

## 依賴關係

- **前置任務**: Task-6 (實現餘額查詢功能), Task-8 (加入轉帳驗證邏輯)
- **後續任務**: Task-11 (記錄交易歷史)

## 成功標準

- 所有測試案例通過
- 轉帳功能滿足所有驗收標準
- 性能指標達到要求 (2秒內響應)
- 程式碼品質達到專案標準
- 與現有系統無縫整合