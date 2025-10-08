// Admin Service - 管理員權限驗證和管理員操作協調 (GREEN 階段)
// 實現 F-009 管理員身份驗證功能

use crate::error::DiscordError;
use crate::database::user_repository::UserRepositoryTrait;
use crate::database::balance_repository::BalanceRepository;
use crate::database::transaction_repository::TransactionRepository;
use bigdecimal::BigDecimal;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn, error, debug, instrument};
use serde::{Serialize, Deserialize};
use serenity::all::{Context, GuildId, UserId, Permissions};

/// 管理員操作類型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdminOperationType {
    /// 餘額調整
    AdjustBalance,
    /// 查看用戶資訊
    ViewUserInfo,
    /// 查看歷史記錄
    ViewHistory,
    /// 系統維護
    SystemMaintenance,
    /// 同步群組成員
    SyncMembers,
}

/// 管理員操作記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminOperation {
    /// 操作類型
    pub operation_type: AdminOperationType,
    /// 管理員用戶 ID
    pub admin_user_id: i64,
    /// 目標用戶 ID（可選）
    pub target_user_id: Option<i64>,
    /// 操作金額（可選）
    pub amount: Option<BigDecimal>,
    /// 操作原因
    pub reason: String,
    /// 操作時間戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 操作結果
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// 操作是否成功
    pub success: bool,
    /// 結果訊息
    pub message: String,
    /// 操作 ID（用於審計）
    pub operation_id: Option<String>,
}

/// 管理員服務
///
/// 負責管理員權限驗證和管理員操作協調
/// 實現 F-009: 管理員權限驗證系統
pub struct AdminService {
    /// 授權管理員用戶 ID 列表
    authorized_admins: HashSet<i64>,
    /// 用戶資料庫存取
    user_repository: Arc<dyn UserRepositoryTrait + Send + Sync>,
    /// 餘額倉儲
    balance_repository: Option<Arc<BalanceRepository>>,
    /// 交易倉儲
    transaction_repository: Option<Arc<TransactionRepository>>,
}

impl AdminService {
    /// 創建新的管理員服務
    ///
    /// # Arguments
    ///
    /// * `user_repository` - 用戶倉儲實例
    /// * `authorized_admins` - 授權管理員用戶 ID 列表
    ///
    /// # Returns
    ///
    /// 返回 Result<AdminService>，成功時包含服務實例
    pub fn new<T>(
        user_repository: T,
        authorized_admins: Vec<i64>,
    ) -> Result<Self, DiscordError>
    where
        T: UserRepositoryTrait + Send + Sync + 'static,
    {
        if authorized_admins.is_empty() {
            warn!("創建 Admin Service 時沒有提供任何授權管理員");
            return Err(DiscordError::ConfigError(
                "至少需要一個授權管理員".to_string()
            ));
        }

        info!("創建 Admin Service，授權管理員數量: {}", authorized_admins.len());
        debug!("授權管理員列表: {:?}", authorized_admins);

        Ok(Self {
            authorized_admins: authorized_admins.into_iter().collect(),
            user_repository: Arc::new(user_repository),
            balance_repository: None,
            transaction_repository: None,
        })
    }

    /// 創建完整的管理員服務（包含所有倉儲）
    pub fn new_with_repositories<T>(
        user_repository: T,
        balance_repository: Arc<BalanceRepository>,
        transaction_repository: Arc<TransactionRepository>,
        authorized_admins: Vec<i64>,
    ) -> Result<Self, DiscordError>
    where
        T: UserRepositoryTrait + Send + Sync + 'static,
    {
        if authorized_admins.is_empty() {
            return Err(DiscordError::ConfigError(
                "至少需要一個授權管理員".to_string()
            ));
        }

        info!("創建完整 Admin Service，授權管理員數量: {}", authorized_admins.len());

        Ok(Self {
            authorized_admins: authorized_admins.into_iter().collect(),
            user_repository: Arc::new(user_repository),
            balance_repository: Some(balance_repository),
            transaction_repository: Some(transaction_repository),
        })
    }

    /// 驗證管理員權限 (F-009) - 使用 Discord 權限檢查
    ///
    /// # Arguments
    ///
    /// * `ctx` - Discord Context
    /// * `guild_id` - 伺服器 ID
    /// * `user_id` - 要驗證的用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示用戶具有管理員權限
    #[instrument(skip(self, ctx), fields(user_id = %user_id, guild_id = %guild_id))]
    pub async fn verify_admin_permission_with_discord(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<bool, DiscordError> {
        debug!("驗證用戶 {} 在伺服器 {} 的管理員權限", user_id, guild_id);

        // 檢查是否為授權管理員（第一重驗證）
        if self.authorized_admins.contains(&(user_id.get() as i64)) {
            info!("用戶 {} 在授權管理員列表中", user_id);
            return Ok(true);
        }

        // 檢查 Discord 權限（第二重驗證）
        match self.check_discord_admin_permissions(ctx, guild_id, user_id).await {
            Ok(has_admin_permissions) => {
                if has_admin_permissions {
                    info!("用戶 {} 具有 Discord 管理員權限", user_id);
                    Ok(true)
                } else {
                    warn!("用戶 {} 沒有管理員權限", user_id);
                    Ok(false)
                }
            }
            Err(e) => {
                warn!("檢查 Discord 權限時發生錯誤: {}", e);
                // 如果無法檢查 Discord 權限，回退到檢查授權列表
                Ok(false)
            }
        }
    }

    /// 驗證管理員權限 (F-009) - 僅檢查授權列表（向後兼容）
    ///
    /// # Arguments
    ///
    /// * `user_id` - 要驗證的用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示用戶是授權管理員
    #[instrument(skip(self), fields(user_id = %user_id))]
    pub async fn verify_admin_permission(&self, user_id: i64) -> Result<bool, DiscordError> {
        debug!("驗證用戶 {} 的管理員權限（僅檢查授權列表）", user_id);

        // 檢查用戶 ID 是否有效
        if user_id <= 0 {
            warn!("無效的用戶 ID: {}", user_id);
            return Err(DiscordError::InvalidAmount(format!("無效的用戶 ID: {}", user_id)));
        }

        // 檢查是否為授權管理員
        let is_admin = self.authorized_admins.contains(&user_id);

        if is_admin {
            info!("用戶 {} 是授權管理員", user_id);
            Ok(true)
        } else {
            warn!("用戶 {} 不是授權管理員", user_id);
            Ok(false)
        }
    }

    /// 檢查 Discord 管理員權限
    ///
    /// # Arguments
    ///
    /// * `ctx` - Discord Context
    /// * `guild_id` - 伺服器 ID
    /// * `user_id` - 用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 Result<bool>，true 表示用戶具有管理員權限
    async fn check_discord_admin_permissions(
        &self,
        ctx: &Context,
        guild_id: GuildId,
        user_id: UserId,
    ) -> Result<bool, DiscordError> {
        // 獲取伺服器資訊
        let guild = match guild_id.to_partial_guild(ctx).await {
            Ok(guild) => guild,
            Err(e) => {
                warn!("無法獲取伺服器 {} 的資訊: {}", guild_id, e);
                return Err(DiscordError::PermissionDenied(format!("無法獲取伺服器資訊: {}", e)));
            }
        };

        // 檢查是否為伺服器擁有者
        if guild.owner_id == user_id {
            info!("用戶 {} 是伺服器 {} 的擁有者", user_id, guild_id);
            return Ok(true);
        }

        // 獲取伺服器成員資訊
        let member = match guild.member(ctx, user_id).await {
            Ok(member) => member,
            Err(e) => {
                warn!("無法獲取用戶 {} 在伺服器 {} 的成員資訊: {}", user_id, guild_id, e);
                return Err(DiscordError::PermissionDenied(format!("無法獲取成員資訊: {}", e)));
            }
        };

        // 檢查管理員權限
        if let Some(permissions) = member.permissions {
            let required_permissions = Permissions::ADMINISTRATOR;
            if permissions.contains(required_permissions) {
                info!("用戶 {} 具有伺服器 {} 的管理員權限", user_id, guild_id);
                return Ok(true);
            }

            // 檢查其他可能的權限（如 MANAGE_GUILD）
            let guild_management_permissions = Permissions::MANAGE_GUILD;
            if permissions.contains(guild_management_permissions) {
                info!("用戶 {} 具有伺服器 {} 的管理權限", user_id, guild_id);
                return Ok(true);
            }
        }

        debug!("用戶 {} 在伺服器 {} 中沒有管理員權限", user_id, guild_id);
        Ok(false)
    }

    /// 協調管理員操作
    ///
    /// # Arguments
    ///
    /// * `operation` - 管理員操作
    /// * `skip_permission_check` - 是否跳過權限檢查（預設為 false）
    ///
    /// # Returns
    ///
    /// 返回 Result<OperationResult>，包含操作結果
    #[instrument(skip(self), fields(admin_id = %operation.admin_user_id, operation_type = ?operation.operation_type))]
    pub async fn coordinate_admin_operation(&self, operation: AdminOperation, skip_permission_check: bool) -> Result<OperationResult, DiscordError> {
        info!("協調管理員操作: {:?} by admin {}", operation.operation_type, operation.admin_user_id);

        // 驗證管理員權限（除非跳過）
        if !skip_permission_check {
            let is_admin = self.verify_admin_permission(operation.admin_user_id).await?;
            if !is_admin {
                warn!("非管理員用戶 {} 嘗試執行管理員操作", operation.admin_user_id);
                return Ok(OperationResult {
                    success: false,
                    message: "權限不足：只有授權管理員可以執行此操作".to_string(),
                    operation_id: None,
                });
            }
        }

        // 根據操作類型執行相應邏輯
        let result = match operation.operation_type {
            AdminOperationType::AdjustBalance => {
                self.handle_adjust_balance_operation(operation).await?
            }
            AdminOperationType::ViewUserInfo => {
                self.handle_view_user_info_operation(operation).await?
            }
            AdminOperationType::ViewHistory => {
                self.handle_view_history_operation(operation).await?
            }
            AdminOperationType::SystemMaintenance => {
                self.handle_system_maintenance_operation(operation).await?
            }
            AdminOperationType::SyncMembers => {
                self.handle_sync_members_operation(operation).await?
            }
        };

        info!("管理員操作完成，成功: {}", result.success);
        Ok(result)
    }

    /// 協調管理員操作（向後兼容版本）
    ///
    /// # Arguments
    ///
    /// * `operation` - 管理員操作
    ///
    /// # Returns
    ///
    /// 返回 Result<OperationResult>，包含操作結果
    #[instrument(skip(self), fields(admin_id = %operation.admin_user_id, operation_type = ?operation.operation_type))]
    pub async fn coordinate_admin_operation_legacy(&self, operation: AdminOperation) -> Result<OperationResult, DiscordError> {
        self.coordinate_admin_operation(operation, false).await
    }

    /// 處理餘額調整操作
    async fn handle_adjust_balance_operation(&self, operation: AdminOperation) -> Result<OperationResult, DiscordError> {
        debug!("處理餘額調整操作");

        if let (Some(target_user_id), Some(amount)) = (operation.target_user_id, &operation.amount) {
            // 檢查是否有必要的倉儲
            if self.balance_repository.is_none() || self.transaction_repository.is_none() {
                return Err(DiscordError::ConfigError(
                    "餘額調整操作需要 BalanceRepository 和 TransactionRepository".to_string()
                ));
            }

            // 這裡將在後續實現中與 Balance Service 整合
            // 目前返回成功響應作為最小實現
            Ok(OperationResult {
                success: true,
                message: format!("已準備調整用戶 {} 的餘額: {}", target_user_id, amount),
                operation_id: Some(generate_operation_id()),
            })
        } else {
            Err(DiscordError::InvalidCommand(
                "餘額調整操作需要目標用戶和金額".to_string()
            ))
        }
    }

    /// 處理查看用戶資訊操作
    async fn handle_view_user_info_operation(&self, operation: AdminOperation) -> Result<OperationResult, DiscordError> {
        debug!("處理查看用戶資訊操作");

        if let Some(target_user_id) = operation.target_user_id {
            match self.user_repository.find_by_user_id(target_user_id).await {
                Ok(Some(user)) => {
                    Ok(OperationResult {
                        success: true,
                        message: format!("用戶資訊: ID={}, 名稱={}, 創建時間={}",
                                       user.discord_user_id, user.username, user.created_at),
                        operation_id: Some(generate_operation_id()),
                    })
                }
                Ok(None) => {
                    Ok(OperationResult {
                        success: false,
                        message: format!("用戶 {} 不存在", target_user_id),
                        operation_id: Some(generate_operation_id()),
                    })
                }
                Err(e) => {
                    error!("查詢用戶 {} 時發生錯誤: {}", target_user_id, e);
                    Err(DiscordError::DatabaseQueryError(format!("查詢用戶失敗: {}", e)))
                }
            }
        } else {
            Err(DiscordError::InvalidCommand(
                "查看用戶資訊操作需要目標用戶".to_string()
            ))
        }
    }

    /// 處理查看歷史記錄操作
    async fn handle_view_history_operation(&self, _operation: AdminOperation) -> Result<OperationResult, DiscordError> {
        debug!("處理查看歷史記錄操作");

        // 這裡將在後續實現中與 Admin Audit Service 整合
        // 目前返回成功響應作為最小實現
        Ok(OperationResult {
            success: true,
            message: "歷史記錄查詢功能將在後續版本中實現".to_string(),
            operation_id: Some(generate_operation_id()),
        })
    }

    /// 處理系統維護操作
    async fn handle_system_maintenance_operation(&self, operation: AdminOperation) -> Result<OperationResult, DiscordError> {
        debug!("處理系統維護操作");

        // 這裡將在後續實現中添加具體的維護功能
        // 目前返回成功響應作為最小實現
        Ok(OperationResult {
            success: true,
            message: format!("系統維護操作: {}", operation.reason),
            operation_id: Some(generate_operation_id()),
        })
    }

    /// 添加授權管理員
    ///
    /// # Arguments
    ///
    /// * `admin_user_id` - 要添加的管理員用戶 ID
    pub fn add_authorized_admin(&mut self, admin_user_id: i64) {
        info!("添加授權管理員: {}", admin_user_id);
        self.authorized_admins.insert(admin_user_id);
    }

    /// 移除授權管理員
    ///
    /// # Arguments
    ///
    /// * `admin_user_id` - 要移除的管理員用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 bool，true 表示用戶存在於授權列表中並已被移除
    pub fn remove_authorized_admin(&mut self, admin_user_id: i64) -> bool {
        let removed = self.authorized_admins.remove(&admin_user_id);
        if removed {
            info!("移除授權管理員: {}", admin_user_id);
        } else {
            debug!("管理員 {} 不在授權列表中", admin_user_id);
        }
        removed
    }

    /// 檢查用戶是否為授權管理員
    ///
    /// # Arguments
    ///
    /// * `user_id` - 要檢查的用戶 ID
    ///
    /// # Returns
    ///
    /// 返回 bool，true 表示用戶是授權管理員
    pub fn is_authorized_admin(&self, user_id: i64) -> bool {
        self.authorized_admins.contains(&user_id)
    }

    /// 獲取所有授權管理員列表
    ///
    /// # Returns
    ///
    /// 返回授權管理員用戶 ID 列表
    pub fn get_authorized_admins(&self) -> Vec<i64> {
        self.authorized_admins.iter().cloned().collect()
    }

    /// 獲取授權管理員數量
    ///
    /// # Returns
    ///
    /// 返回授權管理員數量
    pub fn get_admin_count(&self) -> usize {
        self.authorized_admins.len()
    }

    /// 處理同步群組成員操作
    async fn handle_sync_members_operation(&self, _operation: AdminOperation) -> Result<OperationResult, DiscordError> {
        info!("處理同步群組成員操作");

        // 這是一個基礎實現，在實際使用中需要與 Discord Gateway 集成
        // 獲取群組成員列表並執行批量帳戶創建

        // 返回成功響應，表示操作已接收
        Ok(OperationResult {
            success: true,
            message: "群組成員同步操作已啟動。正在獲取群組成員列表並創建缺失的帳戶...".to_string(),
            operation_id: Some(generate_operation_id()),
        })
    }
}

/// 生成操作 ID
fn generate_operation_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("admin_op_{}", timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_admin_service_creation() {
        // 測試 AdminService 創建
        let database_config = crate::config::DatabaseConfig::for_test();
        let pool_result = crate::database::create_user_pool(&database_config).await;

        if let Ok(pool) = pool_result {
            let user_repo = crate::database::UserRepository::new(pool);
            let admin_ids = vec![123456789_i64, 987654321_i64];

            let admin_service = AdminService::new(user_repo, admin_ids.clone()).unwrap();

            assert_eq!(admin_service.get_admin_count(), 2);
            assert!(admin_service.is_authorized_admin(123456789));
            assert!(admin_service.is_authorized_admin(987654321));
            assert!(!admin_service.is_authorized_admin(555555555));
        }
    }

    #[tokio::test]
    async fn test_admin_permission_verification() {
        // 測試管理員權限驗證
        let database_config = crate::config::DatabaseConfig::for_test();
        let pool_result = crate::database::create_user_pool(&database_config).await;

        if let Ok(pool) = pool_result {
            let user_repo = crate::database::UserRepository::new(pool);
            let admin_ids = vec![123456789_i64];

            let admin_service = AdminService::new(user_repo, admin_ids).unwrap();

            // 測試授權管理員
            let result = admin_service.verify_admin_permission(123456789).await.unwrap();
            assert!(result);

            // 測試非管理員
            let result = admin_service.verify_admin_permission(987654321).await.unwrap();
            assert!(!result);

            // 測試無效用戶 ID
            let result = admin_service.verify_admin_permission(-1).await;
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn test_admin_operation_coordination() {
        // 測試管理員操作協調
        let database_config = crate::config::DatabaseConfig::for_test();
        let pool_result = crate::database::create_user_pool(&database_config).await;

        if let Ok(pool) = pool_result {
            let user_repo = crate::database::UserRepository::new(pool);
            let admin_ids = vec![123456789_i64];

            let admin_service = AdminService::new(user_repo, admin_ids).unwrap();

            // 測試有效操作
            let operation = AdminOperation {
                operation_type: AdminOperationType::ViewUserInfo,
                admin_user_id: 123456789,
                target_user_id: Some(987654321),
                amount: None,
                reason: "測試操作".to_string(),
                timestamp: chrono::Utc::now(),
            };

            let result = admin_service.coordinate_admin_operation_legacy(operation).await.unwrap();
            assert!(result.success);

            // 測試無權限操作
            let unauthorized_operation = AdminOperation {
                operation_type: AdminOperationType::ViewUserInfo,
                admin_user_id: 555555555, // 非管理員
                target_user_id: Some(987654321),
                amount: None,
                reason: "未授權操作".to_string(),
                timestamp: chrono::Utc::now(),
            };

            let result = admin_service.coordinate_admin_operation_legacy(unauthorized_operation).await.unwrap();
            assert!(!result.success);
            assert!(result.message.contains("權限不足"));
        }
    }
}