use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, error};

/// 配置相關事件
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConfigEvent {
    /// 配置已更新
    ConfigUpdated {
        path: String,
        success: bool,
    },
    /// 配置驗證失敗
    ConfigValidationFailed {
        path: String,
        error: String,
    },
    /// 配置已回滾
    ConfigRolledBack {
        path: String,
        reason: String,
    },
    /// 檔案監控錯誤
    WatcherError {
        path: String,
        error: String,
    },
    /// 熱重載已啟動
    HotReloadStarted {
        path: String,
    },
    /// 熱重載已停止
    HotReloadStopped {
        path: String,
    },
}

/// 事件總線，支援多訂閱者的事件分發
pub struct EventBus<T: Clone + Send + Sync + 'static> {
    sender: broadcast::Sender<T>,
    receiver_count: Arc<RwLock<usize>>,
}

impl<T: Clone + Send + Sync + 'static> EventBus<T> {
    /// 創建新的事件總線
    /// 
    /// # Arguments
    /// * `capacity` - 事件佇列容量
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            receiver_count: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 發布事件到所有訂閱者
    pub async fn publish(&self, event: T) -> Result<(), EventBusError> {
        match self.sender.send(event) {
            Ok(count) => {
                debug!("Event published to {} subscribers", count);
                Ok(())
            }
            Err(_) => {
                // 沒有接收者時不算錯誤，只是警告
                warn!("No subscribers available for event");
                Ok(())
            }
        }
    }
    
    /// 訂閱事件
    pub async fn subscribe(&self) -> broadcast::Receiver<T> {
        let mut count = self.receiver_count.write().await;
        *count += 1;
        debug!("New subscriber added, total: {}", *count);
        self.sender.subscribe()
    }
    
    /// 獲取當前訂閱者數量
    pub async fn subscriber_count(&self) -> usize {
        let receiver_count = *self.receiver_count.read().await;
        // 實際訂閱者數量需要從sender獲取
        self.sender.receiver_count().max(receiver_count)
    }
    
    /// 檢查是否有訂閱者
    pub fn has_subscribers(&self) -> bool {
        self.sender.receiver_count() > 0
    }
}

/// 事件總線錯誤類型
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Failed to send event: {0}")]
    SendError(String),
    
    #[error("Channel closed")]
    ChannelClosed,
}

impl Default for EventBus<ConfigEvent> {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::<String>::new(10);
        let mut subscriber = bus.subscribe().await;
        
        bus.publish("test_event".to_string()).await.unwrap();
        
        let received = timeout(Duration::from_millis(100), subscriber.recv()).await;
        assert!(received.is_ok());
        assert_eq!(received.unwrap().unwrap(), "test_event");
    }
    
    #[tokio::test]
    async fn test_config_event_serialization() {
        let event = ConfigEvent::ConfigUpdated {
            path: "/test/path.yaml".to_string(),
            success: true,
        };
        
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ConfigEvent = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(event, deserialized);
    }
}
