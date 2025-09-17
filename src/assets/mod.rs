pub mod background;
pub mod cache;
pub mod fonts;

use anyhow::Result;
use tracing::info;
use std::path::Path;
use tokio::fs;

/// 資源管理器，統一管理背景圖片、字體等資源
pub struct AssetManager {
    /// 資源根目錄
    root_path: String,
    /// 背景圖片管理器
    background: background::BackgroundManager,
    /// 緩存管理器
    cache: cache::CacheManager,
    /// 字體管理器
    fonts: fonts::FontManager,
}

impl AssetManager {
    /// 創建新的資源管理器
    /// 
    /// # Arguments
    /// * `root_path` - 資源根目錄路徑
    /// 
    /// # Returns
    /// * `Result<Self>` - 成功返回 AssetManager 實例，失敗返回錯誤
    pub async fn new<P: AsRef<Path>>(root_path: P) -> Result<Self> {
        let root_path = root_path.as_ref().to_string_lossy().to_string();
        info!("初始化資源管理器: root_path={}", root_path);

        // 確保根目錄存在
        fs::create_dir_all(&root_path).await?;

        // 創建子目錄
        let backgrounds_path = Path::new(&root_path).join("backgrounds");
        let fonts_path = Path::new(&root_path).join("fonts");
        
        fs::create_dir_all(&backgrounds_path).await?;
        fs::create_dir_all(&fonts_path).await?;

        // 初始化子管理器
        let background = background::BackgroundManager::new(&backgrounds_path).await?;
        let cache = cache::CacheManager::new(1000, 3600).await?; // 1000 entries, 1 hour TTL
        let fonts = fonts::FontManager::new(&fonts_path).await?;

        info!("資源管理器初始化成功");
        
        Ok(Self {
            root_path,
            background,
            cache,
            fonts,
        })
    }

    /// 獲取背景圖片管理器的引用
    pub fn backgrounds(&self) -> &background::BackgroundManager {
        &self.background
    }

    /// 獲取緩存管理器的引用
    pub fn cache(&self) -> &cache::CacheManager {
        &self.cache
    }

    /// 獲取字體管理器的引用
    pub fn fonts(&self) -> &fonts::FontManager {
        &self.fonts
    }

    /// 清理過期和不需要的資源
    pub async fn cleanup(&self) -> Result<()> {
        info!("開始清理資源");

        // 清理背景圖片
        self.background.cleanup().await?;

        // 清理緩存
        self.cache.cleanup().await?;

        // 清理字體不需要，它們是靜態資源

        info!("資源清理完成");
        Ok(())
    }

    /// 獲取資源總大小統計
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        let background_size = self.background.get_total_size().await?;
        let font_size = self.fonts.get_total_size().await?;
        let cache_size = self.cache.get_memory_usage().await;

        Ok(StorageStats {
            background_assets_size: background_size,
            font_assets_size: font_size,
            cache_memory_size: cache_size,
            total_disk_size: background_size + font_size,
        })
    }
}

/// 存儲統計信息
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// 背景資源大小（字節）
    pub background_assets_size: u64,
    /// 字體資源大小（字節）
    pub font_assets_size: u64,
    /// 緩存記憶體使用（字節）
    pub cache_memory_size: u64,
    /// 總磁碟使用大小
    pub total_disk_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_asset_manager_creation() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = AssetManager::new(temp_dir.path()).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_get_storage_stats() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = AssetManager::new(temp_dir.path()).await.unwrap();
        
        let stats = manager.get_storage_stats().await;
        assert!(stats.is_ok());
        let stats = stats.unwrap();
        
        // 初始狀態背景應該是空的，但字體目錄可能有README檔案
        assert_eq!(stats.background_assets_size, 0);
        // 字體大小可能不為0（因為有README.md檔案）
        assert!(stats.font_assets_size >= 0);
        assert_eq!(stats.total_disk_size, stats.background_assets_size + stats.font_assets_size);
    }

    #[tokio::test]
    async fn test_cleanup() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = AssetManager::new(temp_dir.path()).await.unwrap();
        
        let result = manager.cleanup().await;
        assert!(result.is_ok());
    }
}