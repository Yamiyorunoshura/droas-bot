# Admin Service Errors

## 管理員服務未初始化錯誤 (2025-10-08)

**Error Type**: Configuration Error / Service Initialization Error

**Context**:
- 系統啟動後，管理員指令返回「管理員服務未初始化」錯誤
- AdminService 未在系統初始化過程中正確創建和註冊
- Discord Gateway 無法路由管理員指令到對應服務

**Root Cause**:
1. Config 結構體缺少管理員配置欄位
2. main.rs 中的 create_services 函數未創建 AdminService 實例
3. Services 結構體未包含 admin_service 欄位
4. Discord Gateway 未註冊 AdminService

**Solution**:
1. **配置層修復**:
   - 在 config.rs 中添加 AdminConfig 結構體
   - 實現從環境變數 ADMIN_USER_IDS 讀取授權管理員列表
   - 修改 Config 結構體包含 admin 欄位

2. **服務初始化修復**:
   - 更新 main.rs 中的 create_services 函數
   - 創建 AdminService 實例並傳入適當依賴
   - 將 AdminService 添加到 Services 結構體

3. **路由註冊修復**:
   - 在 Discord Gateway 中註冊 AdminService
   - 確保管理員指令能正確路由到服務

**Environment Configuration**:
- 生產環境：設置 `ADMIN_USER_IDS` 環境變數（逗號分隔的 Discord 用戶 ID）
- 測試環境：使用預設管理員 ID `123456789`

**Verification Steps**:
1. 編譯檢查：`cargo build --release`
2. 啟動驗證：檢查日誌中的「創建完整 Admin Service」訊息
3. 功能測試：執行管理員指令確認服務正常運作

**Prevention Measures**:
1. 在新增服務時，確保遵循完整的初始化流程：配置 → 服務創建 → 註冊
2. 為所有核心服務添加初始化日誌，便於問題診斷
3. 在 CI/CD 中加入服務初始化狀態檢查
4. 文檔化新服務的完整配置和初始化流程

**Source**:
- config.rs: AdminConfig 結構體和配置更新
- main.rs: create_services 函數修改
- Services 結構體更新
- Discord Gateway 服務註冊

## 管理員審計服務未初始化錯誤 (2025-10-08)

**Error Type**: Configuration Error / Service Initialization Error

**Context**:
- 系統啟動後，使用管理員指令時出現「管理員審計服務未初始化」錯誤
- AdminAuditService 已經完整實現，但在 main.rs 的服務初始化流程中缺少創建和注入
- Discord Gateway 無法訪問管理員審計服務，導致審計功能失效

**Root Cause**:
1. main.rs 的 import 語句中缺少 AdminAuditService 模組
2. create_services 函數中未創建 admin_audit_service 實例
3. Services 結構體中缺少 admin_audit_service 字段
4. Discord Gateway 配置中未注入 admin_audit_service

**Solution**:
1. **模組導入修復**:
   - 在 main.rs 的 import 語句中添加 AdminAuditService 模組

2. **服務實例化修復**:
   - 在 create_services 函數中創建 admin_audit_service 實例
   - 傳入適當的依賴（admin_audit_repository）

3. **服務結構修復**:
   - 將 admin_audit_service 字段添加到 Services 結構體

4. **依賴注入修復**:
   - 在 Discord Gateway 配置中注入 admin_audit_service

**Verification Steps**:
1. 編譯檢查：`cargo build --release`
2. 啟動驗證：檢查服務日誌確認無初始化錯誤
3. 功能測試：執行管理員指令確認審計服務正常運作

**Prevention Measures**:
1. 新增服務時確保遵循完整的初始化檢查清單
2. 為所有服務依賴建立明確的文檔化流程
3. 在代碼審查中特別檢查服務初始化的完整性

**Source**:
- main.rs: import 語句、create_services 函數和 Services 結構體修改
- Discord Gateway 配置更新

## 管理員權限認證問題 (2025-10-08)

**Error Type**: Permission System Bug

**Context**:
- 原有的 AdminService::verify_admin_permission 方法只檢查硬編碼的管理員列表
- 完全忽略 Discord 的原生權限系統
- 伺服器擁有者和具有 Administrator 權限的用戶無法執行管理員命令

**Root Cause**:
1. AdminService 權限檢查邏輯不完整，僅依賴預設授權列表
2. 缺少 Discord 原生權限系統整合
3. CommandResult 結構缺少伺服器資訊傳遞
4. Discord Gateway 未提供必要的 Context 進行權限驗證

**Solution**:
1. **擴展權限檢查邏輯**:
   - 新增 verify_admin_permission_with_discord 方法
   - 實現雙重驗證機制：授權列表 + Discord 原生權限
   - 檢查伺服器擁有者身份
   - 檢查 Administrator 和 MANAGE_GUILD 權限
   - 提供詳細的權限檢查日誌

2. **更新資料流架構**:
   - 修改 CommandResult 結構，新增 guild_id 和 discord_context 字段
   - 更新 Discord Gateway 傳遞必要的 Context 和伺服器資訊
   - 修改 ServiceRouter 使用新的權限檢查方法

3. **向後兼容性**:
   - 保留原有的 verify_admin_permission 方法作為回退選項
   - 確保在無 Discord Context 時仍能正常運作

**Permission Verification Priority**:
1. 授權列表檢查：用戶是否在預設的管理員列表中
2. 伺服器擁有者檢查：用戶是否為伺服器擁有者
3. Discord 權限檢查：用戶是否具有 Administrator 或 MANAGE_GUILD 權限
4. 回退機制：如果 Discord 權限檢查失敗，回退到僅檢查授權列表

**Verification Steps**:
1. 編譯檢查：`cargo build --release`
2. 啟動驗證：確認機器人成功連接到 Discord
3. 權限測試：驗證伺服器擁有者和管理員角色能執行管理員命令

**Prevention Measures**:
1. 權限系統設計應整合平台原生權限機制
2. 實施多層次權限驗證提高安全性
3. 保留向後兼容性確保系統穩定性
4. 完整記錄權限檢查過程便於除錯

**Source**:
- src/services/admin_service.rs: verify_admin_permission_with_discord 方法實現
- src/discord_gateway/service_router.rs: ServiceRouter 權限檢查更新
- src/discord_gateway/command_parser.rs: CommandResult 結構修改