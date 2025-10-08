# 配置錯誤案例

## 環境變數名稱不一致

**錯誤類型**: 配置錯誤 (Configuration Error)

**發生情境**:
在環境配置中，.env文件使用DISCORD_BOT_TOKEN，但程式碼期望DISCORD_TOKEN，導致機器人無法啟動。

**具體位置**:
- `.env`文件 - 使用DISCORD_BOT_TOKEN
- 程式碼 - 期望DISCORD_TOKEN

**錯誤表現**:
- 機器人啟動失敗
- Discord Token無法讀取
- 配置值為空或None

**解決方案**:
1. 統一環境變數名稱為DISCORD_TOKEN
2. 更新.env文件中的變數名稱
3. 創建.env.example模板文件
4. 添加配置驗證機制

**證據來源**:
- 錯誤文檔: `docs/knowledge/errors-development.md` [環境變數配置不一致]段落
- cutover修復開發筆記: `archive/0.1.0/dev-notes/cutover-fixes-dev-notes.md` [配置修復]段落

**預防措施**:
- 建立統一的配置命名規範
- 使用配置文件模板
- 實現配置驗證和錯誤提示
- 文檔化所有環境變數

---

## tracing-subscriber功能配置錯誤

**錯誤類型**: 功能配置錯誤 (Feature Configuration Error)

**發生情境**:
tracing-subscriber的EnvFilter功能需要明確啟用env-filter特性，但Cargo.toml中缺少此配置，導致日誌過濾功能失效。

**具體位置**:
- `Cargo.toml` - 缺少env-filter feature配置

**錯誤表現**:
- 日誌過濾功能不工作
- 所有日誌級別都顯示
- 環境變數RUST_LOG設置無效

**解決方案**:
1. 在Cargo.toml中添加env-filter feature
2. 驗證日誌過濾功能正常工作
3. 測試不同日誌級別的配置

**證據來源**:
- 錯誤文檔: `docs/knowledge/errors-development.md` [tracing-subscriber功能配置錯誤]段落

**預防措施**:
- 仔細閱讀依賴庫的文檔
- 測試所有依賴庫的功能
- 建立功能清單檢查
- 使用feature flag的最佳實踐

---

## Discord Developer Portal配置缺失

**錯誤類型**: 外部服務配置錯誤 (External Service Configuration Error)

**發生情境**:
Discord Developer Portal中未啟用必要的GUILD_MEMBERS intent，導致自動群組成員帳戶創建功能無法正常工作。

**具體位置**:
- Discord Developer Portal - Gateway Intents配置

**錯誤表現**:
- 無法接收群組成員變更事件
- sync_members命令無法獲取成員列表
- 自動帳戶創建功能失效

**解決方案**:
1. 在Discord Developer Portal啟用GUILD_MEMBERS intent
2. 配置Bot權限：管理員、讀取訊息、發送訊息
3. 驗證intent配置正確性
4. 添加配置檢查和錯誤提示

**證據來源**:
- cutover報告: `archive/0.2.4/cutover.md` [Discord Developer Portal 設定]段落
- 建議改進: `archive/0.2.4/cutover.md` [建議改進項目]段落 - 建議添加自動檢查

**預防措施**:
- 建立Discord配置檢查清單
- 實現自動配置驗證工具
- 文檔化所有Discord Developer Portal設置
- 添加配置錯誤的友好提示

---

## Redis連接配置錯誤

**錯誤類型**: 服務連接配置錯誤 (Service Connection Configuration Error)

**發生情境**:
Redis服務連接配置不正確或服務不可用，導致快取功能失效，但系統能正確降級到記憶體快取。

**具體位置**:
- 環境變數REDIS_URL配置
- Redis服務可用性

**錯誤表現**:
- Redis連接失敗
- 快取功能降級到記憶體
- 性能可能受影響但不影響功能

**解決方案**:
1. 驗證Redis服務運行狀態
2. 檢查REDIS_URL環境變數格式
3. 確認Redis服務版本兼容性
4. 實現適當的錯誤處理和降級機制

**證據來源**:
- cutover報告: `archive/0.2.4/cutover.md` [Environment Setup]段落 - Redis 8.x連接測試
- 0.2.0 cutover報告: `archive/0.2.4/cutover.md` [configuration_required]段落

**預防措施**:
- 建立服務依賴檢查機制
- 實現健康檢查端點
- 文檔化所有外部服務配置
- 提供配置驗證工具

---

## 監控端口配置衝突

**錯誤類型**: 網絡配置錯誤 (Network Configuration Error)

**發生情境**:
監控服務端口(預設8080)被其他服務佔用，導致監控服務無法啟動。

**具體位置**:
- 環境變數DROAS_MONITORING_PORT
- 監控服務啟動配置

**錯誤表現**:
- 監控服務啟動失敗
- 端口綁定錯誤
- 健康檢查服務不可用

**解決方案**:
1. 檢查端口佔用情況
2. 修改DROAS_MONITORING_PORT環境變數
3. 實現端口自動檢測和選擇
4. 添加端口衝突的錯誤提示

**證據來源**:
- cutover報告: `archive/0.2.4/cutover.md` [可選配置]段落 - DROAS_MONITORING_PORT配置說明

**預防措施**:
- 實現端口可用性檢查
- 提供多個備選端口配置
- 文檔化所有網絡配置選項
- 添加配置衝突檢測