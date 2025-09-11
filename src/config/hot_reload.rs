use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use tokio::task::JoinHandle;
use tracing::{info, warn, error, debug};

use crate::config::{
    ConfigService, FileWatcher, EventBus, ConfigEvent, 
    WatchHandle, ConfigError
};

/// 熱重載服務，整合檔案監控和配置重載
pub struct HotReloadService {
    file_watcher: Arc<FileWatcher>,
    event_bus: Arc<EventBus<ConfigEvent>>,
    active_watches: Arc<RwLock<Vec<(PathBuf, WatchHandle, JoinHandle<()>)>>>,
    is_running: Arc<RwLock<bool>>,
}

impl HotReloadService {
    /// 創建新的熱重載服務
    pub fn new() -> Self {
        Self {
            file_watcher: Arc::new(FileWatcher::new()),
            event_bus: Arc::new(EventBus::new(100)),
            active_watches: Arc::new(RwLock::new(Vec::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }
    
    /// 啟動熱重載服務
    /// 
    /// # Arguments
    /// * `config_path` - 配置檔案路徑
    /// * `config_service` - 配置服務實例
    pub async fn start(
        &self,
        config_path: &Path,
        config_service: Arc<ConfigService>,
    ) -> Result<(), HotReloadError> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(HotReloadError::AlreadyRunning);
        }
        
        info!("Starting hot reload service for: {:?}", config_path);
        
        // 創建檔案變更通道
        let (tx, mut rx) = mpsc::channel::<PathBuf>(10);
        
        // 開始監控檔案
        let watch_handle = self.file_watcher
            .watch(config_path, tx)
            .await
            .map_err(|e| HotReloadError::WatcherError(e.to_string()))?;
        
        // 發送啟動事件
        self.event_bus.publish(ConfigEvent::HotReloadStarted {
            path: config_path.to_string_lossy().to_string(),
        }).await.ok();
        
        // 啟動重載任務
        let config_path = config_path.to_path_buf();
        let config_service = Arc::clone(&config_service);
        let event_bus = Arc::clone(&self.event_bus);
        
        let reload_task = tokio::spawn(async move {
            while let Some(changed_path) = rx.recv().await {
                info!("Config file changed: {:?}", changed_path);
                
                // 執行熱重載
                match Self::perform_reload(&changed_path, &config_service, &event_bus).await {
                    Ok(_) => {
                        info!("Configuration reloaded successfully");
                    }
                    Err(e) => {
                        error!("Failed to reload configuration: {}", e);
                    }
                }
            }
            debug!("Reload task terminated");
        });
        
        // 儲存監控狀態
        let mut watches = self.active_watches.write().await;
        watches.push((config_path.clone(), watch_handle, reload_task));
        
        *is_running = true;
        info!("Hot reload service started");
        
        Ok(())
    }
    
    /// 執行配置重載
    async fn perform_reload(
        config_path: &Path,
        config_service: &Arc<ConfigService>,
        event_bus: &Arc<EventBus<ConfigEvent>>,
    ) -> Result<(), HotReloadError> {
        debug!("Performing configuration reload");
        
        // 保存當前配置作為備份
        let backup_config = config_service.get_config().await.ok();
        
        // 嘗試重載配置
        match config_service.reload_config(config_path).await {
            Ok(_) => {
                // 發送成功事件
                event_bus.publish(ConfigEvent::ConfigUpdated {
                    path: config_path.to_string_lossy().to_string(),
                    success: true,
                }).await.ok();
                
                Ok(())
            }
            Err(e) => {
                error!("Configuration reload failed: {}", e);
                
                // 發送失敗事件
                event_bus.publish(ConfigEvent::ConfigValidationFailed {
                    path: config_path.to_string_lossy().to_string(),
                    error: e.to_string(),
                }).await.ok();
                
                // 如果有備份，嘗試回滾
                if let Some(backup) = backup_config {
                    if config_service.set_config(backup).await.is_ok() {
                        event_bus.publish(ConfigEvent::ConfigRolledBack {
                            path: config_path.to_string_lossy().to_string(),
                            reason: format!("Validation failed: {}", e),
                        }).await.ok();
                        
                        warn!("Configuration rolled back due to validation failure");
                    }
                }
                
                Err(HotReloadError::ReloadFailed(e.to_string()))
            }
        }
    }
    
    /// 停止熱重載服務
    pub async fn stop(&self) -> Result<(), HotReloadError> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(());
        }
        
        info!("Stopping hot reload service");
        
        let mut watches = self.active_watches.write().await;
        
        // 停止所有監控
        for (path, handle, task) in watches.drain(..) {
            // 停止檔案監控
            if let Err(e) = self.file_watcher.stop_watch(handle).await {
                warn!("Failed to stop file watcher: {}", e);
            }
            
            // 取消重載任務
            task.abort();
            
            // 發送停止事件
            self.event_bus.publish(ConfigEvent::HotReloadStopped {
                path: path.to_string_lossy().to_string(),
            }).await.ok();
        }
        
        *is_running = false;
        info!("Hot reload service stopped");
        
        Ok(())
    }
    
    /// 訂閱配置事件
    pub async fn subscribe_events(&self) -> broadcast::Receiver<ConfigEvent> {
        self.event_bus.subscribe().await
    }
    
    /// 檢查服務是否正在運行
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
    
    /// 獲取當前監控的配置檔案數量
    pub async fn watch_count(&self) -> usize {
        self.active_watches.read().await.len()
    }
}

/// 熱重載錯誤類型
#[derive(Debug, thiserror::Error)]
pub enum HotReloadError {
    #[error("Hot reload service is already running")]
    AlreadyRunning,
    
    #[error("Watcher error: {0}")]
    WatcherError(String),
    
    #[error("Reload failed: {0}")]
    ReloadFailed(String),
    
    #[error("Config error: {0}")]
    ConfigError(#[from] ConfigError),
}

impl Default for HotReloadService {
    fn default() -> Self {
        Self::new()
    }
}

// Remove Drop implementation to avoid stack overflow
// Cleanup should be handled explicitly by calling stop()

// 為了 Drop trait 實作 Clone
impl Clone for HotReloadService {
    fn clone(&self) -> Self {
        Self {
            file_watcher: Arc::clone(&self.file_watcher),
            event_bus: Arc::clone(&self.event_bus),
            active_watches: Arc::clone(&self.active_watches),
            is_running: Arc::clone(&self.is_running),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_hot_reload_service_lifecycle() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        
        let config = r#"
bot_config:
  discord_token: "test_token"
  llm_config:
    base_url: "http://localhost:8080"
    api_key: "test_key"
    model: "gpt-4"
    max_tokens: 1000
  system_prompt: "Test"
  protection_level: "low"
  enabled: true
"#;
        fs::write(&config_path, config).unwrap();
        
        let service = HotReloadService::new();
        let config_service = Arc::new(ConfigService::new());
        
        // Load initial config
        config_service.load_config(&config_path).await.unwrap();
        
        // Start service
        assert!(!service.is_running().await);
        service.start(&config_path, config_service).await.unwrap();
        assert!(service.is_running().await);
        assert_eq!(service.watch_count().await, 1);
        
        // Stop service
        service.stop().await.unwrap();
        assert!(!service.is_running().await);
        assert_eq!(service.watch_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_hot_reload_already_running() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");
        fs::write(&config_path, "content").unwrap();
        
        let service = HotReloadService::new();
        let config_service = Arc::new(ConfigService::new());
        
        // Start service
        service.start(&config_path, config_service.clone()).await.unwrap();
        
        // Try to start again
        let result = service.start(&config_path, config_service).await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(matches!(e, HotReloadError::AlreadyRunning));
        }
        
        service.stop().await.unwrap();
    }
}
