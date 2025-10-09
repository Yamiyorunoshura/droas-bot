# Task-06 實施計劃 - 稽核日誌系統

## 任務上下文

**摘要**: 實現稽核日誌系統，記錄所有交易（轉帳、調整），查詢最近50筆記錄，按時間降序排列，管理者權限控制

**需求映射**:
- 功能需求: REQ-014 (記錄所有交易), REQ-015 (查詢稽核記錄), REQ-016 (稽核權限控制)
- 非功能需求: NFR-004 (權限驗證), NFR-015 (稽核日誌保留期限)

**架構上下文**:
- 組件: AuditService, AuditRepository
- 整合點: PostgreSQL (主資料存儲), Discord API (權限驗證)
- 依賴: Task-01 (Discord Gateway整合), Task-04 (轉帳系統), Task-05 (管理調整系統)

## TDD階段規劃

### RED階段 - 測試設計

**目標**: 建立完整的稽核日誌系統測試框架，確保交易記錄、查詢功能、權限控制等正確實現

**驗收標準到測試映射**:
1. 記錄所有交易（轉帳、調整）→ 交易記錄測試
2. 查詢最近50筆記錄 → 查詢功能測試
3. 按時間降序排列 → 排序功能測試
4. 管理者權限控制 → 權限控制測試

**測試用例**:
1. **交易記錄測試**
   - 測試轉帳交易完整記錄
   - 測試調整交易完整記錄
   - 測試記錄包含所有必要信息
   - 測試記錄保存失敗處理

2. **查詢功能測試**
   - 測試查詢最近50筆記錄
   - 測試記錄少於50筆時的處理
   - 測試查詢性能
   - 測試查詢結果格式正確

3. **排序功能測試**
   - 測試按時間降序排列
   - 測試相同時間記錄的排序
   - 測試排序穩定性
   - 測試排序性能

4. **權限控制測試**
   - 測試管理者查詢權限
   - 測試非管理者拒絕訪問
   - 測試權限檢查性能
   - 測試權限檢查錯誤處理

**測試文件**:
- `tests/audit_service/audit_service_test.rs`
- `tests/audit_service/record_test.rs`
- `tests/audit_service/query_test.rs`
- `tests/audit_service/permission_test.rs`

### GREEN階段 - 最小實現

**目標**: 實現稽核日誌系統的最小可行功能，使所有測試通過

**實現步驟**:
1. **實現交易記錄功能**
   - 創建 `src/services/audit_service.rs`
   - 實現轉帳交易記錄
   - 實現調整交易記錄
   - 添加記錄驗證

2. **開發查詢功能**
   - 實現最近記錄查詢
   - 實現分頁查詢
   - 實現排序功能
   - 添加查詢緩存

3. **實現權限控制**
   - 實現Discord權限檢查
   - 添加權限緩存
   - 實現權限錯誤處理
   - 添加權限日誌

4. **集成Discord Slash Command**
   - 實現 `/audit` 命令處理
   - 添加參數解析
   - 實現私密回應
   - 添加錯誤處理

5. **優化查詢性能**
   - 實現查詢優化
   - 添加索引支持
   - 實現結果緩存
   - 添加性能監控

**文件結構**:
- `src/services/audit_service.rs`
- `src/database/audit_repository.rs`
- `src/models/audit_log.rs`
- `src/commands/audit_command.rs`
- `src/utils/audit_formatter.rs`

**依賴項**:
- SQLx (資料庫ORM)
- Serenity (Discord集成)
- anyhow (錯誤處理)
- chrono (時間處理)

### REFACTOR階段 - 重構和優化

**目標**: 提升稽核日誌系統的性能、安全性和可維護性

**重構目標**:
1. **性能優化**
   - 實現查詢結果緩存
   - 優化資料庫查詢
   - 實現批量操作支持
   - 添加預處理語句

2. **安全性增強**
   - 實現訪問日誌
   - 添加敏感數據脫敏
   - 實現數據保留策略
   - 添加審計告警

3. **功能擴展**
   - 實現高級查詢
   - 添加統計報告
   - 實現導出功能
   - 添加實時監控

**質量改進**:
1. **監控和日誌**
   - 添加稽核操作監控
   - 實現統計報告
   - 添加性能指標
   - 實現告警機制

2. **用戶體驗改進**
   - 實現查詢預覽
   - 添加快捷操作
   - 實現結果篩選
   - 優化顯示格式

3. **可維護性改進**
   - 提取通用接口
   - 實現配置管理
   - 添加文檔注釋
   - 改進錯誤處理

**檢查清單**:
- [ ] 所有測試通過並達到95%+覆蓋率
- [ ] 交易記錄功能完整
- [ ] 查詢功能正常
- [ ] 排序功能正確
- [ ] 權限控制嚴格
- [ ] 性能指標達標
- [ ] 代碼通過clippy檢查無警告

## 附加細節

**配置變更**:
- 配置稽核保留期限
- 設置查詢限制參數
- 配置權限檢查緩存
- 設置敏感數據脫敏規則

**數據庫遷移**:
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    target_user_id BIGINT,
    action VARCHAR(50) NOT NULL,
    amount DECIMAL(15,2),
    old_balance DECIMAL(15,2),
    new_balance DECIMAL(15,2),
    reason TEXT,
    transaction_id UUID,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_guild ON audit_logs(guild_id);
CREATE INDEX idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_composite ON audit_logs(guild_id, created_at DESC);
```

**API文檔**:
- AuditService接口文檔
- AuditRepository API文檔
- 稽核查詢規則文檔
- Discord Slash Command文檔

**安全考慮**:
- 權限驗證機制
- 訪問日誌記錄
- 敏感數據脫敏
- 數據保留策略

## 風險管理

1. **性能瓶頸風險**
   - 實現查詢優化
   - 添加緩存機制
   - 實現分頁支持

2. **數據隱私風險**
   - 實現數據脫敏
   - 添加訪問控制
   - 實現數據加密

3. **存儲空間風險**
   - 實現數據保留策略
   - 添加數據清理
   - 實現數據壓縮

## 驗證檢查清單

- [ ] 交易記錄功能完整
- [ ] 查詢功能正常
- [ ] 排序功能正確
- [ ] 權限控制嚴格
- [ ] 性能指標達標
- [ ] 安全性措施到位
- [ ] 監控和日誌完整
- [ ] 文檔齊全且準確

## 備註

稽核日誌系統是合規和安全的重要組成部分，需要確保所有交易記錄的完整性和準確性。查詢性能是關鍵考慮因素，特別是在大量數據的情況下。權限控制必須嚴格，防止敏感信息洩露。