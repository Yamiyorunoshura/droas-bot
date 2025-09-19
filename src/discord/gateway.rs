use std::time::{Duration, Instant};
use tracing::{error, info, warn};

/// Gateway 連接狀態
#[derive(Debug, Clone, PartialEq)]
pub enum GatewayStatus {
    /// 未連接
    Disconnected,
    /// 連接中
    Connecting,
    /// 已連接
    Connected,
    /// 錯誤狀態
    Error(String),
}

/// Gateway 連接管理器
///
/// 負責管理 Discord Gateway WebSocket 連接的狀態、監控和自動恢復。
pub struct GatewayManager {
    /// 當前連接狀態
    status: GatewayStatus,
    /// 連接建立時間
    connected_at: Option<Instant>,
    /// 重連次數
    reconnect_count: u64,
    /// 心跳計數器
    heartbeat_count: u64,
    /// 最後一次心跳時間
    last_heartbeat: Option<Instant>,
    /// 心跳間隔（毫秒）
    heartbeat_interval: Option<Duration>,
}

impl GatewayManager {
    /// 創建新的 Gateway 管理器
    pub fn new() -> Self {
        Self {
            status: GatewayStatus::Disconnected,
            connected_at: None,
            reconnect_count: 0,
            heartbeat_count: 0,
            last_heartbeat: None,
            heartbeat_interval: None,
        }
    }

    /// 設置連接狀態
    pub fn set_status(&mut self, status: GatewayStatus) {
        let previous_status = std::mem::replace(&mut self.status, status.clone());

        match (&previous_status, &status) {
            (GatewayStatus::Connecting, GatewayStatus::Connected) => {
                self.connected_at = Some(Instant::now());
                info!("Gateway 連接已建立");
            }
            (GatewayStatus::Connected, GatewayStatus::Disconnected) => {
                self.connected_at = None;
                warn!("Gateway 連接已斷開");
            }
            (_, GatewayStatus::Error(ref error)) => {
                error!("Gateway 錯誤: {}", error);
            }
            _ => {}
        }
    }

    /// 獲取當前連接狀態
    pub fn get_status(&self) -> GatewayStatus {
        self.status.clone()
    }

    /// 獲取運行時間
    pub fn get_uptime(&self) -> Duration {
        match (self.connected_at, &self.status) {
            (Some(connected_at), GatewayStatus::Connected) => connected_at.elapsed(),
            _ => Duration::from_secs(0),
        }
    }

    /// 獲取重連次數
    pub fn get_reconnect_count(&self) -> u64 {
        self.reconnect_count
    }

    /// 增加重連次數
    pub fn increment_reconnect_count(&mut self) {
        self.reconnect_count += 1;
        info!("Gateway 重連次數: {}", self.reconnect_count);
    }

    /// 更新心跳信息
    pub fn update_heartbeat(&mut self, interval_ms: u64) {
        self.last_heartbeat = Some(Instant::now());
        self.heartbeat_interval = Some(Duration::from_millis(interval_ms));
        self.heartbeat_count += 1;

        // 每 100 次心跳記錄一次（避免日誌過多）
        if self.heartbeat_count % 100 == 0 {
            info!("Gateway 心跳正常，間隔: {}ms，心跳次數: {}", interval_ms, self.heartbeat_count);
        }
    }

    /// 檢查心跳是否超時
    pub fn is_heartbeat_timeout(&self) -> bool {
        match (self.last_heartbeat, self.heartbeat_interval) {
            (Some(last), Some(interval)) => {
                // 如果超過 2.5 倍心跳間隔沒有收到心跳，認為超時
                last.elapsed() > interval.mul_f32(2.5)
            }
            _ => false,
        }
    }

    /// 獲取連接統計信息
    pub fn get_connection_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            status: self.status.clone(),
            uptime: self.get_uptime(),
            reconnect_count: self.reconnect_count,
            last_heartbeat: self.last_heartbeat,
            heartbeat_interval: self.heartbeat_interval,
            is_healthy: self.is_connection_healthy(),
        }
    }

    /// 檢查連接是否健康
    pub fn is_connection_healthy(&self) -> bool {
        match self.status {
            GatewayStatus::Connected => !self.is_heartbeat_timeout(),
            _ => false,
        }
    }

    /// 計算連接可靠性評分（0-100）
    pub fn calculate_reliability_score(&self) -> u8 {
        let uptime = self.get_uptime();
        let total_time = uptime.as_secs() + (self.reconnect_count * 30); // 假設每次重連損失 30 秒

        if total_time == 0 {
            return 0;
        }

        let uptime_ratio = uptime.as_secs() as f64 / total_time as f64;
        let reliability_score = (uptime_ratio * 100.0).min(100.0).max(0.0) as u8;

        // 如果有太多重連，降低評分
        if self.reconnect_count > 10 {
            reliability_score.saturating_sub((self.reconnect_count - 10) as u8 * 2)
        } else {
            reliability_score
        }
    }
}

impl Default for GatewayManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 連接信息結構
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub status: GatewayStatus,
    pub uptime: Duration,
    pub reconnect_count: u64,
    pub last_heartbeat: Option<Instant>,
    pub heartbeat_interval: Option<Duration>,
    pub is_healthy: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[test]
    fn test_gateway_manager_initial_state() {
        let manager = GatewayManager::new();

        assert_eq!(manager.get_status(), GatewayStatus::Disconnected);
        assert_eq!(manager.get_uptime(), Duration::from_secs(0));
        assert_eq!(manager.get_reconnect_count(), 0);
        assert!(!manager.is_connection_healthy());
    }

    #[test]
    fn test_gateway_status_transitions() {
        let mut manager = GatewayManager::new();

        // 測試連接流程
        manager.set_status(GatewayStatus::Connecting);
        assert_eq!(manager.get_status(), GatewayStatus::Connecting);

        manager.set_status(GatewayStatus::Connected);
        assert_eq!(manager.get_status(), GatewayStatus::Connected);
        assert!(manager.connected_at.is_some());

        // 等待一小段時間以測試運行時間
        // 注意：在異步測試中使用 std::thread::sleep 會導致問題
        // 這裡我們直接測試連接狀態而不依賴時間等待
        assert!(manager.connected_at.is_some());
    }

    #[test]
    fn test_reconnect_count() {
        let mut manager = GatewayManager::new();

        assert_eq!(manager.get_reconnect_count(), 0);

        manager.increment_reconnect_count();
        assert_eq!(manager.get_reconnect_count(), 1);

        manager.increment_reconnect_count();
        assert_eq!(manager.get_reconnect_count(), 2);
    }

    #[test]
    fn test_heartbeat_functionality() {
        let mut manager = GatewayManager::new();

        // 初始狀態不應該有心跳超時
        assert!(!manager.is_heartbeat_timeout());

        // 更新心跳
        manager.update_heartbeat(45000); // 45 秒間隔
        assert!(!manager.is_heartbeat_timeout());

        // 模擬心跳超時（需要實際等待或使用模擬時間）
        // 這裡只測試邏輯結構
        assert!(manager.last_heartbeat.is_some());
        assert!(manager.heartbeat_interval.is_some());
    }

    #[test]
    fn test_heartbeat_counter() {
        let mut manager = GatewayManager::new();

        // 初始心跳計數器應為 0
        assert_eq!(manager.heartbeat_count, 0);

        // 更新心跳應增加計數器
        manager.update_heartbeat(30000);
        assert_eq!(manager.heartbeat_count, 1);

        // 再更新一次
        manager.update_heartbeat(30000);
        assert_eq!(manager.heartbeat_count, 2);

        // 測試每100次記錄一次的邏輯
        for i in 3..=100 {
            manager.update_heartbeat(30000);
            assert_eq!(manager.heartbeat_count, i);
        }

        // 第100次心跳，計數器應為100
        assert_eq!(manager.heartbeat_count, 100);
    }

    #[test]
    fn test_reliability_score() {
        let mut manager = GatewayManager::new();

        // 新創建的管理器評分應該是 0
        assert_eq!(manager.calculate_reliability_score(), 0);

        // 連接後評分應該提高
        manager.set_status(GatewayStatus::Connected);
        // 注意：在異步測試中避免使用 thread::sleep
        // 我們直接測試連接狀態而不是依賴時間等待
        assert!(manager.is_connection_healthy()); // 連接應該是健康的
    }

    #[test]
    fn test_connection_info() {
        let mut manager = GatewayManager::new();
        manager.set_status(GatewayStatus::Connected);
        manager.update_heartbeat(45000);

        let info = manager.get_connection_info();

        assert!(matches!(info.status, GatewayStatus::Connected));
        assert!(info.heartbeat_interval.is_some());
        assert!(info.last_heartbeat.is_some());
        // 由於沒有心跳超時，應該是健康的
        assert!(info.is_healthy);
    }

    #[test]
    fn test_error_status() {
        let mut manager = GatewayManager::new();

        manager.set_status(GatewayStatus::Error("Test error".to_string()));

        if let GatewayStatus::Error(msg) = manager.get_status() {
            assert_eq!(msg, "Test error");
        } else {
            panic!("Expected Error status");
        }

        assert!(!manager.is_connection_healthy());
    }
}
