use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{RwLock, mpsc};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tracing::{info, warn, error, debug};

/// 檔案監控器，監控配置檔案變更
pub struct FileWatcher {
    watchers: Arc<RwLock<HashMap<WatchHandle, WatcherState>>>,
    next_handle: Arc<RwLock<u64>>,
}

/// 監控處理標識
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct WatchHandle(u64);

/// 監控器狀態
struct WatcherState {
    _watcher: Box<dyn Watcher + Send + Sync>,
    path: PathBuf,
    sender: mpsc::Sender<PathBuf>,
}

impl FileWatcher {
    /// 創建新的檔案監控器
    pub fn new() -> Self {
        Self {
            watchers: Arc::new(RwLock::new(HashMap::new())),
            next_handle: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 開始監控指定檔案
    /// 
    /// # Arguments
    /// * `path` - 要監控的檔案路徑
    /// * `sender` - 檔案變更事件發送通道
    /// 
    /// # Returns
    /// * `WatchHandle` - 用於停止監控的處理標識
    pub async fn watch(
        &self,
        path: &Path,
        sender: mpsc::Sender<PathBuf>,
    ) -> Result<WatchHandle, WatcherError> {
        info!("Starting file watcher for: {:?}", path);
        
        // 檢查檔案是否存在
        if !path.exists() {
            return Err(WatcherError::FileNotFound(path.to_string_lossy().to_string()));
        }
        
        let path_buf = path.to_path_buf();
        let sender_clone = sender.clone();
        let watched_path = path_buf.clone();
        
        // 創建notify watcher
        let mut watcher = notify::recommended_watcher(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // 只處理修改和創建事件
                        match event.kind {
                            EventKind::Modify(_) | EventKind::Create(_) => {
                                debug!("File change detected: {:?}", event.paths);
                                for event_path in &event.paths {
                                    if event_path == &watched_path {
                                        let sender = sender_clone.clone();
                                        let path = watched_path.clone();
                                        tokio::spawn(async move {
                                            if let Err(e) = sender.send(path).await {
                                                warn!("Failed to send file change event: {}", e);
                                            }
                                        });
                                    }
                                }
                            }
                            _ => {
                                // 忽略其他事件類型
                            }
                        }
                    }
                    Err(e) => {
                        error!("Watch error: {:?}", e);
                    }
                }
            }
        ).map_err(|e| WatcherError::NotifyError(e.to_string()))?;
        
        // 開始監控
        watcher.watch(&path_buf, RecursiveMode::NonRecursive)
            .map_err(|e| WatcherError::NotifyError(e.to_string()))?;
        
        // 生成handle並儲存watcher
        let mut handle_guard = self.next_handle.write().await;
        let handle = WatchHandle(*handle_guard);
        *handle_guard += 1;
        
        let state = WatcherState {
            _watcher: Box::new(watcher),
            path: path_buf,
            sender,
        };
        
        let mut watchers = self.watchers.write().await;
        watchers.insert(handle, state);
        
        info!("File watcher started with handle: {:?}", handle);
        Ok(handle)
    }
    
    /// 停止監控
    pub async fn stop_watch(&self, handle: WatchHandle) -> Result<(), WatcherError> {
        let mut watchers = self.watchers.write().await;
        
        if let Some(state) = watchers.remove(&handle) {
            info!("Stopped watching: {:?}", state.path);
            Ok(())
        } else {
            Err(WatcherError::InvalidHandle)
        }
    }
    
    /// 停止所有監控
    pub async fn stop_all(&self) {
        let mut watchers = self.watchers.write().await;
        let count = watchers.len();
        watchers.clear();
        info!("Stopped all {} file watchers", count);
    }
    
    /// 獲取當前監控的檔案數量
    pub async fn watch_count(&self) -> usize {
        self.watchers.read().await.len()
    }
    
    /// 檢查某個檔案是否正在被監控
    pub async fn is_watching(&self, path: &Path) -> bool {
        let watchers = self.watchers.read().await;
        watchers.values().any(|state| state.path == path)
    }
}

/// 檔案監控錯誤類型
#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Notify error: {0}")]
    NotifyError(String),
    
    #[error("Invalid watch handle")]
    InvalidHandle,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use tokio::time::{timeout, Duration};
    
    #[tokio::test]
    async fn test_file_watcher_basic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        fs::write(&file_path, "initial").unwrap();
        
        let watcher = FileWatcher::new();
        let (tx, mut rx) = mpsc::channel(10);
        
        let handle = watcher.watch(&file_path, tx).await.unwrap();
        assert_eq!(watcher.watch_count().await, 1);
        assert!(watcher.is_watching(&file_path).await);
        
        // Trigger a change
        tokio::time::sleep(Duration::from_millis(100)).await;
        fs::write(&file_path, "modified").unwrap();
        
        // Wait for event
        let result = timeout(Duration::from_secs(5), rx.recv()).await;
        assert!(result.is_ok());
        
        // Stop watching
        watcher.stop_watch(handle).await.unwrap();
        assert_eq!(watcher.watch_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_file_watcher_non_existent_file() {
        let watcher = FileWatcher::new();
        let (tx, _rx) = mpsc::channel(10);
        
        let result = watcher.watch(Path::new("/non/existent/file.yaml"), tx).await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(matches!(e, WatcherError::FileNotFound(_)));
        }
    }
}
