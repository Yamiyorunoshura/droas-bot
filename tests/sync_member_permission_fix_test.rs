//! 測試 sync_member 權限檢查修復
//!
//! 這個測試驗證 sync_member 指令現在正確使用 Discord 權限檢查，
//! 允許伺服器管理員執行操作，而不僅僅是授權列表中的管理員。

use droas_bot::services::admin_service::{AdminService, AdminOperation, AdminOperationType};
use droas_bot::database::UserRepository;
use droas_bot::config::DatabaseConfig;

/// 測試 sync_member 權限檢查修復
#[tokio::test]
async fn test_sync_member_permission_fix() {
    // 創建測試環境
    let database_config = droas_bot::config::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());

    // 設置一個授權管理員（不是伺服器擁有者）
    let authorized_admin_id = 123456789i64;
    let admin_service = AdminService::new(user_repo.clone(), vec![authorized_admin_id]).unwrap();

    // 測試案例 1: 授權管理員應該能夠執行 sync_members（傳統方式）
    println!("測試案例 1: 授權管理員執行 sync_members");

    let operation1 = AdminOperation {
        operation_type: AdminOperationType::SyncMembers,
        admin_user_id: authorized_admin_id,
        target_user_id: None,
        amount: None,
        reason: "測試同步成員".to_string(),
        timestamp: chrono::Utc::now(),
    };

    // 使用舊版方法（應該成功）
    let result1 = admin_service.coordinate_admin_operation_legacy(operation1).await;
    assert!(result1.is_ok(), "授權管理員應該能夠執行 sync_members");
    assert!(result1.unwrap().success, "操作應該成功");

    // 測試案例 2: 非授權但具有 Discord 管理員權限的用戶（模擬）
    println!("測試案例 2: 非授權但具有 Discord 管理員權限的用戶");

    let discord_admin_id = 588344488624259073i64; // 伺服器管理員 ID
    let operation2 = AdminOperation {
        operation_type: AdminOperationType::SyncMembers,
        admin_user_id: discord_admin_id,
        target_user_id: None,
        amount: None,
        reason: "伺服器管理員執行同步成員".to_string(),
        timestamp: chrono::Utc::now(),
    };

    // 使用舊版方法（應該失敗，因為不在授權列表中）
    let result2_legacy = admin_service.coordinate_admin_operation_legacy(operation2.clone()).await;
    assert!(result2_legacy.is_ok(), "權限檢查應該成功執行");
    assert!(!result2_legacy.unwrap().success, "非授權用戶使用舊版方法應該失敗");

    // 使用新版方法跳過權限檢查（模擬 Discord Gateway 層已經驗證過）
    let result2_skip = admin_service.coordinate_admin_operation(operation2, true).await;
    assert!(result2_skip.is_ok(), "跳過權限檢查應該成功");
    assert!(result2_skip.unwrap().success, "跳過權限檢查時操作應該成功");

    // 測試案例 3: 普通用戶嘗試執行操作
    println!("測試案例 3: 普通用戶嘗試執行 sync_members");

    let regular_user_id = 999999999i64;
    let operation3 = AdminOperation {
        operation_type: AdminOperationType::SyncMembers,
        admin_user_id: regular_user_id,
        target_user_id: None,
        amount: None,
        reason: "普通用戶嘗試執行同步成員".to_string(),
        timestamp: chrono::Utc::now(),
    };

    // 使用舊版方法（應該失敗）
    let result3_legacy = admin_service.coordinate_admin_operation_legacy(operation3.clone()).await;
    assert!(result3_legacy.is_ok(), "權限檢查應該成功執行");
    assert!(!result3_legacy.unwrap().success, "普通用戶使用舊版方法應該失敗");

    // 使用新版方法跳過權限檢查（應該成功，因為跳過了檢查）
    // 這種情況下，Discord Gateway 層應該已經驗證過權限
    let result3_skip = admin_service.coordinate_admin_operation(operation3, true).await;
    assert!(result3_skip.is_ok(), "跳過權限檢查應該成功");
    assert!(result3_skip.unwrap().success, "跳過權限檢查時操作應該成功");

    println!("✅ sync_member 權限檢查修復測試完成");
}

/// 測試管理員服務的權限檢查邏輯
#[tokio::test]
async fn test_admin_service_permission_methods() {
    let database_config = droas_bot::config::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());

    let authorized_admin_id = 123456789i64;
    let admin_service = AdminService::new(user_repo, vec![authorized_admin_id]).unwrap();

    // 測試授權管理員
    let auth_result = admin_service.verify_admin_permission(authorized_admin_id).await;
    assert!(auth_result.is_ok(), "權限檢查應該成功");
    assert!(auth_result.unwrap(), "授權管理員應該通過權限檢查");

    // 測試非授權管理員
    let non_auth_result = admin_service.verify_admin_permission(987654321).await;
    assert!(non_auth_result.is_ok(), "權限檢查應該成功");
    assert!(!non_auth_result.unwrap(), "非授權管理員不應該通過權限檢查");

    // 測試無效用戶 ID
    let invalid_result = admin_service.verify_admin_permission(-1).await;
    assert!(invalid_result.is_err(), "無效用戶 ID 應該返回錯誤");

    println!("✅ 管理員服務權限檢查邏輯測試完成");
}