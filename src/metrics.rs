use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CommandMetrics {
    pub total_commands: u64,
    pub successful_commands: u64,
    pub failed_commands: u64,
    pub average_response_time_ms: f64,
    pub last_command_time: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct ConnectionMetrics {
    pub total_connections: u64,
    pub successful_connections: u64,
    pub failed_connections: u64,
    pub current_connection_uptime: Option<Instant>,
    pub total_uptime_ms: u64,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub start_time: Instant,
    pub uptime_ms: u64,
}

#[derive(Debug, Clone)]
pub struct DatabaseMetrics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub average_query_time_ms: f64,
    pub connection_pool_active: u32,
    pub connection_pool_idle: u32,
}

#[derive(Debug, Clone)]
pub struct AccountMetrics {
    pub total_account_creations: u64,
    pub successful_account_creations: u64,
    pub failed_account_creations: u64,
    pub account_exists_checks: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub average_creation_time_ms: f64,
}

#[derive(Debug, Clone)]
pub struct TransferMetrics {
    pub total_transfers: u64,
    pub successful_transfers: u64,
    pub failed_transfers: u64,
    pub total_amount_transferred: f64,
    pub average_transfer_amount: f64,
    pub average_transfer_time_ms: f64,
    pub insufficient_balance_errors: u64,
    pub invalid_recipient_errors: u64,
    pub invalid_amount_errors: u64,
    pub self_transfer_attempts: u64,
}

#[derive(Debug, Clone)]
pub struct TransactionMetrics {
    pub total_transactions_recorded: u64,
    pub successful_transaction_queries: u64,
    pub failed_transaction_queries: u64,
    pub transaction_history_queries: u64,
    pub average_query_time_ms: f64,
    pub empty_history_results: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

#[derive(Debug)]
pub struct MetricsCollector {
    commands: Arc<RwLock<HashMap<String, CommandMetrics>>>,
    connection: Arc<RwLock<ConnectionMetrics>>,
    system: Arc<RwLock<SystemMetrics>>,
    database: Arc<RwLock<DatabaseMetrics>>,
    accounts: Arc<RwLock<AccountMetrics>>,
    transfers: Arc<RwLock<TransferMetrics>>,
    transactions: Arc<RwLock<TransactionMetrics>>,
}

impl MetricsCollector {
    /// 創建新的指標收集器
    ///
    /// 初始化所有指標收集器，包括命令、連接、系統、資料庫和帳戶指標。
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
            connection: Arc::new(RwLock::new(ConnectionMetrics {
                total_connections: 0,
                successful_connections: 0,
                failed_connections: 0,
                current_connection_uptime: None,
                total_uptime_ms: 0,
            })),
            system: Arc::new(RwLock::new(SystemMetrics {
                start_time: Instant::now(),
                uptime_ms: 0,
            })),
            database: Arc::new(RwLock::new(DatabaseMetrics {
                total_queries: 0,
                successful_queries: 0,
                failed_queries: 0,
                average_query_time_ms: 0.0,
                connection_pool_active: 0,
                connection_pool_idle: 0,
            })),
            accounts: Arc::new(RwLock::new(AccountMetrics {
                total_account_creations: 0,
                successful_account_creations: 0,
                failed_account_creations: 0,
                account_exists_checks: 0,
                cache_hits: 0,
                cache_misses: 0,
                average_creation_time_ms: 0.0,
            })),
            transfers: Arc::new(RwLock::new(TransferMetrics {
                total_transfers: 0,
                successful_transfers: 0,
                failed_transfers: 0,
                total_amount_transferred: 0.0,
                average_transfer_amount: 0.0,
                average_transfer_time_ms: 0.0,
                insufficient_balance_errors: 0,
                invalid_recipient_errors: 0,
                invalid_amount_errors: 0,
                self_transfer_attempts: 0,
            })),
            transactions: Arc::new(RwLock::new(TransactionMetrics {
                total_transactions_recorded: 0,
                successful_transaction_queries: 0,
                failed_transaction_queries: 0,
                transaction_history_queries: 0,
                average_query_time_ms: 0.0,
                empty_history_results: 0,
                cache_hits: 0,
                cache_misses: 0,
            })),
        }
    }

    pub async fn record_command(&self, command: &str, response_time_ms: u64, success: bool) {
        let mut commands = self.commands.write().await;
        let metrics = commands.entry(command.to_string()).or_insert_with(|| CommandMetrics {
            total_commands: 0,
            successful_commands: 0,
            failed_commands: 0,
            average_response_time_ms: 0.0,
            last_command_time: None,
        });

        metrics.total_commands += 1;
        if success {
            metrics.successful_commands += 1;
        } else {
            metrics.failed_commands += 1;
        }

        // 更新平均響應時間
        let total_commands = metrics.total_commands as f64;
        let current_avg = metrics.average_response_time_ms;
        metrics.average_response_time_ms = (current_avg * (total_commands - 1.0) + response_time_ms as f64) / total_commands;
        metrics.last_command_time = Some(Instant::now());
    }

    pub async fn record_connection_attempt(&self, success: bool) {
        let mut connection = self.connection.write().await;
        connection.total_connections += 1;

        if success {
            connection.successful_connections += 1;
            connection.current_connection_uptime = Some(Instant::now());
        } else {
            connection.failed_connections += 1;
        }
    }

    pub async fn get_command_metrics(&self, command: &str) -> Option<CommandMetrics> {
        let commands = self.commands.read().await;
        commands.get(command).cloned()
    }

    pub async fn get_connection_metrics(&self) -> ConnectionMetrics {
        let connection = self.connection.read().await;
        connection.clone()
    }

    pub async fn update_system_metrics(&self) {
        let mut system = self.system.write().await;
        system.uptime_ms = system.start_time.elapsed().as_millis() as u64;
    }

    pub async fn get_system_metrics(&self) -> SystemMetrics {
        self.update_system_metrics().await;
        let system = self.system.read().await;
        system.clone()
    }

    /// 記錄資料庫查詢指標
    ///
    /// # Arguments
    ///
    /// * `query_time_ms` - 查詢執行時間（毫秒）
    /// * `success` - 查詢是否成功
    pub async fn record_database_query(&self, query_time_ms: u64, success: bool) {
        let mut database = self.database.write().await;
        database.total_queries += 1;

        if success {
            database.successful_queries += 1;
        } else {
            database.failed_queries += 1;
        }

        // 更新平均查詢時間
        let total_queries = database.total_queries as f64;
        let current_avg = database.average_query_time_ms;
        database.average_query_time_ms = (current_avg * (total_queries - 1.0) + query_time_ms as f64) / total_queries;
    }

    /// 更新連接池指標
    ///
    /// # Arguments
    ///
    /// * `active` - 活躍連接數
    /// * `idle` - 空閒連接數
    pub async fn update_connection_pool_metrics(&self, active: u32, idle: u32) {
        let mut database = self.database.write().await;
        database.connection_pool_active = active;
        database.connection_pool_idle = idle;
    }

    /// 獲取資料庫指標
    ///
    /// # Returns
    ///
    /// 返回當前資料庫指標
    pub async fn get_database_metrics(&self) -> DatabaseMetrics {
        let database = self.database.read().await;
        database.clone()
    }

    /// 記錄帳戶創建指標
    ///
    /// # Arguments
    ///
    /// * `creation_time_ms` - 帳戶創建時間（毫秒）
    /// * `success` - 帳戶創建是否成功
    pub async fn record_account_creation(&self, creation_time_ms: u64, success: bool) {
        let mut accounts = self.accounts.write().await;
        accounts.total_account_creations += 1;

        if success {
            accounts.successful_account_creations += 1;
        } else {
            accounts.failed_account_creations += 1;
        }

        // 更新平均創建時間
        let total_creations = accounts.total_account_creations as f64;
        let current_avg = accounts.average_creation_time_ms;
        accounts.average_creation_time_ms = (current_avg * (total_creations - 1.0) + creation_time_ms as f64) / total_creations;
    }

    /// 記錄帳戶存在檢查
    ///
    /// # Arguments
    ///
    /// * `cache_hit` - 是否為快取命中
    pub async fn record_account_exists_check(&self, cache_hit: bool) {
        let mut accounts = self.accounts.write().await;
        accounts.account_exists_checks += 1;

        if cache_hit {
            accounts.cache_hits += 1;
        } else {
            accounts.cache_misses += 1;
        }
    }

    /// 獲取帳戶指標
    ///
    /// # Returns
    ///
    /// 返回當前帳戶指標
    pub async fn get_account_metrics(&self) -> AccountMetrics {
        let accounts = self.accounts.read().await;
        accounts.clone()
    }

    /// 記錄轉帳指標
    ///
    /// # Arguments
    ///
    /// * `amount` - 轉帳金額
    /// * `transfer_time_ms` - 轉帳處理時間（毫秒）
    /// * `success` - 轉帳是否成功
    /// * `error_type` - 錯誤類型（如果失敗）
    pub async fn record_transfer(&self, amount: f64, transfer_time_ms: u64, success: bool, error_type: Option<&str>) {
        let mut transfers = self.transfers.write().await;
        transfers.total_transfers += 1;

        if success {
            transfers.successful_transfers += 1;
            transfers.total_amount_transferred += amount;

            // 更新平均轉帳金額
            let successful_transfers = transfers.successful_transfers as f64;
            let current_avg = transfers.average_transfer_amount;
            transfers.average_transfer_amount = (current_avg * (successful_transfers - 1.0) + amount) / successful_transfers;
        } else {
            transfers.failed_transfers += 1;

            // 記錄錯誤類型
            if let Some(error) = error_type {
                match error {
                    "insufficient_balance" => transfers.insufficient_balance_errors += 1,
                    "invalid_recipient" => transfers.invalid_recipient_errors += 1,
                    "invalid_amount" => transfers.invalid_amount_errors += 1,
                    "self_transfer" => transfers.self_transfer_attempts += 1,
                    _ => {} // 其他錯誤類型
                }
            }
        }

        // 更新平均轉帳時間
        let total_transfers = transfers.total_transfers as f64;
        let current_avg = transfers.average_transfer_time_ms;
        transfers.average_transfer_time_ms = (current_avg * (total_transfers - 1.0) + transfer_time_ms as f64) / total_transfers;
    }

    /// 獲取轉帳指標
    ///
    /// # Returns
    ///
    /// 返回當前轉帳指標
    pub async fn get_transfer_metrics(&self) -> TransferMetrics {
        let transfers = self.transfers.read().await;
        transfers.clone()
    }

    /// 生成 Prometheus 格式的指標輸出
    ///
    /// # Returns
    ///
    /// 返回 Prometheus 格式的指標字符串
    pub async fn generate_prometheus_metrics(&self) -> String {
        let commands = self.commands.read().await;
        let connection = self.connection.read().await;
        let system = self.system.read().await;
        let database = self.database.read().await;
        let accounts = self.accounts.read().await;
        let transfers = self.transfers.read().await;

        let mut output = String::new();

        // 系統指標
        output.push_str("# HELP droas_system_uptime_seconds System uptime in seconds\n");
        output.push_str("# TYPE droas_system_uptime_seconds counter\n");
        output.push_str(&format!("droas_system_uptime_seconds {}\n\n", system.uptime_ms as f64 / 1000.0));

        // Discord 連接指標
        output.push_str("# HELP droas_discord_connections_total Total Discord connection attempts\n");
        output.push_str("# TYPE droas_discord_connections_total counter\n");
        output.push_str(&format!("droas_discord_connections_total {}\n", connection.total_connections));

        output.push_str("# HELP droas_discord_connections_successful Total successful Discord connections\n");
        output.push_str("# TYPE droas_discord_connections_successful counter\n");
        output.push_str(&format!("droas_discord_connections_successful {}\n", connection.successful_connections));

        output.push_str("# HELP droas_discord_connections_failed Total failed Discord connections\n");
        output.push_str("# TYPE droas_discord_connections_failed counter\n");
        output.push_str(&format!("droas_discord_connections_failed {}\n\n", connection.failed_connections));

        // 資料庫指標
        output.push_str("# HELP droas_database_queries_total Total database queries\n");
        output.push_str("# TYPE droas_database_queries_total counter\n");
        output.push_str(&format!("droas_database_queries_total {}\n", database.total_queries));

        output.push_str("# HELP droas_database_queries_successful Total successful database queries\n");
        output.push_str("# TYPE droas_database_queries_successful counter\n");
        output.push_str(&format!("droas_database_queries_successful {}\n", database.successful_queries));

        output.push_str("# HELP droas_database_queries_failed Total failed database queries\n");
        output.push_str("# TYPE droas_database_queries_failed counter\n");
        output.push_str(&format!("droas_database_queries_failed {}\n", database.failed_queries));

        output.push_str("# HELP droas_database_query_duration_seconds Average database query duration\n");
        output.push_str("# TYPE droas_database_query_duration_seconds gauge\n");
        output.push_str(&format!("droas_database_query_duration_seconds {}\n", database.average_query_time_ms / 1000.0));

        output.push_str("# HELP droas_database_connection_pool_active Active database connections\n");
        output.push_str("# TYPE droas_database_connection_pool_active gauge\n");
        output.push_str(&format!("droas_database_connection_pool_active {}\n", database.connection_pool_active));

        output.push_str("# HELP droas_database_connection_pool_idle Idle database connections\n");
        output.push_str("# TYPE droas_database_connection_pool_idle gauge\n");
        output.push_str(&format!("droas_database_connection_pool_idle {}\n\n", database.connection_pool_idle));

        // 帳戶指標
        output.push_str("# HELP droas_accounts_creation_total Total account creation attempts\n");
        output.push_str("# TYPE droas_accounts_creation_total counter\n");
        output.push_str(&format!("droas_accounts_creation_total {}\n", accounts.total_account_creations));

        output.push_str("# HELP droas_accounts_creation_successful_total Total successful account creations\n");
        output.push_str("# TYPE droas_accounts_creation_successful_total counter\n");
        output.push_str(&format!("droas_accounts_creation_successful_total {}\n", accounts.successful_account_creations));

        output.push_str("# HELP droas_accounts_creation_failed_total Total failed account creations\n");
        output.push_str("# TYPE droas_accounts_creation_failed_total counter\n");
        output.push_str(&format!("droas_accounts_creation_failed_total {}\n", accounts.failed_account_creations));

        output.push_str("# HELP droas_accounts_creation_duration_seconds Average account creation duration\n");
        output.push_str("# TYPE droas_accounts_creation_duration_seconds gauge\n");
        output.push_str(&format!("droas_accounts_creation_duration_seconds {}\n", accounts.average_creation_time_ms / 1000.0));

        output.push_str("# HELP droas_accounts_exists_checks_total Total account existence checks\n");
        output.push_str("# TYPE droas_accounts_exists_checks_total counter\n");
        output.push_str(&format!("droas_accounts_exists_checks_total {}\n", accounts.account_exists_checks));

        output.push_str("# HELP droas_accounts_cache_hits_total Total cache hits\n");
        output.push_str("# TYPE droas_accounts_cache_hits_total counter\n");
        output.push_str(&format!("droas_accounts_cache_hits_total {}\n", accounts.cache_hits));

        output.push_str("# HELP droas_accounts_cache_misses_total Total cache misses\n");
        output.push_str("# TYPE droas_accounts_cache_misses_total counter\n");
        output.push_str(&format!("droas_accounts_cache_misses_total {}\n\n", accounts.cache_misses));

        // 轉帳指標 (REFACTOR 階段新增)
        output.push_str("# HELP droas_transfers_total Total transfer attempts\n");
        output.push_str("# TYPE droas_transfers_total counter\n");
        output.push_str(&format!("droas_transfers_total {}\n", transfers.total_transfers));

        output.push_str("# HELP droas_transfers_successful_total Total successful transfers\n");
        output.push_str("# TYPE droas_transfers_successful_total counter\n");
        output.push_str(&format!("droas_transfers_successful_total {}\n", transfers.successful_transfers));

        output.push_str("# HELP droas_transfers_failed_total Total failed transfers\n");
        output.push_str("# TYPE droas_transfers_failed_total counter\n");
        output.push_str(&format!("droas_transfers_failed_total {}\n", transfers.failed_transfers));

        output.push_str("# HELP droas_transfers_amount_total Total amount transferred\n");
        output.push_str("# TYPE droas_transfers_amount_total counter\n");
        output.push_str(&format!("droas_transfers_amount_total {}\n", transfers.total_amount_transferred));

        output.push_str("# HELP droas_transfers_average_amount Average transfer amount\n");
        output.push_str("# TYPE droas_transfers_average_amount gauge\n");
        output.push_str(&format!("droas_transfers_average_amount {}\n", transfers.average_transfer_amount));

        output.push_str("# HELP droas_transfers_duration_seconds Average transfer duration\n");
        output.push_str("# TYPE droas_transfers_duration_seconds gauge\n");
        output.push_str(&format!("droas_transfers_duration_seconds {}\n", transfers.average_transfer_time_ms / 1000.0));

        output.push_str("# HELP droas_transfers_insufficient_balance_errors Total insufficient balance errors\n");
        output.push_str("# TYPE droas_transfers_insufficient_balance_errors counter\n");
        output.push_str(&format!("droas_transfers_insufficient_balance_errors {}\n", transfers.insufficient_balance_errors));

        output.push_str("# HELP droas_transfers_invalid_recipient_errors Total invalid recipient errors\n");
        output.push_str("# TYPE droas_transfers_invalid_recipient_errors counter\n");
        output.push_str(&format!("droas_transfers_invalid_recipient_errors {}\n", transfers.invalid_recipient_errors));

        output.push_str("# HELP droas_transfers_invalid_amount_errors Total invalid amount errors\n");
        output.push_str("# TYPE droas_transfers_invalid_amount_errors counter\n");
        output.push_str(&format!("droas_transfers_invalid_amount_errors {}\n", transfers.invalid_amount_errors));

        output.push_str("# HELP droas_transfers_self_transfer_attempts Total self transfer attempts\n");
        output.push_str("# TYPE droas_transfers_self_transfer_attempts counter\n");
        output.push_str(&format!("droas_transfers_self_transfer_attempts {}\n\n", transfers.self_transfer_attempts));

        // 命令指標
        output.push_str("# HELP droas_commands_total Total commands processed\n");
        output.push_str("# TYPE droas_commands_total counter\n");

        for (command_name, metrics) in commands.iter() {
            output.push_str(&format!(
                "droas_commands_total{{command=\"{}\"}} {}\n",
                command_name, metrics.total_commands
            ));

            output.push_str(&format!(
                "droas_commands_successful_total{{command=\"{}\"}} {}\n",
                command_name, metrics.successful_commands
            ));

            output.push_str(&format!(
                "droas_commands_failed_total{{command=\"{}\"}} {}\n",
                command_name, metrics.failed_commands
            ));

            output.push_str(&format!(
                "droas_command_duration_seconds{{command=\"{}\"}} {}\n",
                command_name, metrics.average_response_time_ms / 1000.0
            ));
        }

        output
    }

    /// 記錄交易記錄創建
    pub async fn record_transaction_created(&self) {
        let mut transactions = self.transactions.write().await;
        transactions.total_transactions_recorded += 1;
    }

    /// 記錄交易查詢
    ///
    /// # Arguments
    ///
    /// * `query_time_ms` - 查詢執行時間（毫秒）
    /// * `success` - 查詢是否成功
    /// * `is_history_query` - 是否為歷史查詢
    pub async fn record_transaction_query(&self, query_time_ms: u64, success: bool, is_history_query: bool) {
        let mut transactions = self.transactions.write().await;

        if success {
            transactions.successful_transaction_queries += 1;
        } else {
            transactions.failed_transaction_queries += 1;
        }

        if is_history_query {
            transactions.transaction_history_queries += 1;
        }

        // 更新平均查詢時間
        let total_queries = (transactions.successful_transaction_queries + transactions.failed_transaction_queries) as f64;
        if total_queries > 0.0 {
            let current_avg = transactions.average_query_time_ms;
            transactions.average_query_time_ms = (current_avg * (total_queries - 1.0) + query_time_ms as f64) / total_queries;
        }
    }

    /// 記錄空交易歷史結果
    pub async fn record_empty_transaction_history(&self) {
        let mut transactions = self.transactions.write().await;
        transactions.empty_history_results += 1;
    }

    /// 記錄交易查詢快取命中
    pub async fn record_transaction_cache_hit(&self) {
        let mut transactions = self.transactions.write().await;
        transactions.cache_hits += 1;
    }

    /// 記錄交易查詢快取未命中
    pub async fn record_transaction_cache_miss(&self) {
        let mut transactions = self.transactions.write().await;
        transactions.cache_misses += 1;
    }

    /// 獲取交易指標
    pub async fn get_transaction_metrics(&self) -> TransactionMetrics {
        let transactions = self.transactions.read().await;
        transactions.clone()
    }
}