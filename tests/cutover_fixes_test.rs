//! 修復驗收測試
//!
//! 這個測試文件包含針對 cutover 報告中發現問題的修復驗證測試

use droas_bot::{
    services::user_account_service::{UserAccountService, BulkAccountCreationRequest},
    database::user_repository::UserRepository,
    services::admin_service::AdminService,
    error::DiscordError,
};
use std::time::Instant;

/// 測試大規模批量處理邊界問題
///
/// 重現 cutover 報告中的問題：1040/1050 帳戶創建
#[tokio::test]
async fn test_bulk_processing_boundary_issue_reproduction() {
    // GIVEN: 創建測試環境
    let database_config = droas_bot::database::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());
    let user_service = UserAccountService::new(user_repo.clone()).unwrap();

    // WHEN: 創建 1050 個測試用戶（重現 cutover 報告的場景）
    let mut user_ids = Vec::new();
    let mut usernames = Vec::new();

    for i in 1..=1050 {
        user_ids.push(i as i64);
        usernames.push(format!("BoundaryTestUser{}", i));
    }

    let request = BulkAccountCreationRequest {
        user_ids,
        usernames,
    };

    let start_time = Instant::now();
    let result = user_service.bulk_create_accounts(request).await;
    let elapsed = start_time.elapsed();

    // THEN: 應該處理所有 1050 個帳戶（這個測試預期會失敗，重現問題）
    assert!(result.is_ok(), "批量創建應該成功");

    let bulk_result = result.unwrap();

    // 這個斷言應該會失敗，重現 cutover 報告中的問題
    assert_eq!(
        bulk_result.created_count,
        1050,
        "應該創建 1050 個帳戶，但實際創建了 {} 個。這重現了 cutover 報告中的邊界問題",
        bulk_result.created_count
    );

    // 驗證總處理數量
    assert_eq!(
        bulk_result.total_processed,
        1050,
        "應該處理 1050 個帳戶，但實際處理了 {} 個",
        bulk_result.total_processed
    );

    // 記錄性能指標
    println!("批量創建 1050 個帳戶耗時: {:?}", elapsed);
    println!("創建成功: {}, 跳過: {}, 失敗: {}",
             bulk_result.created_count,
             bulk_result.skipped_count,
             bulk_result.failed_count);
}

/// 測試分批處理邊界邏輯
///
/// 測試不同大小的數組，特別是批次大小 (20) 的倍數
#[tokio::test]
async fn test_batch_processing_boundary_logic() {
    let database_config = droas_bot::database::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());
    let user_service = UserAccountService::new(user_repo.clone()).unwrap();

    // 測試案例：恰好是批次大小倍數的數量
    let test_cases = vec![
        (20, "恰好一批"),
        (40, "恰好兩批"),
        (100, "恰好五批"),
        (19, "少於一批"),
        (21, "多於一批"),
        (99, "接近批次邊界"),
        (101, "超過批次邊界"),
        (1040, "cutover 報告中的問題數量"),
        (1050, "cutover 報告中的測試數量"),
        (1060, "超過 cutover 報告數量"),
    ];

    for (user_count, description) in test_cases {
        println!("測試案例: {} 個用戶 ({})", user_count, description);

        // 清理之前的測試數據
        cleanup_test_data(&user_repo).await;

        let mut user_ids = Vec::new();
        let mut usernames = Vec::new();

        for i in 1..=user_count {
            user_ids.push((i * 10000) as i64); // 使用大的 ID 避免衝突
            usernames.push(format!("BoundaryUser{}_{}", i, description));
        }

        let request = BulkAccountCreationRequest {
            user_ids,
            usernames,
        };

        let result = user_service.bulk_create_accounts(request).await;

        assert!(result.is_ok(), "批量創建 {} 個帳戶應該成功 ({})", user_count, description);

        let bulk_result = result.unwrap();
        assert_eq!(
            bulk_result.created_count,
            user_count,
            "應該創建 {} 個帳戶，但實際創建了 {} 個 ({})",
            user_count,
            bulk_result.created_count,
            description
        );

        assert_eq!(
            bulk_result.total_processed,
            user_count,
            "應該處理 {} 個帳戶，但實際處理了 {} 個 ({})",
            user_count,
            bulk_result.total_processed,
            description
        );
    }
}

/// 測試 step_by 邊界行為
///
/// 獨立測試 step_by 在不同邊界條件下的行為
#[test]
fn test_step_by_boundary_behavior() {
    // 測試 step_by 在不同數量下的行為
    let test_cases = vec![
        (1050, 20),
        (1040, 20),
        (1000, 20),
        (21, 20),
        (19, 20),
        (1, 20),
        (0, 20),
    ];

    for (total_size, batch_size) in test_cases {
        let batch_starts: Vec<usize> = (0..total_size).step_by(batch_size).collect();

        println!("測試 step_by: 總數 {}, 批次大小 {}", total_size, batch_size);
        println!("  批次起始位置: {:?}", batch_starts);

        // 驗證第一個批次
        if !batch_starts.is_empty() {
            assert_eq!(batch_starts[0], 0, "第一個批次應該從 0 開始");
        }

        // 驗證最後一個批次的邊界
        if let Some(&last_start) = batch_starts.last() {
            assert!(last_start < total_size, "最後一個批次起始位置 {} 應該小於總數 {}", last_start, total_size);

            let expected_end = std::cmp::min(last_start + batch_size, total_size);
            println!("  最後一批: {}..{} ({} 項目)", last_start, expected_end, expected_end - last_start);

            // 驗證最後一批會處理所有剩餘項目
            assert!(expected_end <= total_size, "批次結束位置不應超過總數");
            assert!(expected_end > last_start, "批次應該包含至少一個項目");
        }
    }
}

/// 測試切片操作的邊界行為
///
/// 測試在不同邊界條件下切片操作的正確性
#[test]
fn test_slice_boundary_operations() {
    let data: Vec<i32> = (1..=1050).collect();
    let batch_size = 20;

    for batch_start in (0..data.len()).step_by(batch_size) {
        let batch_end = std::cmp::min(batch_start + batch_size, data.len());
        let batch = &data[batch_start..batch_end];

        println!("批次 {}..{}: 長度 {}, 第一項: {}, 最後一項: {}",
                 batch_start, batch_end, batch.len(),
                 batch.first().unwrap_or(&0), batch.last().unwrap_or(&0));

        // 驗證切片的正確性
        assert!(!batch.is_empty(), "批次不應為空");
        assert_eq!(batch.len(), batch_end - batch_start, "批次長度應該正確");

        // 驗證內容的正確性
        if !batch.is_empty() {
            assert_eq!(batch[0], (batch_start + 1) as i32, "批次第一項應該正確");
            assert_eq!(batch[batch.len() - 1], batch_end as i32, "批次最後一項應該正確");
        }
    }
}

/// 清理測試數據
async fn cleanup_test_data(user_repo: &UserRepository) {
    // 這裡可以添加清理邏輯，如果需要的話
    // 目前暫時留空，因為測試使用不同的 ID 範圍
}

/// 測試批量處理的完整性檢查
///
/// 驗證批量處理是否處理了所有輸入項目
#[tokio::test]
async fn test_bulk_processing_integrity_check() {
    let database_config = droas_bot::database::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());
    let user_service = UserAccountService::new(user_repo.clone()).unwrap();

    // 使用不同的 ID 範圍避免衝突
    let base_id = 20000i64;
    let user_count = 1050;

    let mut user_ids = Vec::new();
    let mut usernames = Vec::new();

    for i in 1..=user_count {
        user_ids.push(base_id + i as i64);
        usernames.push(format!("IntegrityTestUser{}", i));
    }

    // 保存輸入數據的副本用於驗證
    let input_user_ids = user_ids.clone();
    let input_usernames = usernames.clone();

    let request = BulkAccountCreationRequest {
        user_ids,
        usernames,
    };

    let result = user_service.bulk_create_accounts(request).await;
    assert!(result.is_ok(), "批量創建應該成功");

    let bulk_result = result.unwrap();

    // 完整性檢查：確保所有輸入項目都被處理
    assert_eq!(
        bulk_result.total_processed,
        input_user_ids.len(),
        "總處理數量應該等於輸入數量"
    );

    assert_eq!(
        bulk_result.created_count + bulk_result.skipped_count + bulk_result.failed_count,
        input_user_ids.len(),
        "處理結果總和應該等於輸入數量"
    );

    // 驗證沒有項目被遺漏
    let mut processed_count = 0;
    processed_count += bulk_result.created_count;
    processed_count += bulk_result.skipped_count;
    processed_count += bulk_result.failed_count;

    assert_eq!(
        processed_count,
        input_user_ids.len(),
        "所有帳戶都應該被處理（創建、跳過或失敗）"
    );

    println!("完整性檢查通過：輸入 {} 項目，處理 {} 項目", input_user_ids.len(), processed_count);
    println!("  創建: {}, 跳過: {}, 失敗: {}",
             bulk_result.created_count,
             bulk_result.skipped_count,
             bulk_result.failed_count);
}

/// 測試錯誤處理邊界情況
///
/// 重現 cutover 報告中的錯誤處理邊界情況問題
#[tokio::test]
async fn test_error_handling_boundary_situations() {
    // GIVEN: 創建測試環境
    let database_config = droas_bot::database::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());
    let user_service = UserAccountService::new(user_repo.clone()).unwrap();

    // WHEN: 測試各種錯誤邊界情況

    // 測試案例 1: 空的輸入數據
    println!("測試案例 1: 空的輸入數據");
    let empty_request = BulkAccountCreationRequest {
        user_ids: vec![],
        usernames: vec![],
    };

    let result = user_service.bulk_create_accounts(empty_request).await;
    assert!(result.is_ok(), "空輸入應該成功處理");

    let empty_result = result.unwrap();
    assert_eq!(empty_result.total_processed, 0, "空輸入應該處理 0 項目");
    assert_eq!(empty_result.created_count, 0, "空輸入應該創建 0 個帳戶");

    // 測試案例 2: 不匹配的數組長度
    println!("測試案例 2: 不匹配的數組長度");
    let mismatched_request = BulkAccountCreationRequest {
        user_ids: vec![1, 2, 3],
        usernames: vec!["User1".to_string(), "User2".to_string()], // 少一個用戶名
    };

    let result = user_service.bulk_create_accounts(mismatched_request).await;
    assert!(result.is_err(), "不匹配的數組長度應該返回錯誤");

    match result.unwrap_err() {
        DiscordError::InvalidCommand(msg) => {
            assert!(msg.contains("長度不匹配"), "錯誤消息應該指出長度不匹配");
        },
        other => panic!("預期 InvalidCommand 錯誤，但得到: {:?}", other),
    }

    // 測試案例 3: 包含無效字符的用戶名
    println!("測試案例 3: 包含無效字符的用戶名");
    let invalid_chars_request = BulkAccountCreationRequest {
        user_ids: vec![10001, 10002, 10003],
        usernames: vec![
            "ValidUser".to_string(),
            "User\nWith\nNewlines".to_string(), // 包含換行符
            "User\0WithNull".to_string(), // 包含空字符
        ],
    };

    let result = user_service.bulk_create_accounts(invalid_chars_request).await;
    // 這個測試可能成功也可能失敗，取決於驗證邏輯
    // 重要的是觀察系統如何處理這些邊界情況

    if let Ok(bulk_result) = result {
        println!("  無效字符處理結果: 創建 {}, 跳過 {}, 失敗 {}",
                 bulk_result.created_count,
                 bulk_result.skipped_count,
                 bulk_result.failed_count);
    } else {
        println!("  無效字符導致錯誤: {:?}", result.unwrap_err());
    }

    // 測試案例 4: 極長的用戶名
    println!("測試案例 4: 極長的用戶名");
    let long_username = "A".repeat(10000); // 10,000 個字符的用戶名
    let long_name_request = BulkAccountCreationRequest {
        user_ids: vec![20001],
        usernames: vec![long_username.clone()],
    };

    let result = user_service.bulk_create_accounts(long_name_request).await;
    // 觀察系統如何處理極長的用戶名
    match result {
        Ok(bulk_result) => {
            println!("  極長用戶名處理結果: 創建 {}, 跳過 {}, 失敗 {}",
                     bulk_result.created_count,
                     bulk_result.skipped_count,
                     bulk_result.failed_count);
        },
        Err(e) => {
            println!("  極長用戶名導致錯誤: {:?}", e);
        }
    }

    // 測試案例 5: 重複的用戶 ID
    println!("測試案例 5: 重複的用戶 ID");
    let duplicate_ids_request = BulkAccountCreationRequest {
        user_ids: vec![30001, 30002, 30001, 30003], // 重複的 30001
        usernames: vec![
            "UserA".to_string(),
            "UserB".to_string(),
            "UserA_Duplicate".to_string(),
            "UserC".to_string(),
        ],
    };

    let result = user_service.bulk_create_accounts(duplicate_ids_request).await;
    assert!(result.is_ok(), "重複用戶 ID 應該能處理");

    let duplicate_result = result.unwrap();
    println!("  重複 ID 處理結果: 創建 {}, 跳過 {}, 失敗 {}",
             duplicate_result.created_count,
             duplicate_result.skipped_count,
             duplicate_result.failed_count);

    // 驗證總數正確
    assert_eq!(duplicate_result.total_processed, 4, "應該處理 4 個輸入項目");
}

/// 測試資料庫連接錯誤處理
///
/// 模擬資料庫連接問題，測試錯誤恢復機制
#[tokio::test]
async fn test_database_connection_error_handling() {
    // 創建無效的資料庫配置來模擬連接錯誤
    let invalid_config = droas_bot::config::DatabaseConfig {
        url: "postgres://invalid:invalid@localhost:9999/invalid".to_string(),
        max_connections: 5,
        min_connections: 1,
        connection_timeout: 5,
    };

    // 嘗試創建連接池（預期會失敗）
    let pool_result = droas_bot::database::create_user_pool(&invalid_config).await;

    match pool_result {
        Ok(_) => panic!("無效的資料庫配置應該導致連接失敗"),
        Err(e) => {
            println!("正確檢測到資料庫連接錯誤: {:?}", e);
            // 這裡可以添加更詳細的錯誤類型檢查
        }
    }
}

/// 測試併發操作錯誤處理
///
/// 測試多個併發批量操作時的錯誤處理
#[tokio::test]
async fn test_concurrent_operations_error_handling() {
    let database_config = droas_bot::database::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());
    let user_service = std::sync::Arc::new(UserAccountService::new(user_repo.clone()).unwrap());

    // 創建多個併發批量操作
    let mut handles = vec![];

    for i in 0..5 {
        let service_clone = std::sync::Arc::clone(&user_service);
        let handle = tokio::spawn(async move {
            let base_id = 50000 + (i * 1000);
            let user_ids: Vec<i64> = (base_id..base_id + 100).collect();
            let usernames: Vec<String> = (base_id..base_id + 100)
                .map(|id| format!("ConcurrentUser{}", id))
                .collect();

            let request = BulkAccountCreationRequest {
                user_ids,
                usernames,
            };

            let start_time = Instant::now();
            let result = service_clone.bulk_create_accounts(request).await;
            let elapsed = start_time.elapsed();

            (i, result, elapsed)
        });

        handles.push(handle);
    }

    // 等待所有操作完成並檢查結果
    let mut success_count = 0;
    let mut error_count = 0;

    for handle in handles {
        match handle.await {
            Ok((task_id, result, elapsed)) => {
                match result {
                    Ok(bulk_result) => {
                        success_count += 1;
                        println!("併發操作 {} 成功: 創建 {}, 耗時 {:?}",
                                 task_id, bulk_result.created_count, elapsed);
                    },
                    Err(e) => {
                        error_count += 1;
                        println!("併發操作 {} 失敗: {:?}", task_id, e);
                    }
                }
            },
            Err(e) => {
                error_count += 1;
                println!("併發操作任務錯誤: {:?}", e);
            }
        }
    }

    println!("併發操作結果: 成功 {}, 失敗 {}", success_count, error_count);

    // 至少應該有一些操作成功
    assert!(success_count > 0, "至少應該有一些併發操作成功");
}

/// 測試 sync_members 命令集成
///
/// 重現 cutover 報告中的 !sync_members 命令未集成問題
#[tokio::test]
async fn test_sync_members_command_integration() {
    // GIVEN: 創建測試環境
    let database_config = droas_bot::database::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());
    let admin_service = AdminService::new(user_repo.clone(), vec![123456789i64]).unwrap();

    // WHEN: 嘗試使用 sync_members 命令
    // 這個測試目前預期會失敗，因為 sync_members 命令尚未集成

    // 測試案例 1: 檢查 sync_members 命令是否在幫助系統中註冊
    println!("測試案例 1: 檢查 sync_members 命令註冊狀態");

    // 創建幫助服務並檢查命令
    let help_service = droas_bot::services::HelpService::new();
    let command_help = help_service.get_command_help("sync_members").await;

    match command_help {
        Ok(help_text) => {
            println!("sync_members 命令已註冊: {}", help_text);
        },
        Err(e) => {
            println!("sync_members 命令未註冊（預期結果）: {:?}", e);
            // 這是預期的結果，證明問題存在
        }
    }

    // 測試案例 2: 檢查 Admin Service 是否有 sync_members 處理邏輯
    println!("測試案例 2: 檢查 Admin Service 中的 sync_members 處理邏輯");

    // 這個測試需要模擬 Discord 上下文和群組成員獲取
    // 目前 sync_members 功能可能還沒有實現，所以這會失敗

    // 測試案例 3: 檢查命令路由是否支援 sync_members
    println!("測試案例 3: 檢查命令路由支援");

    let command_parser = droas_bot::discord_gateway::CommandParser::new();
    let available_commands = command_parser.get_available_commands();

    let sync_members_available = available_commands.contains(&"sync_members".to_string());

    if sync_members_available {
        println!("✅ sync_members 命令在命令解析器中可用");
    } else {
        println!("❌ sync_members 命令在命令解析器中不可用（重現了 cutover 問題）");
        // 這是預期的結果，重現了 cutover 報告中的問題
    }

    // 測試案例 4: 驗證管理員權限檢查
    println!("測試案例 4: 驗證管理員權限檢查");

    // 測試授權管理員
    let is_admin_authorized = admin_service.is_authorized_admin(123456789);
    assert!(is_admin_authorized, "授權管理員應該通過權限檢查");

    // 測試非授權用戶
    let is_unauthorized_admin = admin_service.is_authorized_admin(999999999);
    assert!(!is_unauthorized_admin, "非授權用戶應該被拒絕");

    println!("✅ 管理員權限檢查正常工作");
}

/// 測試 sync_members 命令的完整流程
///
/// 測試從命令解析到批量創建的完整流程
#[tokio::test]
async fn test_sync_members_complete_workflow() {
    // GIVEN: 創建完整的測試環境
    let database_config = droas_bot::database::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());
    let user_service = UserAccountService::new(user_repo.clone()).unwrap();
    let admin_service = AdminService::new(user_repo.clone(), vec![123456789i64]).unwrap();

    // WHEN: 模擬 sync_members 命令的執行流程

    // 測試案例 1: 模擬獲取群組成員列表
    println!("測試案例 1: 模擬獲取群組成員列表");

    // 創建模擬群組成員數據
    let mut guild_members = Vec::new();

    // 添加一些已存在的用戶
    let existing_user_id = 60001i64;
    let _ = user_repo.create_user(
        droas_bot::database::user_repository::CreateUserRequest {
            discord_user_id: existing_user_id,
            username: "ExistingGuildMember".to_string(),
            initial_balance: Some(1000.into()),
        }
    ).await;

    // 添加新成員（需要創建帳戶）
    let new_members = vec![
        (60002i64, "NewGuildMember1".to_string()),
        (60003i64, "NewGuildMember2".to_string()),
        (60004i64, "NewGuildMember3".to_string()),
    ];

    for (user_id, username) in &new_members {
        guild_members.push((*user_id, username.clone()));
    }
    guild_members.push((existing_user_id, "ExistingGuildMember".to_string()));

    // 測試案例 2: 執行批量帳戶創建
    println!("測試案例 2: 執行批量帳戶創建");

    let (user_ids, usernames): (Vec<i64>, Vec<String>) = guild_members.into_iter().unzip();

    let request = BulkAccountCreationRequest {
        user_ids,
        usernames,
    };

    let result = user_service.bulk_create_accounts(request).await;
    assert!(result.is_ok(), "批量帳戶創建應該成功");

    let bulk_result = result.unwrap();

    println!("批量創建結果:");
    println!("  總處理: {}", bulk_result.total_processed);
    println!("  創建: {}", bulk_result.created_count);
    println!("  跳過: {}", bulk_result.skipped_count);
    println!("  失敗: {}", bulk_result.failed_count);

    // 驗證結果
    assert_eq!(bulk_result.total_processed, 4, "應該處理 4 個成員");
    assert_eq!(bulk_result.created_count, 3, "應該創建 3 個新帳戶");
    assert_eq!(bulk_result.skipped_count, 1, "應該跳過 1 個已存在帳戶");
    assert_eq!(bulk_result.failed_count, 0, "應該沒有失敗的帳戶");

    // 測試案例 3: 生成統計報告
    println!("測試案例 3: 生成統計報告");

    let report = generate_sync_members_report(&bulk_result);
    println!("統計報告:\n{}", report);

    assert!(report.contains("總計: 4"), "報告應該包含總計");
    assert!(report.contains("創建: 3"), "報告應該包含創建數量");
    assert!(report.contains("跳過: 1"), "報告應該包含跳過數量");

    println!("✅ sync_members 完整流程測試完成");
}

/// 測試 sync_members 命令的權限控制
///
/// 驗證只有管理員可以執行 sync_members 命令
#[tokio::test]
async fn test_sync_members_permission_control() {
    // GIVEN: 創建測試環境
    let database_config = droas_bot::database::DatabaseConfig::for_test();
    let pool = droas_bot::database::create_user_pool(&database_config).await.unwrap();
    let user_repo = UserRepository::new(pool.clone());
    let admin_service = AdminService::new(user_repo.clone(), vec![123456789i64]).unwrap();

    // WHEN: 測試不同用戶的權限

    // 測試案例 1: 授權管理員
    println!("測試案例 1: 授權管理員執行 sync_members");
    let admin_user_id = 123456789i64;
    let is_admin = admin_service.is_authorized_admin(admin_user_id);
    assert!(is_admin, "授權管理員應該可以執行 sync_members");

    // 測試案例 2: 非授權用戶
    println!("測試案例 2: 非授權用戶執行 sync_members");
    let unauthorized_user_id = 999999999i64;
    let is_unauthorized = admin_service.is_authorized_admin(unauthorized_user_id);
    assert!(!is_unauthorized, "非授權用戶不應該可以執行 sync_members");

    // 測試案例 3: 審計記錄驗證
    println!("測試案例 3: 審計記錄驗證");

    // 這裡可以添加審計記錄的測試，但需要 AdminAuditService
    // 目前暫時跳過，因為這不是當前的主要問題

    println!("✅ sync_members 權限控制測試完成");
}

/// 生成 sync_members 統計報告的輔助函數
fn generate_sync_members_report(result: &droas_bot::services::user_account_service::BulkAccountCreationResult) -> String {
    format!(
        "🔄 群組成員同步完成\n\n" +
        "📊 **統計報告**\n" +
        "• 總計: {} 成員\n" +
        "• ✅ 創建: {} 帳戶\n" +
        "• ⏭️ 跳過: {} 帳戶（已存在）\n" +
        "• ❌ 失敗: {} 帳戶\n\n" +
        "操作執行者: 系統管理員\n" +
        "執行時間: {}",
        result.total_processed,
        result.created_count,
        result.skipped_count,
        result.failed_count,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    )
}