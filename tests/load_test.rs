// Load Tests - 負載測試
// 模擬真實的 Discord 機器人負載，測試系統在 1000+ 並發用戶下的表現

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, RwLock};
use tokio::time::sleep;
use bigdecimal::BigDecimal;
use std::str::FromStr;
use droas_bot::cache::BalanceCache;

// 性能測試結果結構
#[derive(Debug, Clone)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub total_operations: u32,
    pub successful_operations: u32,
    pub failed_operations: u32,
    pub duration: Duration,
    pub avg_response_time: Duration,
    pub p50_response_time: Duration,
    pub p95_response_time: Duration,
    pub p99_response_time: Duration,
    pub operations_per_second: f64,
}

impl PerformanceTestResult {
    fn new(
        test_name: String,
        total_operations: u32,
        successful_operations: u32,
        failed_operations: u32,
        duration: Duration,
        response_times: Vec<Duration>,
    ) -> Self {
        let mut sorted_times = response_times.clone();
        sorted_times.sort();

        let avg_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<Duration>() / response_times.len() as u32
        } else {
            Duration::ZERO
        };

        let p50_response_time = if !sorted_times.is_empty() {
            sorted_times[sorted_times.len() * 50 / 100]
        } else {
            Duration::ZERO
        };

        let p95_response_time = if !sorted_times.is_empty() {
            sorted_times[sorted_times.len() * 95 / 100]
        } else {
            Duration::ZERO
        };

        let p99_response_time = if !sorted_times.is_empty() {
            sorted_times[sorted_times.len() * 99 / 100]
        } else {
            Duration::ZERO
        };

        let operations_per_second = if duration.as_secs_f64() > 0.0 {
            successful_operations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            test_name,
            total_operations,
            successful_operations,
            failed_operations,
            duration,
            avg_response_time,
            p50_response_time,
            p95_response_time,
            p99_response_time,
            operations_per_second,
        }
    }

    /// 驗證性能是否滿足 NFR 要求
    pub fn meets_nfr_requirements(&self) -> bool {
        // NFR-P-001: 95% 的命令在 2 秒內響應
        let meets_response_time = self.p95_response_time < Duration::from_secs(2);

        // NFR-P-002: 餘額查詢在 500ms 內完成（對於餘額查詢測試）
        let meets_balance_query_time = self.test_name.contains("balance")
            && self.p95_response_time < Duration::from_millis(500);

        // 成功率應該 > 99%
        let success_rate = self.successful_operations as f64 / self.total_operations as f64;
        let meets_success_rate = success_rate >= 0.99;

        match self.test_name.as_str() {
            name if name.contains("balance") => meets_response_time && meets_balance_query_time && meets_success_rate,
            _ => meets_response_time && meets_success_rate,
        }
    }

    pub fn print(&self) {
        println!("\n=== {} ===", self.test_name);
        println!("總操作數: {}", self.total_operations);
        println!("成功操作數: {}", self.successful_operations);
        println!("失敗操作數: {}", self.failed_operations);
        println!("執行時間: {:?}", self.duration);
        println!("平均響應時間: {:?}", self.avg_response_time);
        println!("P50 響應時間: {:?}", self.p50_response_time);
        println!("P95 響應時間: {:?}", self.p95_response_time);
        println!("P99 響應時間: {:?}", self.p99_response_time);
        println!("每秒操作數: {:.2}", self.operations_per_second);
        println!("NFR 要求滿足: {}", if self.meets_nfr_requirements() { "✓" } else { "✗" });
    }
}

// 模擬服務層用於負載測試
struct MockBalanceService {
    cache: Arc<BalanceCache>,
    // 模擬資料庫延遲
    db_delay: Duration,
}

impl MockBalanceService {
    fn new(cache: Arc<BalanceCache>, db_delay: Duration) -> Self {
        Self { cache, db_delay }
    }

    async fn get_balance(&self, user_id: u64) -> Option<BigDecimal> {
        // 先檢查快取
        if let Some(balance) = self.cache.get_balance(user_id).await {
            return Some(balance);
        }

        // 模擬資料庫查詢延遲
        sleep(self.db_delay).await;

        // 模擬從資料庫獲取餘額
        let balance = BigDecimal::from_str(&format!("{}.00", user_id % 10000)).unwrap();

        // 存入快取
        self.cache.set_balance(user_id, balance.clone()).await;

        Some(balance)
    }

    async fn set_balance(&self, user_id: u64, balance: BigDecimal) {
        // 模擬資料庫更新延遲
        sleep(self.db_delay).await;

        // 更新快取
        self.cache.set_balance(user_id, balance).await;
    }
}

/// Discord 命令類型模擬
#[derive(Debug, Clone)]
pub enum MockDiscordCommand {
    Balance { user_id: u64 },
    Transfer { from_user_id: u64, to_user_id: u64, amount: BigDecimal },
    History { user_id: u64 },
    Help { user_id: u64 },
}

/// Discord 事件模擬
#[derive(Debug, Clone)]
pub struct MockDiscordEvent {
    pub command: MockDiscordCommand,
    pub timestamp: Instant,
    pub user_id: u64,
}

impl MockDiscordEvent {
    fn new(command: MockDiscordCommand, user_id: u64) -> Self {
        Self {
            command,
            timestamp: Instant::now(),
            user_id,
        }
    }
}

/// 模擬 Discord 網關和命令路由器
struct MockDiscordGateway {
    balance_service: Arc<MockBalanceService>,
    event_counter: Arc<RwLock<u64>>,
    error_counter: Arc<RwLock<u64>>,
}

impl MockDiscordGateway {
    fn new(balance_service: Arc<MockBalanceService>) -> Self {
        Self {
            balance_service,
            event_counter: Arc::new(RwLock::new(0)),
            error_counter: Arc::new(RwLock::new(0)),
        }
    }

    async fn process_event(&self, event: MockDiscordEvent) -> Result<(), String> {
        // 增加事件計數
        {
            let mut counter = self.event_counter.write().await;
            *counter += 1;
        }

        // 模擬 Discord 事件處理延遲（網絡延遲、解析等）
        sleep(Duration::from_millis(5)).await;

        let result = match event.command {
            MockDiscordCommand::Balance { user_id } => {
                self.handle_balance_command(user_id).await
            }
            MockDiscordCommand::Transfer { from_user_id, to_user_id, amount } => {
                self.handle_transfer_command(from_user_id, to_user_id, amount).await
            }
            MockDiscordCommand::History { user_id } => {
                self.handle_history_command(user_id).await
            }
            MockDiscordCommand::Help { user_id: _ } => {
                self.handle_help_command().await
            }
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // 增加錯誤計數
                let mut counter = self.error_counter.write().await;
                *counter += 1;
                Err(e)
            }
        }
    }

    async fn handle_balance_command(&self, user_id: u64) -> Result<(), String> {
        // 模擬餘額查詢邏輯
        let balance = self.balance_service.get_balance(user_id).await;
        if balance.is_some() {
            Ok(())
        } else {
            Err("用戶不存在".to_string())
        }
    }

    async fn handle_transfer_command(&self, from_user_id: u64, to_user_id: u64, amount: BigDecimal) -> Result<(), String> {
        // 模擬轉帳驗證和執行
        if from_user_id == to_user_id {
            return Err("不能轉帳給自己".to_string());
        }

        let from_balance = self.balance_service.get_balance(from_user_id).await;
        let to_balance = self.balance_service.get_balance(to_user_id).await;

        if from_balance.is_none() || to_balance.is_none() {
            return Err("用戶不存在".to_string());
        }

        if let Some(ref balance) = from_balance {
            if *balance < amount {
                return Err("餘額不足".to_string());
            }
        }

        // 模擬轉帳操作
        sleep(Duration::from_millis(20)).await;

        // 更新餘額
        let new_from_balance = from_balance.unwrap() - amount.clone();
        let new_to_balance = to_balance.unwrap() + amount;

        self.balance_service.set_balance(from_user_id, new_from_balance).await;
        self.balance_service.set_balance(to_user_id, new_to_balance).await;

        Ok(())
    }

    async fn handle_history_command(&self, user_id: u64) -> Result<(), String> {
        // 模擬歷史查詢（通常需要訪問資料庫）
        sleep(Duration::from_millis(50)).await;

        // 驗證用戶存在
        let balance = self.balance_service.get_balance(user_id).await;
        if balance.is_some() {
            Ok(())
        } else {
            Err("用戶不存在".to_string())
        }
    }

    async fn handle_help_command(&self) -> Result<(), String> {
        // 幫助命令通常不需要資料庫訪問
        sleep(Duration::from_millis(1)).await;
        Ok(())
    }

    async fn get_stats(&self) -> (u64, u64) {
        let events = *self.event_counter.read().await;
        let errors = *self.error_counter.read().await;
        (events, errors)
    }
}

/// 負載測試配置
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub concurrent_users: u32,
    pub duration: Duration,
    pub ramp_up_time: Duration,
    pub commands_per_second: f64,
    pub command_distribution: HashMap<String, f64>, // 命令類型分布
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        let mut command_distribution = HashMap::new();
        command_distribution.insert("balance".to_string(), 0.6);    // 60% 餘額查詢
        command_distribution.insert("transfer".to_string(), 0.2);    // 20% 轉帳
        command_distribution.insert("history".to_string(), 0.15);    // 15% 歷史查詢
        command_distribution.insert("help".to_string(), 0.05);       // 5% 幫助

        Self {
            concurrent_users: 1000,
            duration: Duration::from_secs(60),
            ramp_up_time: Duration::from_secs(10),
            commands_per_second: 100.0,
            command_distribution,
        }
    }
}

/// 負載測試生成器
struct LoadTestGenerator {
    config: LoadTestConfig,
    current_user_id: u64,
}

impl LoadTestGenerator {
    fn new(config: LoadTestConfig) -> Self {
        Self {
            config,
            current_user_id: 1,
        }
    }

    fn generate_command(&mut self) -> MockDiscordCommand {
        let rand_value = fastrand::f64();
        let mut cumulative = 0.0;

        // 先收集命令分布，避免借用衝突
        let command_distribution: Vec<(String, f64)> = self.config.command_distribution.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        for (command_type, probability) in command_distribution {
            cumulative += probability;
            if rand_value <= cumulative {
                let user_id = self.get_next_user_id();

                match command_type.as_str() {
                    "balance" => return MockDiscordCommand::Balance { user_id },
                    "transfer" => {
                        let from_user_id = user_id;
                        let to_user_id = self.get_random_other_user(from_user_id);
                        let amount = BigDecimal::from_str(&format!("{}", fastrand::u64(1..=1000))).unwrap();
                        return MockDiscordCommand::Transfer { from_user_id, to_user_id, amount };
                    }
                    "history" => return MockDiscordCommand::History { user_id },
                    "help" => return MockDiscordCommand::Help { user_id },
                    _ => return MockDiscordCommand::Balance { user_id },
                }
            }
        }

        // 默認返回餘額查詢
        MockDiscordCommand::Balance { user_id: self.get_next_user_id() }
    }

    fn get_next_user_id(&mut self) -> u64 {
        let user_id = self.current_user_id;
        self.current_user_id = if self.current_user_id >= self.config.concurrent_users as u64 {
            1
        } else {
            self.current_user_id + 1
        };
        user_id
    }

    fn get_random_other_user(&self, current_user: u64) -> u64 {
        let mut other_user = fastrand::u64(1..=self.config.concurrent_users as u64);
        while other_user == current_user {
            other_user = fastrand::u64(1..=self.config.concurrent_users as u64);
        }
        other_user
    }
}

/// 執行負載測試
pub async fn run_load_test(
    gateway: Arc<MockDiscordGateway>,
    config: LoadTestConfig,
) -> PerformanceTestResult {
    let test_name = format!("Load Test - {} users, {}s duration, {:.1} cmd/s",
                           config.concurrent_users,
                           config.duration.as_secs(),
                           config.commands_per_second);

    println!("開始負載測試: {}", test_name);

    let semaphore = Arc::new(Semaphore::new(config.concurrent_users as usize));
    let mut generator = LoadTestGenerator::new(config.clone());
    let start_time = Instant::now();
    let end_time = start_time + config.duration;

    let mut tasks = Vec::new();
    let mut response_times = Vec::new();

    // 計算命令生成間隔
    let command_interval = Duration::from_secs_f64(1.0 / config.commands_per_second);

    while Instant::now() < end_time {
        let gateway = Arc::clone(&gateway);
        let semaphore = Arc::clone(&semaphore);
        let command = generator.generate_command();

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let operation_start = Instant::now();

            let event = MockDiscordEvent::new(command, fastrand::u64(1..=10000));
            let result = gateway.process_event(event).await;

            let response_time = operation_start.elapsed();
            (result.is_ok(), response_time)
        });

        tasks.push(task);

        // 控制命令生成速率
        sleep(command_interval).await;
    }

    // 等待所有任務完成
    let mut successful_operations = 0;
    let mut failed_operations = 0;

    for task in tasks {
        match task.await {
            Ok((success, response_time)) => {
                response_times.push(response_time);
                if success {
                    successful_operations += 1;
                } else {
                    failed_operations += 1;
                }
            }
            Err(_) => {
                failed_operations += 1;
            }
        }
    }

    let duration = start_time.elapsed();
    let total_operations = successful_operations + failed_operations;

    // 獲取網關統計
    let (events_processed, errors) = gateway.get_stats().await;
    println!("網關處理事件數: {}", events_processed);
    println!("網關錯誤數: {}", errors);

    let result = PerformanceTestResult::new(
        test_name,
        total_operations,
        successful_operations,
        failed_operations,
        duration,
        response_times,
    );

    result.print();
    result
}

/// 高負載測試 - 1000+ 並發用戶
pub async fn test_high_load_performance() -> PerformanceTestResult {
    let cache = Arc::new(BalanceCache::new());
    let balance_service = Arc::new(MockBalanceService::new(cache, Duration::from_millis(5)));
    let gateway = Arc::new(MockDiscordGateway::new(balance_service));

    let config = LoadTestConfig {
        concurrent_users: 1000,
        duration: Duration::from_secs(30),
        ramp_up_time: Duration::from_secs(5),
        commands_per_second: 200.0,
        command_distribution: {
            let mut dist = HashMap::new();
            dist.insert("balance".to_string(), 0.7);
            dist.insert("transfer".to_string(), 0.15);
            dist.insert("history".to_string(), 0.1);
            dist.insert("help".to_string(), 0.05);
            dist
        },
    };

    run_load_test(gateway, config).await
}

/// 峰值負載測試
pub async fn test_peak_load_performance() -> PerformanceTestResult {
    let cache = Arc::new(BalanceCache::new());
    let balance_service = Arc::new(MockBalanceService::new(cache, Duration::from_millis(3)));
    let gateway = Arc::new(MockDiscordGateway::new(balance_service));

    let config = LoadTestConfig {
        concurrent_users: 1500,
        duration: Duration::from_secs(20),
        ramp_up_time: Duration::from_secs(3),
        commands_per_second: 500.0,
        command_distribution: {
            let mut dist = HashMap::new();
            dist.insert("balance".to_string(), 0.8);
            dist.insert("transfer".to_string(), 0.1);
            dist.insert("history".to_string(), 0.05);
            dist.insert("help".to_string(), 0.05);
            dist
        },
    };

    run_load_test(gateway, config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use droas_bot::cache::BalanceCache;

    #[tokio::test]
    async fn test_load_basic() {
        let cache = Arc::new(BalanceCache::new());
        let balance_service = Arc::new(MockBalanceService::new(cache, Duration::from_millis(1)));
        let gateway = Arc::new(MockDiscordGateway::new(balance_service));

        let config = LoadTestConfig {
            concurrent_users: 10,
            duration: Duration::from_secs(5),
            ramp_up_time: Duration::from_secs(1),
            commands_per_second: 20.0,
            command_distribution: HashMap::new(),
        };

        let result = run_load_test(gateway, config).await;

        // 基本負載測試應該成功
        assert!(result.successful_operations > 0);
        assert!(result.meets_nfr_requirements());
    }

    #[tokio::test]
    async fn test_load_generator() {
        let config = LoadTestConfig::default();
        let mut generator = LoadTestGenerator::new(config);

        // 生成一些命令並驗證分布
        let mut command_counts = HashMap::new();
        for _ in 0..1000 {
            let command = generator.generate_command();
            let command_type = match command {
                MockDiscordCommand::Balance { .. } => "balance",
                MockDiscordCommand::Transfer { .. } => "transfer",
                MockDiscordCommand::History { .. } => "history",
                MockDiscordCommand::Help { .. } => "help",
            };
            *command_counts.entry(command_type).or_insert(0) += 1;
        }

        // 驗證命令分布大致符合預期
        let total_commands: i32 = command_counts.values().sum();
        let balance_ratio = *command_counts.get("balance").unwrap_or(&0) as f64 / total_commands as f64;

        assert!((balance_ratio - 0.6).abs() < 0.1); // 允許 10% 的誤差
    }
}