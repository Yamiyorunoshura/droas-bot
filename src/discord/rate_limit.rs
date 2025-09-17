use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, info, warn};
use anyhow::Result;

/// 速率限制信息
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// 限制窗口內允許的請求數
    pub limit: u32,
    /// 剩餘請求數
    pub remaining: u32,
    /// 重置時間
    pub reset_at: Instant,
    /// 重置後時間（秒）
    pub reset_after: Duration,
    /// 是否為全域限制
    pub global: bool,
}

/// 路由特定的速率限制信息
#[derive(Debug, Clone)]
struct RouteRateLimit {
    /// 速率限制信息
    rate_limit: RateLimit,
    /// 路由標識符
    route_id: String,
}

/// 指數退避重試配置
#[derive(Debug, Clone)]
pub struct ExponentialBackoffConfig {
    /// 基礎延遲（毫秒）
    pub base_delay_ms: u64,
    /// 最大延遲（毫秒）
    pub max_delay_ms: u64,
    /// 最大重試次數
    pub max_retries: u32,
    /// 退避倍數（通常是2.0）
    pub backoff_multiplier: f64,
    /// 加入隨機抖動以避免「驚群效應」
    pub jitter: bool,
}

impl Default for ExponentialBackoffConfig {
    fn default() -> Self {
        Self {
            base_delay_ms: 1000,      // 1 秒
            max_delay_ms: 32000,      // 32 秒
            max_retries: 3,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// 重試狀態追蹤
#[derive(Debug, Clone)]
struct RetryState {
    /// 當前重試次數
    attempt: u32,
    /// 下次重試延遲
    next_delay_ms: u64,
    /// 最後重試時間
    last_attempt: Instant,
}

impl RetryState {
    fn new(config: &ExponentialBackoffConfig) -> Self {
        Self {
            attempt: 0,
            next_delay_ms: config.base_delay_ms,
            last_attempt: Instant::now(),
        }
    }
}

/// 速率限制統計
#[derive(Debug, Clone, Default)]
pub struct RateLimitStats {
    /// 總請求數
    pub requests_made: u64,
    /// 速率限制觸發次數
    pub rate_limit_hits: u64,
    /// 全域限制觸發次數
    pub global_rate_limit_hits: u64,
    /// 平均等待時間（毫秒）
    pub average_wait_time_ms: u64,
}

/// Discord API 速率限制管理器
/// 
/// 負責處理 Discord API 的速率限制，包括全域限制和路由特定限制。
/// 實現指數退避算法和智能重試機制。
pub struct RateLimiter {
    /// 全域速率限制
    global_rate_limit: RwLock<Option<RateLimit>>,
    /// 路由特定速率限制
    route_rate_limits: RwLock<HashMap<String, RouteRateLimit>>,
    /// 統計信息
    stats: RateLimitStats,
    /// 請求計數器
    request_counter: AtomicU64,
    /// 限制觸發計數器
    rate_limit_counter: AtomicU64,
    /// 全域限制觸發計數器
    global_limit_counter: AtomicU64,
    /// 總等待時間（用於計算平均值）
    total_wait_time: AtomicU64,
    /// 指數退避配置
    backoff_config: ExponentialBackoffConfig,
    /// 各路由的重試狀態
    retry_states: RwLock<HashMap<String, RetryState>>,
}

impl RateLimiter {
    /// 創建新的速率限制管理器
    pub fn new() -> Self {
        Self {
            global_rate_limit: RwLock::new(None),
            route_rate_limits: RwLock::new(HashMap::new()),
            stats: RateLimitStats::default(),
            request_counter: AtomicU64::new(0),
            rate_limit_counter: AtomicU64::new(0),
            global_limit_counter: AtomicU64::new(0),
            total_wait_time: AtomicU64::new(0),
            backoff_config: ExponentialBackoffConfig::default(),
            retry_states: RwLock::new(HashMap::new()),
        }
    }
    
    /// 使用自定義配置創建速率限制管理器
    pub fn with_config(backoff_config: ExponentialBackoffConfig) -> Self {
        Self {
            global_rate_limit: RwLock::new(None),
            route_rate_limits: RwLock::new(HashMap::new()),
            stats: RateLimitStats::default(),
            request_counter: AtomicU64::new(0),
            rate_limit_counter: AtomicU64::new(0),
            global_limit_counter: AtomicU64::new(0),
            total_wait_time: AtomicU64::new(0),
            backoff_config,
            retry_states: RwLock::new(HashMap::new()),
        }
    }
    
    /// 在發送請求前檢查速率限制
    /// 
    /// 如果需要等待，此方法會自動延遲適當的時間。
    /// 
    /// # Arguments
    /// * `route` - API 路由標識符
    /// 
    /// # Returns
    /// 等待的時間（如果有的話）
    pub async fn wait_if_rate_limited(&self, route: &str) -> Duration {
        let _start_time = Instant::now();
        let mut total_wait = Duration::from_secs(0);
        
        // 檢查全域限制
        if let Some(wait_time) = self.check_global_rate_limit().await {
            debug!("等待全域速率限制: {:.2}s", wait_time.as_secs_f64());
            sleep(wait_time).await;
            total_wait += wait_time;
            
            // 更新統計
            self.global_limit_counter.fetch_add(1, Ordering::Relaxed);
        }
        
        // 檢查路由特定限制
        if let Some(wait_time) = self.check_route_rate_limit(route).await {
            debug!("等待路由 {} 速率限制: {:.2}s", route, wait_time.as_secs_f64());
            sleep(wait_time).await;
            total_wait += wait_time;
            
            // 更新統計
            self.rate_limit_counter.fetch_add(1, Ordering::Relaxed);
        }
        
        // 記錄請求
        self.request_counter.fetch_add(1, Ordering::Relaxed);
        
        // 更新等待時間統計
        if total_wait > Duration::from_secs(0) {
            let wait_ms = total_wait.as_millis() as u64;
            self.total_wait_time.fetch_add(wait_ms, Ordering::Relaxed);
        }
        
        total_wait
    }
    
    /// 處理 HTTP 429 回應
    /// 
    /// 根據回應標頭更新速率限制信息。
    /// 
    /// # Arguments
    /// * `route` - API 路由標識符
    /// * `retry_after` - 重試等待時間（秒）
    /// * `global` - 是否為全域限制
    pub async fn handle_rate_limit_response(
        &self, 
        route: &str, 
        retry_after: f64, 
        global: bool
    ) {
        let wait_duration = Duration::from_secs_f64(retry_after);
        let reset_at = Instant::now() + wait_duration;
        
        let rate_limit = RateLimit {
            limit: 1, // 從 429 回應無法得知確切限制，設為最小值
            remaining: 0,
            reset_at,
            reset_after: wait_duration,
            global,
        };
        
        if global {
            info!("收到全域速率限制，等待 {:.2} 秒", retry_after);
            let mut global_limit = self.global_rate_limit.write().await;
            *global_limit = Some(rate_limit);
        } else {
            info!("路由 {} 收到速率限制，等待 {:.2} 秒", route, retry_after);
            let mut route_limits = self.route_rate_limits.write().await;
            route_limits.insert(route.to_string(), RouteRateLimit {
                rate_limit,
                route_id: route.to_string(),
            });
        }
    }
    
    /// 更新成功請求的速率限制信息
    /// 
    /// 根據回應標頭更新剩餘請求數和重置時間。
    /// 
    /// # Arguments
    /// * `route` - API 路由標識符
    /// * `limit` - 速率限制數量
    /// * `remaining` - 剩餘請求數
    /// * `reset_after` - 重置等待時間（秒）
    pub async fn update_rate_limit_info(
        &self,
        route: &str,
        limit: u32,
        remaining: u32,
        reset_after: f64,
    ) {
        let reset_duration = Duration::from_secs_f64(reset_after);
        let reset_at = Instant::now() + reset_duration;
        
        let rate_limit = RateLimit {
            limit,
            remaining,
            reset_at,
            reset_after: reset_duration,
            global: false,
        };
        
        let mut route_limits = self.route_rate_limits.write().await;
        route_limits.insert(route.to_string(), RouteRateLimit {
            rate_limit,
            route_id: route.to_string(),
        });
        
        // 如果剩餘請求數很少，提前警告
        if remaining <= 2 && remaining > 0 {
            warn!("路由 {} 速率限制即將耗盡：{}/{}", route, remaining, limit);
        }
    }
    
    /// 檢查全域速率限制
    async fn check_global_rate_limit(&self) -> Option<Duration> {
        let global_limit = self.global_rate_limit.read().await;
        
        if let Some(ref limit) = *global_limit {
            if Instant::now() < limit.reset_at {
                return Some(limit.reset_at.duration_since(Instant::now()));
            }
        }
        
        None
    }
    
    /// 檢查路由特定速率限制
    async fn check_route_rate_limit(&self, route: &str) -> Option<Duration> {
        let route_limits = self.route_rate_limits.read().await;
        
        if let Some(route_limit) = route_limits.get(route) {
            let limit = &route_limit.rate_limit;
            
            // 如果還在限制窗口內且沒有剩餘請求
            if Instant::now() < limit.reset_at && limit.remaining == 0 {
                return Some(limit.reset_at.duration_since(Instant::now()));
            }
        }
        
        None
    }
    
    /// 清理過期的速率限制信息
    pub async fn cleanup_expired_limits(&self) {
        let now = Instant::now();
        
        // 清理全域限制
        {
            let mut global_limit = self.global_rate_limit.write().await;
            if let Some(ref limit) = *global_limit {
                if now >= limit.reset_at {
                    *global_limit = None;
                    debug!("已清理過期的全域速率限制");
                }
            }
        }
        
        // 清理路由特定限制
        {
            let mut route_limits = self.route_rate_limits.write().await;
            route_limits.retain(|route, route_limit| {
                let keep = now < route_limit.rate_limit.reset_at;
                if !keep {
                    debug!("已清理路由 {} 的過期速率限制", route);
                }
                keep
            });
        }
    }
    
    /// 獲取統計信息
    pub fn get_stats(&self) -> RateLimitStats {
        let requests = self.request_counter.load(Ordering::Relaxed);
        let rate_limits = self.rate_limit_counter.load(Ordering::Relaxed);
        let global_limits = self.global_limit_counter.load(Ordering::Relaxed);
        let total_wait = self.total_wait_time.load(Ordering::Relaxed);
        
        let average_wait = if rate_limits + global_limits > 0 {
            total_wait / (rate_limits + global_limits)
        } else {
            0
        };
        
        RateLimitStats {
            requests_made: requests,
            rate_limit_hits: rate_limits,
            global_rate_limit_hits: global_limits,
            average_wait_time_ms: average_wait,
        }
    }
    
    /// 獲取所有活躍的速率限制信息
    pub async fn get_active_limits(&self) -> Vec<(String, RateLimit)> {
        let mut limits = Vec::new();
        
        // 全域限制
        if let Some(ref global_limit) = *self.global_rate_limit.read().await {
            limits.push(("GLOBAL".to_string(), global_limit.clone()));
        }
        
        // 路由特定限制
        for (route, route_limit) in self.route_rate_limits.read().await.iter() {
            limits.push((route.clone(), route_limit.rate_limit.clone()));
        }
        
        limits
    }
    
    /// 帶指數退避的重試機制
    /// 
    /// 用於處理網路錯誤或其他暫時性失敗的重試。
    /// 
    /// # Arguments
    /// * `route` - API 路由標識符
    /// * `operation` - 要重試的操作（異步闉包）
    /// 
    /// # Returns
    /// * `Ok(T)` - 操作成功的結果
    /// * `Err(...)` - 達到最大重試次數後的最後錯誤
    pub async fn retry_with_exponential_backoff<T, F, Fut, E>(
        &self,
        route: &str,
        mut operation: F,
    ) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut retry_state = {
            let mut states = self.retry_states.write().await;
            states.entry(route.to_string())
                .or_insert_with(|| RetryState::new(&self.backoff_config))
                .clone()
        };
        
        loop {
            // 先檢查速率限制
            self.wait_if_rate_limited(route).await;
            
            // 執行操作
            match operation().await {
                Ok(result) => {
                    // 成功，清除重試狀態
                    self.retry_states.write().await.remove(route);
                    return Ok(result);
                }
                Err(error) => {
                    retry_state.attempt += 1;
                    
                    // 檢查是否已達到最大重試次數
                    if retry_state.attempt >= self.backoff_config.max_retries {
                        debug!(
                            "路由 {} 達到最大重試次數 {}，停止重試",
                            route, 
                            self.backoff_config.max_retries
                        );
                        self.retry_states.write().await.remove(route);
                        return Err(error);
                    }
                    
                    // 計算等待時間
                    let delay_ms = self.calculate_backoff_delay(&retry_state);
                    let delay = Duration::from_millis(delay_ms);
                    
                    info!(
                        "路由 {} 第 {} 次重試失敗，{:.2}秒後重試：{:?}", 
                        route, 
                        retry_state.attempt, 
                        delay.as_secs_f64(),
                        error
                    );
                    
                    // 等待
                    sleep(delay).await;
                    
                    // 更新下次延遲
                    retry_state.next_delay_ms = std::cmp::min(
                        (retry_state.next_delay_ms as f64 * self.backoff_config.backoff_multiplier) as u64,
                        self.backoff_config.max_delay_ms
                    );
                    retry_state.last_attempt = Instant::now();
                    
                    // 更新狀態
                    {
                        let mut states = self.retry_states.write().await;
                        states.insert(route.to_string(), retry_state.clone());
                    }
                }
            }
        }
    }
    
    /// 計算指數退避延遲時間
    fn calculate_backoff_delay(&self, retry_state: &RetryState) -> u64 {
        let mut delay_ms = retry_state.next_delay_ms;
        
        // 如果啟用了随機抖動
        if self.backoff_config.jitter {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            // 使用伸虛隨機數生成器來避免引入新的依賴
            let mut hasher = DefaultHasher::new();
            retry_state.attempt.hash(&mut hasher);
            retry_state.last_attempt.elapsed().as_nanos().hash(&mut hasher);
            let hash = hasher.finish();
            
            // 將抖動範圍設為 ±25%
            let jitter_factor = 0.75 + (hash % 500) as f64 / 2000.0; // 0.75 ~ 1.0
            delay_ms = (delay_ms as f64 * jitter_factor) as u64;
        }
        
        // 確保不超過最大延遲
        std::cmp::min(delay_ms, self.backoff_config.max_delay_ms)
    }
    
    /// 重設指定路由的重試狀態
    pub async fn reset_retry_state(&self, route: &str) {
        self.retry_states.write().await.remove(route);
    }
    
    /// 重設所有重試狀態
    pub async fn reset_all_retry_states(&self) {
        self.retry_states.write().await.clear();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tokio::time::{pause, advance, resume}; // 不再需要時間模擬
    
    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new();
        let stats = limiter.get_stats();
        
        assert_eq!(stats.requests_made, 0);
        assert_eq!(stats.rate_limit_hits, 0);
        assert_eq!(stats.global_rate_limit_hits, 0);
    }
    
    #[tokio::test]
    async fn test_no_rate_limit() {
        let limiter = RateLimiter::new();
        
        let wait_time = limiter.wait_if_rate_limited("test_route").await;
        assert_eq!(wait_time, Duration::from_secs(0));
        
        let stats = limiter.get_stats();
        assert_eq!(stats.requests_made, 1);
        assert_eq!(stats.rate_limit_hits, 0);
    }
    
    #[tokio::test]
    async fn test_route_rate_limit() {
        let limiter = RateLimiter::new();
        
        // 設置速率限制
        limiter.handle_rate_limit_response("test_route", 1.0, false).await;
        
        // 應該需要等待
        let wait_time = limiter.wait_if_rate_limited("test_route").await;
        assert!(wait_time > Duration::from_millis(500)); // 應該接近 1 秒
        
        let stats = limiter.get_stats();
        assert_eq!(stats.rate_limit_hits, 1);
    }
    
    #[tokio::test]
    async fn test_global_rate_limit() {
        let limiter = RateLimiter::new();
        
        // 設置全域速率限制
        limiter.handle_rate_limit_response("any_route", 0.5, true).await;
        
        // 任何路由都應該等待
        let wait_time = limiter.wait_if_rate_limited("different_route").await;
        assert!(wait_time > Duration::from_millis(250));
        
        let stats = limiter.get_stats();
        assert_eq!(stats.global_rate_limit_hits, 1);
    }
    
    #[tokio::test]
    async fn test_rate_limit_info_update() {
        let limiter = RateLimiter::new();
        
        // 更新速率限制信息
        limiter.update_rate_limit_info("test_route", 100, 50, 30.0).await;
        
        let limits = limiter.get_active_limits().await;
        assert_eq!(limits.len(), 1);
        assert_eq!(limits[0].0, "test_route");
        assert_eq!(limits[0].1.limit, 100);
        assert_eq!(limits[0].1.remaining, 50);
    }
    
    #[tokio::test]
    async fn test_cleanup_expired_limits() {
        let limiter = RateLimiter::new();
        
        // 添加短期限制 (0.05 秒)
        limiter.handle_rate_limit_response("test_route", 0.05, false).await;
        
        // 驗證限制存在
        let limits = limiter.get_active_limits().await;
        assert_eq!(limits.len(), 1);
        
        // 實際等待讓限制過期
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 清理過期限制
        limiter.cleanup_expired_limits().await;
        
        // 驗證限制已清理
        let limits = limiter.get_active_limits().await;
        assert_eq!(limits.len(), 0);
    }
    
    #[tokio::test]
    async fn test_stats_calculation() {
        let limiter = RateLimiter::new();
        
        // 執行幾個請求
        limiter.wait_if_rate_limited("route1").await;
        limiter.wait_if_rate_limited("route2").await;
        
        let stats = limiter.get_stats();
        assert_eq!(stats.requests_made, 2);
    }
    
    #[tokio::test]
    async fn test_exponential_backoff_config() {
        use std::sync::{Arc, Mutex};
        
        let config = ExponentialBackoffConfig {
            base_delay_ms: 10,  // 使用較短的延遲以加速測試
            max_delay_ms: 1000,
            max_retries: 3,  // 允許最多3次重試（第4次嘗試成功）
            backoff_multiplier: 2.0,
            jitter: false, // 關閉抖動以便測試
        };
        
        let limiter = RateLimiter::with_config(config);
        
        // 使用 Arc<Mutex<>> 管理可變狀態
        let attempt_count = Arc::new(Mutex::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);
        
        let result = limiter.retry_with_exponential_backoff(
            "test_route",
            move || {
                let counter = Arc::clone(&attempt_count_clone);
                async move {
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                    let current_attempt = *count;
                    drop(count); // 釋放鎖
                    
                    if current_attempt <= 2 {
                        Err("Mock error")
                    } else {
                        Ok("Success")
                    }
                }
            }
        ).await;
        
        // 應該在第3次嘗試時成功
        assert_eq!(result.unwrap(), "Success");
        let final_count = *attempt_count.lock().unwrap();
        assert_eq!(final_count, 3);
    }
    
    #[tokio::test]
    async fn test_exponential_backoff_max_retries() {
        let config = ExponentialBackoffConfig {
            base_delay_ms: 10,  // 短延遲以速速測試
            max_delay_ms: 100,
            max_retries: 2,
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let limiter = RateLimiter::with_config(config);
        
        // 模擬始終失敗的操作
        let result = limiter.retry_with_exponential_backoff(
            "failing_route",
            || async { Err::<String, &str>("Always fails") }
        ).await;
        
        // 應該在達到最大重試次數後失敗
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Always fails");
    }
    
    #[tokio::test]
    async fn test_backoff_delay_calculation() {
        let config = ExponentialBackoffConfig {
            base_delay_ms: 100,
            max_delay_ms: 1000,
            max_retries: 5,
            backoff_multiplier: 2.0,
            jitter: false,
        };
        
        let limiter = RateLimiter::with_config(config);
        let mut retry_state = RetryState::new(&limiter.backoff_config);
        
        // 測試逆增長延遲
        assert_eq!(limiter.calculate_backoff_delay(&retry_state), 100);
        
        retry_state.next_delay_ms = 200;
        assert_eq!(limiter.calculate_backoff_delay(&retry_state), 200);
        
        retry_state.next_delay_ms = 400;
        assert_eq!(limiter.calculate_backoff_delay(&retry_state), 400);
        
        // 測試最大值限制
        retry_state.next_delay_ms = 2000; // 超過 max_delay_ms
        assert_eq!(limiter.calculate_backoff_delay(&retry_state), 1000); // 應該被限制
    }
    
    #[tokio::test]
    async fn test_retry_state_reset() {
        use std::sync::{Arc, Mutex};
        
        let limiter = RateLimiter::new();
        
        // 使用 Arc<Mutex<>> 管理可變狀態
        let attempt = Arc::new(Mutex::new(0));
        let attempt_clone = Arc::clone(&attempt);
        
        let _result = limiter.retry_with_exponential_backoff(
            "reset_test",
            move || {
                let counter = Arc::clone(&attempt_clone);
                async move {
                    let mut count = counter.lock().unwrap();
                    *count += 1;
                    let current_attempt = *count;
                    drop(count);
                    
                    if current_attempt == 1 {
                        Err("First attempt fails")
                    } else {
                        Ok("Success on second attempt")
                    }
                }
            }
        ).await;
        
        // 成功後，重試狀態應該被清除
        let states = limiter.retry_states.read().await;
        assert!(!states.contains_key("reset_test"));
    }
}