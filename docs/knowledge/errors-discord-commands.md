# Discord Commands Errors

## 命令無法識別錯誤 (2025-10-08)

**Error Type**: Command Registration Missing

**Context**: 用戶執行 `!sync_member` 命令時出現 "Unknown command: sync_member" 錯誤，表明命令註冊系統存在完整鏈路缺失問題。

**Root Cause**: 命令註冊系統在多個層面存在缺失：
1. `Command` 枚舉缺少對應變體
2. 命令解析器沒有註冊命令字符串映射
3. 命令註冊表缺少命令條目
4. 服務路由器沒有對應的處理邏輯

**Solution**: 修復需要確保命令的完整鏈路實現：
1. 在 `src/discord_gateway/command_parser.rs` 中：
   - 添加 `SyncMembers` 變體到 `Command` 枚舉
   - 在 `with_prefix()` 方法中添加 `"sync_members" => Command::SyncMembers` 映射

2. 在 `src/discord_gateway/command_registry.rs` 中：
   - 在 `new()` 方法中註冊命令
   - 添加適當的命令描述

3. 在 `src/discord_gateway/service_router.rs` 中：
   - 在 `route_command()` 方法中添加處理分支
   - 實現 `handle_sync_members_command()` 方法
   - 包含管理員權限驗證和服務調用邏輯

4. 在 `src/command_router.rs` 中：
   - 在 `requires_user_account()` 方法中添加命令的帳戶驗證配置

**Prevention**:
- 實施命令註冊檢查清單，確保新命令包含所有必要組件
- 添加編譯時測試驗證所有已註冊命令都有完整的處理鏈路
- 建立命令開發模板，確保開發者不會遺漏任何關鍵步驟

**Source**: src/discord_gateway/command_parser.rs, src/discord_gateway/command_registry.rs, src/discord_gateway/service_router.rs, src/command_router.rs

## 管理員權限雙重驗證問題 (2025-10-08)

**Error Type**: Permission Check Inconsistency

**Context**: 伺服器管理員執行 !sync_member 指令時被拒絕，顯示權限不足，但其他管理員指令可以正常使用

**Root Cause**: 系統存在雙重權限檢查不一致問題：
1. Discord Gateway 層使用 `verify_admin_permission_with_discord`（檢查授權列表 + Discord 權限）
2. Service 層在 `coordinate_admin_operation` 中使用 `verify_admin_permission`（僅檢查授權列表）
3. 伺服器管理員在第二層檢查時失敗，因為他們不在授權列表中但具有 Discord 管理員權限

**Solution**: 實施統一的權限檢查流程：
1. 修改 `AdminService` 的 `coordinate_admin_operation` 函數，添加 `skip_permission_check` 參數
2. 在 `ServiceRouter` 中對已在 Discord Gateway 層驗證過權限的調用使用 `skip_permission_check = true`
3. 保持向後兼容性，添加 `coordinate_admin_operation_legacy` 方法

**Implementation**:
```rust
// 在 src/services/admin_service.rs 中
pub async fn coordinate_admin_operation(
    &self,
    ctx: &Context,
    guild_id: GuildId,
    executor_user_id: UserId,
    operation: &str,
    details: &str,
    skip_permission_check: bool, // 新增參數
) -> Result<(), Error> {
    if !skip_permission_check {
        self.verify_admin_permission(guild_id, executor_user_id).await?;
    }
    // ... 其他邏輯
}
```

**Prevention**:
- 確保所有管理員指令使用統一的權限檢查流程
- 在 Discord Gateway 層完成權限驗證後，避免在 Service 層重複檢查
- 為需要跳過權限檢查的場景提供明確的參數控制

**Source**: src/services/admin_service.rs, src/discord_gateway/service_router.rs, tests/sync_member_permission_simple_test.rs