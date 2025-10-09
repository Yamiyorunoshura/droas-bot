# Task-05 實施計劃 - 管理調整系統

## 任務上下文

**摘要**: 實現管理調整系統，支持管理者權限驗證、執行餘額調整（add/remove/set）、支持負數餘額、記錄調整操作和理由

**需求映射**:
- 功能需求: REQ-011 (管理者權限驗證), REQ-012 (執行餘額調整), REQ-013 (調整記錄和理由)
- 非功能需求: NFR-004 (權限驗證), NFR-005 (交易完整性), NFR-014 (交易記錄保留期限)

**架構上下文**:
- 組件: AdminService, AuditRepository
- 整合點: PostgreSQL (主資料存儲), Discord API (權限驗證)
- 依賴: Task-01 (Discord Gateway整合), Task-02 (用戶帳戶管理系統), Task-03 (餘額查詢系統)

## TDD階段規劃

### RED階段 - 測試設計

**目標**: 建立完整的管理調整系統測試框架，確保權限驗證、餘額調整、記錄保存等功能正確實現

**驗收標準到測試映射**:
1. 管理者權限驗證 → 權限驗證測試
2. 執行餘額調整（add/remove/set）→ 餘額調整測試
3. 支持負數餘額 → 負數餘額測試
4. 記錄調整操作和理由 → 記錄保存測試

**測試用例**:
1. **權限驗證測試**
   - 測試Discord Administrator權限檢查
   - 測試無權限用戶拒絕訪問
   - 測試權限檢查性能
   - 測試權限檢查錯誤處理

2. **餘額調整測試**
   - 測試add操作正確增加餘額
   - 測試remove操作正確減少餘額
   - 測試set操作正確設置餘額
   - 測試調整操作原子性

3. **負數餘額測試**
   - 測試支持負數餘額設置
   - 測試負數餘額顯示正確
   - 測試負數餘額轉帳限制
   - 測試負數餘額統計準確

4. **記錄保存測試**
   - 測試調整操作完整記錄
   - 測試調整前後餘額記錄
   - 測試操作者和理由記錄
   - 測試交易編號生成

**測試文件**:
- `tests/admin_service/admin_service_test.rs`
- `tests/admin_service/permission_test.rs`
- `tests/admin_service/adjustment_test.rs`
- `tests/admin_service/audit_test.rs`

### GREEN階段 - 最小實現

**目標**: 實現管理調整系統的最小可行功能，使所有測試通過

**實現步驟**:
1. **實現權限驗證系統**
   - 創建 `src/services/admin_service.rs`
   - 實現Discord權限檢查
   - 添加權限緩存
   - 實現權限錯誤處理

2. **開發餘額調整邏輯**
   - 實現add操作邏輯
   - 實現remove操作邏輯
   - 實現set操作邏輯
   - 添加調整驗證

3. **實現記錄保存系統**
   - 創建 `src/database/audit_repository.rs`
   - 實現調整記錄保存
   - 添加記錄查詢功能
   - 實現記錄格式化

4. **集成Discord Slash Command**
   - 實現 `/adjust` 命令處理
   - 添加參數解析和驗證
   - 實現私密回應
   - 添加錯誤處理

5. **添加交易編號生成**
   - 實現管理調整交易編號
   - 添加編號格式化
   - 實現編號唯一性保證
   - 添加編號查詢功能

**文件結構**:
- `src/services/admin_service.rs`
- `src/database/audit_repository.rs`
- `src/models/audit_record.rs`
- `src/commands/adjust_command.rs`
- `src/utils/permission_checker.rs`

**依賴項**:
- SQLx (資料庫ORM)
- Serenity (Discord集成)
- anyhow (錯誤處理)
- uuid (交易編號生成)

### REFACTOR階段 - 重構和優化

**目標**: 提升管理調整系統的安全性、性能和可維護性

**重構目標**:
1. **安全性增強**
   - 實現操作審計
   - 添加操作日誌
   - 實現權限分級
   - 添加敏感操作確認

2. **性能優化**
   - 實現權限檢查緩存
   - 優化資料庫查詢
   - 實現批量操作支持
   - 添加預處理語句

3. **功能擴展**
   - 實現批量調整
   - 添加調整模板
   - 實現調整預覽
   - 添加調整歷史

**質量改進**:
1. **監控和日誌**
   - 添加管理操作監控
   - 實現操作統計
   - 添加安全告警
   - 實現審計報告

2. **用戶體驗改進**
   - 實現操作確認
   - 添加操作預覽
   - 實現快捷操作
   - 優化錯誤訊息

3. **可維護性改進**
   - 提取通用接口
   - 實現配置管理
   - 添加文檔注釋
   - 改進錯誤處理

**檢查清單**:
- [ ] 所有測試通過並達到95%+覆蓋率
- [ ] 權限驗證100%準確
- [ ] 餘額調整功能正常
- [ ] 負數餘額支持完整
- [ ] 記錄保存完整準確
- [ ] 安全性措施到位
- [ ] 代碼通過clippy檢查無警告

## 附加細節

**配置變更**:
- 添加管理員角色配置
- 配置調整限制參數
- 設置審計保留期限
- 配置操作確認要求

**數據庫遷移**:
```sql
CREATE TABLE audit_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_user_id BIGINT NOT NULL,
    target_user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    action VARCHAR(20) NOT NULL,
    old_balance DECIMAL(15,2),
    new_balance DECIMAL(15,2),
    amount DECIMAL(15,2),
    reason TEXT,
    transaction_id UUID,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_records_admin ON audit_records(admin_user_id);
CREATE INDEX idx_audit_records_target ON audit_records(target_user_id);
CREATE INDEX idx_audit_records_guild ON audit_records(guild_id);
CREATE INDEX idx_audit_records_created_at ON audit_records(created_at);
```

**API文檔**:
- AdminService接口文檔
- AuditRepository API文檔
- 權限驗證規則文檔
- Discord Slash Command文檔

**安全考慮**:
- 權限驗證機制
- 操作審計日誌
- 輸入驗證和清理
- 敏感操作確認

## 風險管理

1. **權限濫用風險**
   - 實現嚴格權限檢查
   - 添加操作審計
   - 實現操作限制

2. **數據完整性風險**
   - 實現事務保護
   - 添加數據驗證
   - 實現備份機制

3. **操作錯誤風險**
   - 實現操作確認
   - 添加預覽功能
   - 實現回滾機制

## 驗證檢查清單

- [ ] 權限驗證功能正常
- [ ] 餘額調整操作正確
- [ ] 負數餘額支持完整
- [ ] 記錄保存完整準確
- [ ] 交易編號生成正確
- [ ] 安全性措施到位
- [ ] 監控和日誌完整
- [ ] 文檔齊全且準確

## 備註

管理調整系統是敏感功能，需要特別注意安全性和審計要求。所有操作都必須有完整的記錄，確保可追溯性。權限驗證必須嚴格，防止權限濫用。