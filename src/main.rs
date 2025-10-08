use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, warn};

// 導入本地模組
use droas_bot::{
    config::Config,
    discord_gateway::DiscordGateway,
    database::{BalanceRepository, TransactionRepository, UserRepository, run_migrations},
    services::{
        MonitoringService, UserAccountService, BalanceService, MessageService,
        TransferService, TransactionService, HelpService, AdminService, AdminAuditService,
        create_health_routes
    },
    cache::BalanceCache,
    logging,
};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 初始化日誌系統
    logging::init_logging();
    info!("🚀 DROAS Discord Economy Bot 啟動中...");
    info!("版本: {}", env!("CARGO_PKG_VERSION"));

    // 2. 載入配置
    let config = match Config::from_env() {
        Ok(cfg) => {
            info!("✅ 配置載入成功");
            info!("Discord Token 前綴: {}...", &cfg.discord_token[..std::cmp::min(10, cfg.discord_token.len())]);
            cfg
        }
        Err(e) => {
            error!("❌ 配置載入失敗: {}", e);
            error!("請確認以下環境變數已設置:");
            error!("  - DISCORD_TOKEN: Discord 機器人令牌");
            error!("  - DATABASE_URL: PostgreSQL 資料庫連接字符串");
            error!("  - REDIS_URL: Redis 連接字符串 (可選)");
            std::process::exit(1);
        }
    };

    // 3. 初始化資料庫連接池
    let database_pool = match init_database(&config.database).await {
        Ok(pool) => {
            info!("✅ 資料庫連接成功");
            info!("資料庫 URL: {}", &config.database.url);
            pool
        }
        Err(e) => {
            error!("❌ 資料庫連接失敗: {}", e);
            error!("請確認以下事項:");
            error!("  - PostgreSQL 服務正在運行 (嘗試: brew services start postgresql)");
            error!("  - 資料庫 'droas_bot' 已創建");
            error!("  - 連接配置正確: {}", &config.database.url);
            error!("  - 用戶有適當的權限");
            std::process::exit(1);
        }
    };

    // 4. 初始化快取服務
    let _balance_cache = match init_cache(&config.cache).await {
        Ok(cache) => {
            info!("✅ 快取服務初始化成功");
            Some(cache)
        }
        Err(e) => {
            warn!("⚠️ 快取服務初始化失敗，將使用記憶體快取: {}", e);
            None
        }
    };

    // 5. 創建倉儲實例（注意：這些實例在 create_services 中重新創建）
    let _user_repository = Arc::new(UserRepository::new(database_pool.clone()));
    let _balance_repository = Arc::new(BalanceRepository::new(database_pool.clone()));
    let _transaction_repository = Arc::new(TransactionRepository::new(database_pool.clone()));

    // 6. 創建服務實例
    let services = create_services(
        database_pool.clone(),
        _balance_cache.clone(),
        &config.admin
    ).await;

    // 7. 創建監控服務
    let monitoring_service = create_monitoring_service(database_pool.clone()).await;

    // 8. 啟動監控服務器
    let monitoring_config = droas_bot::services::MonitoringConfig::from_env_with_port_check();
    start_monitoring_servers(monitoring_service.clone(), monitoring_config).await?;

    // 9. 創建並配置 Discord Gateway
    let mut discord_gateway = create_discord_gateway(config.clone(), services).await;

    // 10. 連接到 Discord（創建客戶端）
    match discord_gateway.connect().await {
        Ok(_) => {
            info!("✅ Discord Gateway 客戶端創建成功");
            info!("正在驗證 Discord Token...");
        }
        Err(e) => {
            error!("❌ Discord Gateway 客戶端創建失敗: {}", e);
            error!("請確認以下事項:");
            error!("  - Discord Token 正確且未過期");
            error!("  - 從 Discord Developer Portal 獲取有效的 Bot Token");
            error!("  - 機器人有以下 Gateway Intents: GUILD_MESSAGES, MESSAGE_CONTENT");
            error!("  - 機器人已被添加到伺服器");
            std::process::exit(1);
        }
    }

    // 11. 啟動 Discord 客戶端
    info!("正在啟動 Discord 客戶端...");
    if let Err(e) = discord_gateway.start().await {
        error!("❌ Discord 客戶端啟動失敗: {}", e);
        error!("可能的解決方案:");
        error!("  - 檢查網絡連接");
        error!("  - 確認 Discord 服務正常運行");
        error!("  - 驗證機器人權限設置");
        error!("  - 檢查 Token 是否被撤銷");
        std::process::exit(1);
    }

    // 12. 運行機器人
    info!("🚀 DROAS Bot 已成功啟動並準備接收命令");
    info!("支援的命令: !balance, !transfer, !history, !help");
    info!("按 Ctrl+C 優雅關閉機器人");

    // 設置優雅關閉處理
    tokio::signal::ctrl_c().await?;
    info!("收到關閉信號，正在優雅關閉...");

    // 關閉 Discord 客戶端
    if let Err(e) = discord_gateway.shutdown().await {
        error!("關閉 Discord 客戶端時發生錯誤: {}", e);
    }

    info!("DROAS Bot 已安全關閉");

    Ok(())
}

/// 初始化資料庫連接池
async fn init_database(config: &droas_bot::config::DatabaseConfig) -> Result<sqlx::PgPool> {
    info!("正在連接到資料庫: {}", config.url);

    let pool = sqlx::PgPool::connect(&config.url).await?;

    // 執行資料庫遷移
    info!("正在執行資料庫遷移...");
    run_migrations(&pool).await?;
    info!("✅ 資料庫遷移完成");

    Ok(pool)
}

/// 初始化快取服務
async fn init_cache(config: &droas_bot::config::CacheConfig) -> Result<Arc<BalanceCache>> {
    if config.enable_redis {
        info!("正在連接到 Redis: {}", config.redis_url);

        // 嘗試建立 Redis 連接
        match droas_bot::cache::RedisCache::new_with_ttl(&config.redis_url, config.default_ttl).await {
            Ok(redis_cache) => {
                info!("✅ Redis 連接成功建立");

                // 測試 Redis 連接
                match redis_cache.ping().await {
                    Ok(true) => {
                        info!("✅ Redis 連接健康檢查通過");
                        let balance_cache = BalanceCache::from_redis_with_ttl(redis_cache, config.default_ttl).await;
                        Ok(Arc::new(balance_cache))
                    }
                    Ok(false) => {
                        warn!("⚠️ Redis 連接健康檢查失敗");
                        if config.fallback_to_memory {
                            warn!("📦 降級到記憶體快取");
                            Ok(Arc::new(BalanceCache::new_with_ttl(config.default_ttl)))
                        } else {
                            error!("❌ Redis 連接失敗且不允許降級到記憶體快取");
                            return Err(anyhow::anyhow!("Redis 連接失敗且不允許降級"));
                        }
                    }
                    Err(e) => {
                        warn!("⚠️ Redis 連接錯誤: {}", e);
                        if config.fallback_to_memory {
                            warn!("📦 降級到記憶體快取");
                            Ok(Arc::new(BalanceCache::new_with_ttl(config.default_ttl)))
                        } else {
                            error!("❌ Redis 連接失敗且不允許降級到記憶體快取");
                            return Err(anyhow::anyhow!("Redis 連接失敗: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ 無法建立 Redis 連接: {}", e);
                if config.fallback_to_memory {
                    warn!("📦 降級到記憶體快取");
                    Ok(Arc::new(BalanceCache::new_with_ttl(config.default_ttl)))
                } else {
                    error!("❌ Redis 連接失敗且不允許降級到記憶體快取");
                    return Err(anyhow::anyhow!("Redis 連接失敗: {}", e));
                }
            }
        }
    } else {
        info!("使用記憶體快取");
        Ok(Arc::new(BalanceCache::new_with_ttl(config.default_ttl)))
    }
}

/// 創建監控服務
async fn create_monitoring_service(
    database_pool: sqlx::PgPool,
) -> Arc<MonitoringService> {
    Arc::new(
        droas_bot::create_monitoring_service(database_pool, None, None).await
    )
}

/// 啟動監控服務器
async fn start_monitoring_servers(
    _monitoring_service: Arc<MonitoringService>,
    config: droas_bot::services::MonitoringConfig,
) -> Result<()> {
    let health_routes = create_health_routes(_monitoring_service.clone());
    let health_port = config.server_port;

    tokio::spawn(async move {
        warp::serve(health_routes)
            .run(([127, 0, 0, 1], health_port))
            .await;
    });

    info!("🔍 監控服務器已啟動於端口: {}", health_port);

    Ok(())
}

/// 創建所有服務實例
async fn create_services(
    database_pool: sqlx::PgPool,
    balance_cache: Option<Arc<BalanceCache>>,
    admin_config: &droas_bot::config::AdminConfig,
) -> Services {
    info!("正在創建服務實例...");

    // 創建用戶帳戶服務
    let user_account_service = Arc::new(
        UserAccountService::new(UserRepository::new(database_pool.clone()))
            .expect("無法創建用戶帳戶服務")
    );

    // 創建餘額服務
    let balance_service = Arc::new(
        if let Some(_cache) = balance_cache {
            // 由於 BalanceCache 沒有實現 Clone，這裡我們創建新的服務但不使用快取
            // 或者我們可以重新設計這部分
            BalanceService::new(BalanceRepository::new(database_pool.clone()))
        } else {
            BalanceService::new(BalanceRepository::new(database_pool.clone()))
        }
    );

    // 創建安全服務
    let security_service = droas_bot::services::SecurityService::new(UserRepository::new(database_pool.clone()))
        .expect("無法創建安全服務");

    // 創建轉帳服務
    let transfer_service = Arc::new(
        TransferService::new(
            UserRepository::new(database_pool.clone()),
            TransactionRepository::new(database_pool.clone()),
            security_service,
        ).expect("無法創建轉帳服務")
    );

    // 創建交易服務
    let transaction_service = Arc::new(
        TransactionService::new(
            TransactionRepository::new(database_pool.clone()),
            UserRepository::new(database_pool.clone())
        )
    );

    // 創建消息服務
    let message_service = Arc::new(MessageService::new());

    // 創建幫助服務
    let help_service = Arc::new(HelpService::new());

    // 創建管理員服務
    let admin_service = Arc::new(
        AdminService::new_with_repositories(
            UserRepository::new(database_pool.clone()),
            Arc::new(BalanceRepository::new(database_pool.clone())),
            Arc::new(TransactionRepository::new(database_pool.clone())),
            admin_config.authorized_admins.clone(),
        ).expect("無法創建管理員服務")
    );

    // 創建管理員審計服務
    let admin_audit_service = Arc::new(
        AdminAuditService::new(TransactionRepository::new(database_pool.clone()))
            .expect("無法創建管理員審計服務")
    );

    info!("✅ 所有服務實例創建完成");

    Services {
        user_account_service,
        balance_service,
        transfer_service,
        transaction_service,
        message_service,
        help_service,
        admin_service,
        admin_audit_service,
    }
}

/// 創建 Discord Gateway
async fn create_discord_gateway(
    config: Config,
    services: Services,
) -> DiscordGateway {
    let mut discord_gateway = DiscordGateway::new_with_config(config);

    // 注入所有服務到 Discord Gateway
    discord_gateway.configure_command_router(|router| {
        router
            .with_user_account_service(services.user_account_service)
            .with_balance_service(services.balance_service)
            .with_transfer_service(services.transfer_service)
            .with_transaction_service(services.transaction_service)
            .with_message_service(services.message_service)
            .with_help_service(services.help_service)
            .with_admin_service(services.admin_service)
            .with_admin_audit_service(services.admin_audit_service)
    });

    discord_gateway
}

/// 服務容器結構
struct Services {
    user_account_service: Arc<UserAccountService>,
    balance_service: Arc<BalanceService>,
    transfer_service: Arc<TransferService>,
    transaction_service: Arc<TransactionService>,
    message_service: Arc<MessageService>,
    help_service: Arc<HelpService>,
    admin_service: Arc<AdminService>,
    admin_audit_service: Arc<AdminAuditService>,
}