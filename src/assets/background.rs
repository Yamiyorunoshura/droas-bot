use anyhow::Result;
use tracing::{debug, info, warn, error};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;
use chrono::Utc;

/// 支持的圖片格式
#[derive(Debug, Clone, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

impl ImageFormat {
    /// 從檔案擴展名判斷格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "jpg" | "jpeg" => Some(ImageFormat::Jpeg),
            _ => None,
        }
    }

    /// 獲取 MIME 類型
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
        }
    }

    /// 獲取檔案擴展名
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
        }
    }
}

/// 背景資源信息
#[derive(Debug, Clone)]
pub struct BackgroundInfo {
    /// 資源ID
    pub asset_id: String,
    /// 檔案路徑
    pub file_path: PathBuf,
    /// 圖片格式
    pub format: ImageFormat,
    /// 檔案大小（字節）
    pub file_size: u64,
    /// 上傳時間
    pub uploaded_at: chrono::DateTime<Utc>,
}

/// 背景圖片管理器
pub struct BackgroundManager {
    /// 背景圖片存儲目錄
    storage_path: PathBuf,
    /// 最大檔案大小（5MB）
    max_file_size: u64,
}

impl BackgroundManager {
    /// 創建新的背景管理器
    /// 
    /// # Arguments
    /// * `storage_path` - 背景圖片存儲目錄
    pub async fn new<P: AsRef<Path>>(storage_path: P) -> Result<Self> {
        let storage_path = storage_path.as_ref().to_path_buf();
        info!("初始化背景管理器: storage_path={}", storage_path.display());

        // 確保存儲目錄存在
        fs::create_dir_all(&storage_path).await?;

        Ok(Self {
            storage_path,
            max_file_size: 5 * 1024 * 1024, // 5MB
        })
    }

    /// 驗證圖片檔案
    fn validate_image_data(&self, data: &[u8], format: &ImageFormat) -> Result<()> {
        // 檢查檔案大小
        if data.len() as u64 > self.max_file_size {
            return Err(anyhow::anyhow!(
                "檔案太大: {} bytes, 最大允許: {} bytes",
                data.len(),
                self.max_file_size
            ));
        }

        // 基本的魔數檢查
        match format {
            ImageFormat::Png => {
                if data.len() < 8 || &data[0..8] != b"\x89PNG\r\n\x1a\n" {
                    return Err(anyhow::anyhow!("無效的PNG檔案格式"));
                }
            }
            ImageFormat::Jpeg => {
                if data.len() < 2 || &data[0..2] != b"\xFF\xD8" {
                    return Err(anyhow::anyhow!("無效的JPEG檔案格式"));
                }
            }
        }

        debug!("圖片驗證通過: format={:?}, size={} bytes", format, data.len());
        Ok(())
    }

    /// 生成唯一的檔案路徑
    fn generate_file_path(&self, guild_id: &str, format: &ImageFormat) -> PathBuf {
        let asset_id = Uuid::new_v4().to_string();
        let filename = format!("{}_{}.{}", guild_id, asset_id, format.extension());
        self.storage_path.join("guild").join(guild_id).join(filename)
    }

    /// 上傳背景圖片
    /// 
    /// # Arguments
    /// * `guild_id` - Discord Guild ID
    /// * `data` - 圖片二進制數據
    /// * `format` - 圖片格式
    /// 
    /// # Returns
    /// * `Result<BackgroundInfo>` - 成功返回背景信息
    pub async fn upload_background(&self, guild_id: &str, data: &[u8], format: ImageFormat) -> Result<BackgroundInfo> {
        debug!("上傳背景圖片: guild_id={}, size={} bytes", guild_id, data.len());

        // 驗證圖片數據
        self.validate_image_data(data, &format)?;

        // 生成檔案路徑
        let file_path = self.generate_file_path(guild_id, &format);
        let asset_id = file_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // 確保目錄存在
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // 寫入檔案
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(data).await?;
        file.flush().await?;

        let background_info = BackgroundInfo {
            asset_id,
            file_path: file_path.clone(),
            format,
            file_size: data.len() as u64,
            uploaded_at: Utc::now(),
        };

        info!("背景圖片上傳成功: {}", file_path.display());
        Ok(background_info)
    }

    /// 獲取背景圖片數據
    /// 
    /// # Arguments
    /// * `asset_path` - 資源檔案路徑
    /// 
    /// # Returns
    /// * `Result<Vec<u8>>` - 圖片二進制數據
    pub async fn get_background_data<P: AsRef<Path>>(&self, asset_path: P) -> Result<Vec<u8>> {
        let asset_path = asset_path.as_ref();
        debug!("讀取背景圖片: {}", asset_path.display());

        if !asset_path.exists() {
            return Err(anyhow::anyhow!("背景圖片檔案不存在: {}", asset_path.display()));
        }

        let mut file = fs::File::open(asset_path).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        debug!("背景圖片讀取成功: size={} bytes", data.len());
        Ok(data)
    }

    /// 刪除背景圖片
    /// 
    /// # Arguments
    /// * `asset_path` - 資源檔案路徑
    /// 
    /// # Returns
    /// * `Result<bool>` - 是否成功刪除
    pub async fn delete_background<P: AsRef<Path>>(&self, asset_path: P) -> Result<bool> {
        let asset_path = asset_path.as_ref();
        debug!("刪除背景圖片: {}", asset_path.display());

        if !asset_path.exists() {
            warn!("要刪除的背景圖片檔案不存在: {}", asset_path.display());
            return Ok(false);
        }

        fs::remove_file(asset_path).await?;
        info!("背景圖片刪除成功: {}", asset_path.display());
        Ok(true)
    }

    /// 列出指定 Guild 的所有背景圖片
    /// 
    /// # Arguments
    /// * `guild_id` - Discord Guild ID
    /// 
    /// # Returns
    /// * `Result<Vec<PathBuf>>` - 背景圖片檔案路徑列表
    pub async fn list_guild_backgrounds(&self, guild_id: &str) -> Result<Vec<PathBuf>> {
        let guild_dir = self.storage_path.join("guild").join(guild_id);
        debug!("列出Guild背景圖片: {}", guild_dir.display());

        if !guild_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(guild_dir).await?;
        let mut backgrounds = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ImageFormat::from_extension(ext).is_some() {
                        backgrounds.push(path);
                    }
                }
            }
        }

        debug!("找到 {} 個背景圖片", backgrounds.len());
        Ok(backgrounds)
    }

    /// 清理過期或損壞的背景圖片
    pub async fn cleanup(&self) -> Result<()> {
        info!("開始清理背景圖片");

        let mut cleaned_files = 0;
        let mut total_size_freed = 0u64;

        // 遍歷存儲目錄
        let mut entries = fs::read_dir(&self.storage_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().is_dir() && entry.file_name() == "guild" {
                // 遍歷guild子目錄
                let guild_dir = entry.path();
                let mut guild_entries = fs::read_dir(guild_dir).await?;
                
                while let Some(guild_entry) = guild_entries.next_entry().await? {
                    if guild_entry.path().is_dir() {
                        // 檢查guild特定的背景
                        if let Err(e) = self.cleanup_guild_directory(&guild_entry.path(), &mut cleaned_files, &mut total_size_freed).await {
                            error!("清理Guild目錄失敗: {}, error: {}", guild_entry.path().display(), e);
                        }
                    }
                }
            }
        }

        info!("背景圖片清理完成: 清理了 {} 個檔案，釋放了 {} bytes", cleaned_files, total_size_freed);
        Ok(())
    }

    /// 清理特定 Guild 目錄
    async fn cleanup_guild_directory(&self, guild_dir: &Path, cleaned_files: &mut u32, total_size_freed: &mut u64) -> Result<()> {
        let mut entries = fs::read_dir(guild_dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                // 檢查是否為支持的圖片格式
                let should_remove = if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    ImageFormat::from_extension(ext).is_none()
                } else {
                    true // 沒有擴展名的檔案
                };

                if should_remove {
                    if let Ok(metadata) = fs::metadata(&path).await {
                        let size = metadata.len();
                        if fs::remove_file(&path).await.is_ok() {
                            *cleaned_files += 1;
                            *total_size_freed += size;
                            debug!("清理無效檔案: {}", path.display());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 獲取所有背景圖片的總大小
    pub async fn get_total_size(&self) -> Result<u64> {
        let mut total_size = 0u64;
        
        if let Ok(mut entries) = fs::read_dir(&self.storage_path).await {
            while let Some(entry) = entries.next_entry().await? {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    } else if metadata.is_dir() {
                        total_size += self.get_directory_size(&entry.path()).await?;
                    }
                }
            }
        }

        Ok(total_size)
    }

    /// 遞歸計算目錄大小
    fn get_directory_size<'a>(&'a self, dir_path: &'a Path) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<u64>> + 'a>> {
        Box::pin(async move {
            let mut size = 0u64;
            
            if let Ok(mut entries) = fs::read_dir(dir_path).await {
                while let Some(entry) = entries.next_entry().await? {
                    if let Ok(metadata) = entry.metadata().await {
                        if metadata.is_file() {
                            size += metadata.len();
                        } else if metadata.is_dir() {
                            size += self.get_directory_size(&entry.path()).await?;
                        }
                    }
                }
            }

            Ok(size)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // 創建測試用的PNG圖片數據
    fn create_test_png_data() -> Vec<u8> {
        // 最小有效的PNG檔案（1x1像素透明圖片）
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
            0x89, 0x00, 0x00, 0x00, 0x0B, 0x49, 0x44, 0x41,
            0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
            0x42, 0x60, 0x82
        ]
    }

    // 創建測試用的JPEG圖片數據
    fn create_test_jpeg_data() -> Vec<u8> {
        // 最小有效的JPEG檔案頭部
        vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46,
            0x49, 0x46, 0x00, 0x01, 0x01, 0x01, 0x00, 0x48,
            0x00, 0x48, 0x00, 0x00, 0xFF, 0xD9
        ]
    }

    #[tokio::test]
    async fn test_background_manager_creation() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = BackgroundManager::new(temp_dir.path()).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_image_format_validation() {
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("jpeg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("gif"), None);
    }

    #[tokio::test]
    async fn test_upload_and_get_background() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = BackgroundManager::new(temp_dir.path()).await.unwrap();
        
        let png_data = create_test_png_data();
        let guild_id = "test_guild_123";
        
        // 測試上傳
        let background_info = manager.upload_background(guild_id, &png_data, ImageFormat::Png).await;
        assert!(background_info.is_ok());
        let background_info = background_info.unwrap();
        
        assert_eq!(background_info.format, ImageFormat::Png);
        assert_eq!(background_info.file_size, png_data.len() as u64);
        
        // 測試讀取
        let retrieved_data = manager.get_background_data(&background_info.file_path).await;
        assert!(retrieved_data.is_ok());
        assert_eq!(retrieved_data.unwrap(), png_data);
    }

    #[tokio::test]
    async fn test_invalid_image_validation() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = BackgroundManager::new(temp_dir.path()).await.unwrap();
        
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03]; // 無效數據
        
        let result = manager.upload_background("test_guild", &invalid_data, ImageFormat::Png).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_file_size_validation() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = BackgroundManager::new(temp_dir.path()).await.unwrap();
        
        // 創建過大的檔案（6MB）
        let large_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
        let mut large_data = large_data;
        large_data.extend(vec![0; 6 * 1024 * 1024]); // 6MB of data
        
        let result = manager.upload_background("test_guild", &large_data, ImageFormat::Png).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_guild_backgrounds() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = BackgroundManager::new(temp_dir.path()).await.unwrap();
        
        let png_data = create_test_png_data();
        let jpeg_data = create_test_jpeg_data();
        let guild_id = "test_guild_456";
        
        // 上傳多個背景
        manager.upload_background(guild_id, &png_data, ImageFormat::Png).await.unwrap();
        manager.upload_background(guild_id, &jpeg_data, ImageFormat::Jpeg).await.unwrap();
        
        // 列出背景
        let backgrounds = manager.list_guild_backgrounds(guild_id).await.unwrap();
        assert_eq!(backgrounds.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_background() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = BackgroundManager::new(temp_dir.path()).await.unwrap();
        
        let png_data = create_test_png_data();
        let guild_id = "test_guild_789";
        
        // 上傳背景
        let background_info = manager.upload_background(guild_id, &png_data, ImageFormat::Png).await.unwrap();
        
        // 確認檔案存在
        assert!(background_info.file_path.exists());
        
        // 刪除背景
        let deleted = manager.delete_background(&background_info.file_path).await.unwrap();
        assert!(deleted);
        
        // 確認檔案不存在
        assert!(!background_info.file_path.exists());
    }

    #[tokio::test]
    async fn test_get_total_size() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = BackgroundManager::new(temp_dir.path()).await.unwrap();
        
        // 初始大小應該是0
        let initial_size = manager.get_total_size().await.unwrap();
        assert_eq!(initial_size, 0);
        
        // 上傳一個檔案
        let png_data = create_test_png_data();
        manager.upload_background("test_guild", &png_data, ImageFormat::Png).await.unwrap();
        
        // 大小應該增加
        let new_size = manager.get_total_size().await.unwrap();
        assert!(new_size > initial_size);
        assert_eq!(new_size, png_data.len() as u64);
    }
}