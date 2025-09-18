//! 熔斷機制模組
//!
//! 實作 Circuit Breaker 模式，保護系統免受外部服務過載和不可用的影響。

use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 熔斷器狀態
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// 關閉狀態 - 正常操作
    Closed = 0,
    /// 開啟狀態 - 熔斷器觸發，拒絕請求
    Open = 1,
    /// 半開狀態 - 嘗試恢復服務
    HalfOpen = 2,
}

impl From<u8> for CircuitState {
    fn from(value: u8) -> Self {
        match value {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Open, // 預設為開啟以保護系統
        }
    }
}

/// 熔斷器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 失敗閾值 - 多少次失敗後觸發熔斷
    pub failure_threshold: u64,
    /// 恢復超時 - 開啟狀態持續多久後嘗試半開
    pub recovery_timeout: Duration,
    /// 成功閾值 - 半開狀態下多少次成功後關閉熔斷器
    pub success_threshold: u64,
    /// 請求超時 - 單個請求的最大等待時間
    pub request_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
            request_timeout: Duration::from_secs(10),
        }
    }
}

/// 熔斷器統計資訊
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// 總請求數
    pub total_requests: u64,
    /// 成功請求數
    pub successful_requests: u64,
    /// 失敗請求數
    pub failed_requests: u64,
    /// 被熔斷器拒絕的請求數
    pub rejected_requests: u64,
    /// 熔斷器狀態變化次數
    pub state_changes: u64,
    /// 上次狀態變化時間
    pub last_state_change: Option<Instant>,
    /// 當前狀態持續時間
    pub current_state_duration: Duration,
}

/// 熔斷器內部狀態
struct CircuitBreakerState {
    /// 連續失敗次數
    consecutive_failures: u64,
    /// 半開狀態下的成功次數
    half_open_successes: u64,
    /// 上次失敗時間
    last_failure_time: Option<Instant>,
    /// 狀態變化次數
    state_changes: u64,
    /// 上次狀態變化時間
    last_state_change: Option<Instant>,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self {
            consecutive_failures: 0,
            half_open_successes: 0,
            last_failure_time: None,
            state_changes: 0,
            last_state_change: Some(Instant::now()),
        }
    }
}

/// 熔斷器實現
pub struct CircuitBreaker {
    /// 配置
    config: CircuitBreakerConfig,
    /// 當前狀態（使用 AtomicU8 存儲）
    state: AtomicU8,
    /// 內部狀態（需要鎖保護）
    internal_state: RwLock<CircuitBreakerState>,
    /// 統計計數器
    total_requests: AtomicU64,
    successful_requests: AtomicU64,
    failed_requests: AtomicU64,
    rejected_requests: AtomicU64,
}

impl CircuitBreaker {
    /// 創建新的熔斷器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        info!(
            "創建熔斷器，配置: 失敗閾值={}, 恢復超時={}s, 成功閾值={}",
            config.failure_threshold,
            config.recovery_timeout.as_secs(),
            config.success_threshold
        );

        Self {
            config,
            state: AtomicU8::new(CircuitState::Closed as u8),
            internal_state: RwLock::new(CircuitBreakerState::default()),
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            rejected_requests: AtomicU64::new(0),
        }
    }

    /// 使用預設配置創建熔斷器
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// 檢查是否允許執行請求
    pub async fn can_execute(&self) -> bool {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        let current_state = CircuitState::from(self.state.load(Ordering::Acquire));

        match current_state {
            CircuitState::Closed => {
                debug!("熔斷器關閉狀態，允許請求");
                true
            }
            CircuitState::Open => {
                // 檢查是否應該嘗試半開
                if self.should_attempt_reset().await {
                    debug!("熔斷器嘗試從開啟狀態轉為半開狀態");
                    self.transition_to_half_open().await;
                    true
                } else {
                    debug!("熔斷器開啟狀態，拒絕請求");
                    self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
            CircuitState::HalfOpen => {
                debug!("熔斷器半開狀態，允許探測請求");
                true
            }
        }
    }

    /// 記錄成功執行
    pub async fn record_success(&self) {
        self.successful_requests.fetch_add(1, Ordering::Relaxed);

        let current_state = CircuitState::from(self.state.load(Ordering::Acquire));

        match current_state {
            CircuitState::Closed => {
                // 關閉狀態下的成功不需要特殊處理
                debug!("熔斷器關閉狀態記錄成功");
            }
            CircuitState::HalfOpen => {
                // 半開狀態下需要計算連續成功次數
                let mut internal_state = self.internal_state.write().await;
                internal_state.half_open_successes += 1;

                debug!(
                    "熔斷器半開狀態記錄成功，連續成功次數: {}",
                    internal_state.half_open_successes
                );

                if internal_state.half_open_successes >= self.config.success_threshold {
                    info!("熔斷器達到成功閾值，從半開狀態轉為關閉狀態");
                    self.transition_to_closed(&mut internal_state).await;
                }
            }
            CircuitState::Open => {
                // 開啟狀態下不應該有請求執行，但記錄以防萬一
                warn!("熔斷器開啟狀態意外記錄成功");
            }
        }
    }

    /// 記錄失敗執行
    pub async fn record_failure(&self) {
        self.failed_requests.fetch_add(1, Ordering::Relaxed);

        let current_state = CircuitState::from(self.state.load(Ordering::Acquire));
        let mut internal_state = self.internal_state.write().await;

        internal_state.consecutive_failures += 1;
        internal_state.last_failure_time = Some(Instant::now());

        debug!(
            "熔斷器記錄失敗，連續失敗次數: {}",
            internal_state.consecutive_failures
        );

        match current_state {
            CircuitState::Closed => {
                if internal_state.consecutive_failures >= self.config.failure_threshold {
                    warn!("熔斷器達到失敗閾值，從關閉狀態轉為開啟狀態");
                    self.transition_to_open(&mut internal_state).await;
                }
            }
            CircuitState::HalfOpen => {
                // 半開狀態下任何失敗都會回到開啟狀態
                warn!("熔斷器半開狀態記錄失敗，轉為開啟狀態");
                self.transition_to_open(&mut internal_state).await;
            }
            CircuitState::Open => {
                // 開啟狀態下不應該有請求執行，但更新失敗時間
                debug!("熔斷器開啟狀態記錄失敗");
            }
        }
    }

    /// 執行受保護的操作
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        if !self.can_execute().await {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        // 使用超時包裝操作
        let result = tokio::time::timeout(self.config.request_timeout, operation).await;

        match result {
            Ok(Ok(success)) => {
                self.record_success().await;
                Ok(success)
            }
            Ok(Err(error)) => {
                self.record_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
            Err(_timeout) => {
                self.record_failure().await;
                Err(CircuitBreakerError::Timeout)
            }
        }
    }

    /// 獲取當前狀態
    pub fn get_state(&self) -> CircuitState {
        CircuitState::from(self.state.load(Ordering::Acquire))
    }

    /// 獲取統計資訊
    pub async fn get_stats(&self) -> CircuitBreakerStats {
        let internal_state = self.internal_state.read().await;
        let current_time = Instant::now();

        let current_state_duration = if let Some(last_change) = internal_state.last_state_change {
            current_time.duration_since(last_change)
        } else {
            Duration::from_secs(0)
        };

        CircuitBreakerStats {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            rejected_requests: self.rejected_requests.load(Ordering::Relaxed),
            state_changes: internal_state.state_changes,
            last_state_change: internal_state.last_state_change,
            current_state_duration,
        }
    }

    /// 手動重置熔斷器為關閉狀態
    pub async fn reset(&self) {
        let mut internal_state = self.internal_state.write().await;
        info!("手動重置熔斷器為關閉狀態");
        self.transition_to_closed(&mut internal_state).await;
    }

    /// 檢查是否應該嘗試從開啟狀態重置
    async fn should_attempt_reset(&self) -> bool {
        let internal_state = self.internal_state.read().await;

        if let Some(last_failure) = internal_state.last_failure_time {
            last_failure.elapsed() >= self.config.recovery_timeout
        } else {
            false
        }
    }

    /// 轉換到關閉狀態
    async fn transition_to_closed(&self, internal_state: &mut CircuitBreakerState) {
        self.state
            .store(CircuitState::Closed as u8, Ordering::Release);
        internal_state.consecutive_failures = 0;
        internal_state.half_open_successes = 0;
        internal_state.state_changes += 1;
        internal_state.last_state_change = Some(Instant::now());

        info!("熔斷器狀態轉換為關閉");
    }

    /// 轉換到開啟狀態
    async fn transition_to_open(&self, internal_state: &mut CircuitBreakerState) {
        self.state
            .store(CircuitState::Open as u8, Ordering::Release);
        internal_state.half_open_successes = 0;
        internal_state.state_changes += 1;
        internal_state.last_state_change = Some(Instant::now());

        error!(
            "熔斷器狀態轉換為開啟，將拒絕後續請求 {} 秒",
            self.config.recovery_timeout.as_secs()
        );
    }

    /// 轉換到半開狀態
    async fn transition_to_half_open(&self) {
        let mut internal_state = self.internal_state.write().await;
        self.state
            .store(CircuitState::HalfOpen as u8, Ordering::Release);
        internal_state.half_open_successes = 0;
        internal_state.state_changes += 1;
        internal_state.last_state_change = Some(Instant::now());

        info!("熔斷器狀態轉換為半開，開始探測服務可用性");
    }
}

/// 熔斷器錯誤類型
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// 熔斷器開啟，拒絕執行
    CircuitOpen,
    /// 操作超時
    Timeout,
    /// 操作失敗
    OperationFailed(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "熔斷器開啟，拒絕執行請求"),
            CircuitBreakerError::Timeout => write!(f, "操作超時"),
            CircuitBreakerError::OperationFailed(e) => write!(f, "操作失敗: {}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CircuitBreakerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CircuitBreakerError::OperationFailed(e) => Some(e),
            _ => None,
        }
    }
}

/// 包裝的熔斷器，便於在多執行緒環境中使用
pub type SharedCircuitBreaker = Arc<CircuitBreaker>;

/// 創建共享的熔斷器
pub fn create_shared_circuit_breaker(config: CircuitBreakerConfig) -> SharedCircuitBreaker {
    Arc::new(CircuitBreaker::new(config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_circuit_breaker_basic_flow() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 2,
            request_timeout: Duration::from_millis(50),
        };

        let cb = CircuitBreaker::new(config);

        // 初始狀態應該是關閉
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert!(cb.can_execute().await);

        // 記錄失敗直到觸發熔斷
        cb.record_failure().await;
        assert_eq!(cb.get_state(), CircuitState::Closed);

        cb.record_failure().await;
        assert_eq!(cb.get_state(), CircuitState::Open);

        // 開啟狀態下應該拒絕請求
        assert!(!cb.can_execute().await);

        // 等待恢復超時
        sleep(Duration::from_millis(150)).await;

        // 現在應該能夠嘗試半開
        assert!(cb.can_execute().await);
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // 記錄成功以關閉熔斷器
        cb.record_success().await;
        cb.record_success().await;
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_execute() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 1,
            request_timeout: Duration::from_millis(50),
        };

        let cb = CircuitBreaker::new(config);

        // 成功操作
        let result = cb.execute(async { Ok::<i32, &str>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // 失敗操作
        let result = cb.execute(async { Err::<i32, &str>("error") }).await;
        assert!(matches!(
            result,
            Err(CircuitBreakerError::OperationFailed("error"))
        ));

        // 再次失敗應該觸發熔斷
        let result = cb.execute(async { Err::<i32, &str>("error") }).await;
        assert!(matches!(
            result,
            Err(CircuitBreakerError::OperationFailed("error"))
        ));

        // 現在熔斷器應該開啟
        let result = cb.execute(async { Ok::<i32, &str>(42) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));
    }

    #[tokio::test]
    async fn test_circuit_breaker_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(1),
            success_threshold: 1,
            request_timeout: Duration::from_millis(50),
        };

        let cb = CircuitBreaker::new(config);

        // 超時操作
        let result = cb
            .execute(async {
                sleep(Duration::from_millis(100)).await;
                Ok::<i32, &str>(42)
            })
            .await;

        assert!(matches!(result, Err(CircuitBreakerError::Timeout)));
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let cb = CircuitBreaker::with_defaults();

        // 執行一些操作
        let _ = cb.can_execute().await;
        cb.record_success().await;

        let _ = cb.can_execute().await;
        cb.record_failure().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.rejected_requests, 0);
    }
}
