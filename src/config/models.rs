//! 公會配置資料模型
//! 
//! 此模組定義了公會配置管理系統的核心資料結構，包括：
//! - GuildConfig: 公會特定的配置設定
//! - BackgroundAsset: 背景圖片資源管理

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 公會配置結構
/// 
/// 儲存每個 Discord 公會的個別配置設定，包括歡迎頻道和背景圖片引用。
/// 設計支持未來擴展更多配置選項。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct GuildConfig {
    /// 公會 ID (Discord Snowflake)
    pub guild_id: i64,
    /// 歡迎訊息發送的頻道 ID (可選)
    pub welcome_channel_id: Option<i64>,
    /// 背景圖片資源引用 (可選)
    pub background_ref: Option<String>,
    /// 配置最後更新時間
    pub updated_at: DateTime<Utc>,
    /// 配置創建時間
    pub created_at: DateTime<Utc>,
}

/// 背景圖片資源結構
/// 
/// 管理上傳的背景圖片資源，包括文件路径、媒體類型等元數據。
/// 與 GuildConfig 建立一對一關聯關係。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct BackgroundAsset {
    /// 資源唯一標識符
    pub id: String,
    /// 文件存儲路径
    pub file_path: String,
    /// 媒體類型 (例如: image/png, image/jpeg)
    pub media_type: String,
    /// 文件大小 (bytes)
    pub file_size: i64,
    /// 資源創建時間
    pub created_at: DateTime<Utc>,
}

impl GuildConfig {
    /// 創建新的公會配置
    /// 
    /// # Arguments
    /// 
    /// * `guild_id` - Discord 公會 ID
    /// * `welcome_channel_id` - 歡迎頻道 ID (可選)
    /// * `background_ref` - 背景圖片引用 (可選)
    pub fn new(
        guild_id: i64,
        welcome_channel_id: Option<i64>,
        background_ref: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            guild_id,
            welcome_channel_id,
            background_ref,
            updated_at: now,
            created_at: now,
        }
    }
    
    /// 更新歡迎頻道 ID
    pub fn update_welcome_channel(&mut self, channel_id: Option<i64>) {
        self.welcome_channel_id = channel_id;
        self.updated_at = Utc::now();
    }
    
    /// 更新背景圖片引用
    pub fn update_background(&mut self, background_ref: Option<String>) {
        self.background_ref = background_ref;
        self.updated_at = Utc::now();
    }
    
    /// 檢查是否有有效的配置設定
    pub fn has_valid_config(&self) -> bool {
        self.welcome_channel_id.is_some() || self.background_ref.is_some()
    }
}

impl BackgroundAsset {
    /// 創建新的背景圖片資源
    /// 
    /// # Arguments
    /// 
    /// * `id` - 資源唯一標識符
    /// * `file_path` - 文件存儲路径
    /// * `media_type` - 媒體類型
    /// * `file_size` - 文件大小
    pub fn new(id: String, file_path: String, media_type: String, file_size: i64) -> Self {
        Self {
            id,
            file_path,
            media_type,
            file_size,
            created_at: Utc::now(),
        }
    }
    
    /// 檢查媒體類型是否為支持的圖片格式
    pub fn is_supported_image(&self) -> bool {
        matches!(self.media_type.as_str(), "image/png" | "image/jpeg" | "image/jpg")
    }
    
    /// 檢查文件大小是否在允許範圍內 (5MB)
    pub fn is_valid_size(&self) -> bool {
        self.file_size > 0 && self.file_size <= 5 * 1024 * 1024 // 5MB in bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_guild_config_creation() {
        let guild_id = 123456789i64;
        let welcome_channel_id = Some(987654321i64);
        let background_ref = Some("bg_001".to_string());
        
        let config = GuildConfig::new(guild_id, welcome_channel_id, background_ref);
        
        assert_eq!(config.guild_id, guild_id);
        assert_eq!(config.welcome_channel_id, welcome_channel_id);
        assert_eq!(config.background_ref, Some("bg_001".to_string()));
        assert!(config.has_valid_config());
        
        // 檢查時間戳是否合理 (應該是最近創建的)
        let now = Utc::now();
        let time_diff = now.signed_duration_since(config.created_at);
        assert!(time_diff.num_seconds() < 1);
    }
    
    #[test]
    fn test_guild_config_updates() {
        let mut config = GuildConfig::new(123456789i64, None, None);
        let original_created_at = config.created_at;
        
        // 更新歡迎頻道
        config.update_welcome_channel(Some(111111111i64));
        assert_eq!(config.welcome_channel_id, Some(111111111i64));
        assert!(config.updated_at > original_created_at);
        
        // 更新背景圖片
        let first_update_time = config.updated_at;
        config.update_background(Some("new_bg".to_string()));
        assert_eq!(config.background_ref, Some("new_bg".to_string()));
        assert!(config.updated_at > first_update_time);
        
        assert!(config.has_valid_config());
    }
    
    #[test]
    fn test_guild_config_empty() {
        let config = GuildConfig::new(123456789i64, None, None);
        assert!(!config.has_valid_config());
    }
    
    #[test]
    fn test_background_asset_creation() {
        let asset = BackgroundAsset::new(
            "asset_001".to_string(),
            "/path/to/image.png".to_string(),
            "image/png".to_string(),
            1024000, // 1MB
        );
        
        assert_eq!(asset.id, "asset_001");
        assert_eq!(asset.file_path, "/path/to/image.png");
        assert_eq!(asset.media_type, "image/png");
        assert_eq!(asset.file_size, 1024000);
        assert!(asset.is_supported_image());
        assert!(asset.is_valid_size());
    }
    
    #[test]
    fn test_background_asset_validation() {
        // 測試支持的圖片格式
        let png_asset = BackgroundAsset::new("test".to_string(), "test.png".to_string(), "image/png".to_string(), 1000);
        assert!(png_asset.is_supported_image());
        
        let jpeg_asset = BackgroundAsset::new("test".to_string(), "test.jpg".to_string(), "image/jpeg".to_string(), 1000);
        assert!(jpeg_asset.is_supported_image());
        
        let jpg_asset = BackgroundAsset::new("test".to_string(), "test.jpg".to_string(), "image/jpg".to_string(), 1000);
        assert!(jpg_asset.is_supported_image());
        
        // 測試不支持的格式
        let gif_asset = BackgroundAsset::new("test".to_string(), "test.gif".to_string(), "image/gif".to_string(), 1000);
        assert!(!gif_asset.is_supported_image());
        
        // 測試文件大小限制
        let valid_size = BackgroundAsset::new("test".to_string(), "test.png".to_string(), "image/png".to_string(), 1024 * 1024); // 1MB
        assert!(valid_size.is_valid_size());
        
        let too_large = BackgroundAsset::new("test".to_string(), "test.png".to_string(), "image/png".to_string(), 6 * 1024 * 1024); // 6MB
        assert!(!too_large.is_valid_size());
        
        let zero_size = BackgroundAsset::new("test".to_string(), "test.png".to_string(), "image/png".to_string(), 0);
        assert!(!zero_size.is_valid_size());
    }
}