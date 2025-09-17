//! 命令處理常數配置
//! 
//! 統一管理所有命令處理相關的常數、限制和配置值。

use std::time::Duration;

/// 圖片處理相關常數
pub mod image {
    /// 支援的圖片格式
    pub const SUPPORTED_FORMATS: &[&str] = &["PNG", "JPEG"];
    
    /// PNG文件頭魔術數字
    pub const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    
    /// JPEG文件頭魔術數字
    pub const JPEG_MAGIC: &[u8] = &[0xFF, 0xD8, 0xFF];
    
    /// GIF文件頭（用於拒絕）
    pub const GIF_MAGIC_87A: &[u8] = b"GIF87a";
    pub const GIF_MAGIC_89A: &[u8] = b"GIF89a";
    
    /// WebP文件頭（用於拒絕）
    pub const WEBP_MAGIC: &[u8] = b"WEBP";
    
    /// 最大圖片文件大小（5MB）
    pub const MAX_IMAGE_SIZE: usize = 5 * 1024 * 1024;
    
    /// 預覽圖片尺寸
    pub const PREVIEW_WIDTH: u32 = 1024;
    pub const PREVIEW_HEIGHT: u32 = 512;
    
    /// 支援的MIME類型
    pub const SUPPORTED_MIME_TYPES: &[&str] = &[
        "image/png",
        "image/jpeg",
        "image/jpg",
    ];
}

/// HTTP請求相關常數
pub mod http {
    use super::Duration;
    
    /// 默認請求超時時間
    pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
    
    /// 圖片下載超時時間
    pub const IMAGE_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(15);
    
    /// 頭像下載超時時間
    pub const AVATAR_DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(10);
    
    /// 最大重試次數
    pub const MAX_RETRIES: u32 = 3;
    
    /// 重試基礎延遲（毫秒）
    pub const RETRY_BASE_DELAY_MS: u64 = 1000;
    
    /// 最大重試延遲（毫秒）
    pub const MAX_RETRY_DELAY_MS: u64 = 10000;
    
    /// 用戶代理字符串
    pub const USER_AGENT: &str = "DROAS-Bot/0.1.0";
    
    /// 最大下載檔案大小（10MB）
    pub const MAX_DOWNLOAD_SIZE: usize = 10 * 1024 * 1024;
}

/// 性能相關常數
pub mod performance {
    use super::Duration;
    
    /// 命令處理最大超時時間
    pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);
    
    /// 預覽生成性能目標（P95）
    pub const PREVIEW_GENERATION_TARGET: Duration = Duration::from_secs(3);
    
    /// 檔案處理性能目標
    pub const FILE_PROCESSING_TARGET: Duration = Duration::from_secs(5);
    
    /// 框架處理性能目標
    pub const FRAMEWORK_PROCESSING_TARGET: Duration = Duration::from_millis(100);
}

/// 安全相關常數
pub mod security {
    /// 禁止的主機名稱
    pub const FORBIDDEN_HOSTS: &[&str] = &[
        "localhost",
        "127.0.0.1",
        "0.0.0.0",
        "::1",
    ];
    
    /// 禁止的IP範圍前綴
    pub const FORBIDDEN_IP_PREFIXES: &[&str] = &[
        "192.168.",
        "10.",
        "172.16.",
        "172.17.",
        "172.18.",
        "172.19.",
        "172.20.",
        "172.21.",
        "172.22.",
        "172.23.",
        "172.24.",
        "172.25.",
        "172.26.",
        "172.27.",
        "172.28.",
        "172.29.",
        "172.30.",
        "172.31.",
        "fe80:",
        "::1",
    ];
    
    /// 允許的URL方案
    pub const ALLOWED_SCHEMES: &[&str] = &["https"];
    
    /// 檔案名稱中禁止的字元
    pub const FORBIDDEN_FILENAME_CHARS: &[char] = &[
        '/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0', '\n', '\r',
    ];
}

/// Discord相關常數
pub mod discord {
    /// Discord CDN基礎URL
    pub const CDN_BASE_URL: &str = "https://cdn.discordapp.com";
    
    /// 用戶頭像URL模板
    pub const AVATAR_URL_TEMPLATE: &str = "https://cdn.discordapp.com/avatars/{user_id}/{avatar_hash}.png?size=256";
    
    /// 默認頭像URL模板
    pub const DEFAULT_AVATAR_URL_TEMPLATE: &str = "https://cdn.discordapp.com/embed/avatars/{discriminator}.png";
    
    /// 頭像圖片大小
    pub const AVATAR_SIZE: u16 = 256;
    
    /// Discord訊息最大長度
    pub const MAX_MESSAGE_LENGTH: usize = 2000;
    
    /// Discord嵌入訊息最大長度
    pub const MAX_EMBED_LENGTH: usize = 6000;
}

/// 檔案系統相關常數
pub mod filesystem {
    /// 背景圖片目錄名稱
    pub const BACKGROUNDS_DIR: &str = "backgrounds";
    
    /// 臨時檔案目錄名稱
    pub const TEMP_DIR: &str = "temp";
    
    /// 快取目錄名稱
    pub const CACHE_DIR: &str = "cache";
    
    /// 背景圖片檔案名前綴
    pub const BACKGROUND_PREFIX: &str = "bg_";
    
    /// 預覽圖片檔案名前綴
    pub const PREVIEW_PREFIX: &str = "preview_";
    
    /// 默認檔案權限（Unix）
    pub const DEFAULT_FILE_MODE: u32 = 0o644;
    
    /// 默認目錄權限（Unix）
    pub const DEFAULT_DIR_MODE: u32 = 0o755;
}

/// 快取相關常數
pub mod cache {
    use super::Duration;
    
    /// 用戶頭像快取TTL
    pub const AVATAR_CACHE_TTL: Duration = Duration::from_secs(3600); // 1小時
    
    /// 配置快取TTL
    pub const CONFIG_CACHE_TTL: Duration = Duration::from_secs(300); // 5分鐘
    
    /// 圖片快取TTL
    pub const IMAGE_CACHE_TTL: Duration = Duration::from_secs(1800); // 30分鐘
    
    /// 最大快取條目數
    pub const MAX_CACHE_ENTRIES: usize = 1000;
}

/// 錯誤訊息相關常數
pub mod errors {
    /// 通用錯誤訊息
    pub const GENERIC_ERROR: &str = "發生未預期的錯誤，請稍後再試。";
    
    /// 權限不足訊息
    pub const INSUFFICIENT_PERMISSIONS: &str = "您沒有足夠的權限執行此操作。";
    
    /// 檔案過大訊息
    pub const FILE_TOO_LARGE: &str = "檔案過大，請選擇小於 5MB 的圖片。";
    
    /// 不支援格式訊息
    pub const UNSUPPORTED_FORMAT: &str = "不支援的檔案格式，請使用 PNG 或 JPEG 格式。";
    
    /// 網絡錯誤訊息
    pub const NETWORK_ERROR: &str = "網絡連接失敗，請檢查網絡連接或稍後再試。";
    
    /// 配置錯誤訊息
    pub const CONFIG_ERROR: &str = "配置操作失敗，請稍後再試。";
    
    /// URL無效訊息
    pub const INVALID_URL: &str = "無效的網址，請提供有效的 HTTPS 圖片連結。";
}

/// 日誌相關常數
pub mod logging {
    /// 性能日誌閾值
    pub const PERFORMANCE_LOG_THRESHOLD_MS: u64 = 1000; // 1秒
    
    /// 大檔案日誌閾值
    pub const LARGE_FILE_LOG_THRESHOLD: usize = 1024 * 1024; // 1MB
    
    /// 錯誤重試日誌間隔
    pub const RETRY_LOG_INTERVAL: u32 = 1;
}

/// 環境變數名稱
pub mod env_vars {
    /// 資產目錄路徑
    pub const ASSETS_DIR: &str = "DROAS_ASSETS_DIR";
    
    /// 最大圖片大小
    pub const MAX_IMAGE_SIZE: &str = "DROAS_MAX_IMAGE_SIZE";
    
    /// HTTP超時時間
    pub const HTTP_TIMEOUT: &str = "DROAS_HTTP_TIMEOUT";
    
    /// 命令超時時間
    pub const COMMAND_TIMEOUT: &str = "DROAS_COMMAND_TIMEOUT";
    
    /// 啟用詳細日誌
    pub const VERBOSE_LOGGING: &str = "DROAS_VERBOSE_LOGGING";
}

/// 運行時配置結構
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// 資產目錄路徑
    pub assets_dir: String,
    /// 最大圖片大小
    pub max_image_size: usize,
    /// HTTP請求超時時間
    pub http_timeout: Duration,
    /// 命令處理超時時間
    pub command_timeout: Duration,
    /// 是否啟用詳細日誌
    pub verbose_logging: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            assets_dir: std::env::var(env_vars::ASSETS_DIR)
                .unwrap_or_else(|_| "./assets".to_string()),
            max_image_size: std::env::var(env_vars::MAX_IMAGE_SIZE)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(image::MAX_IMAGE_SIZE),
            http_timeout: std::env::var(env_vars::HTTP_TIMEOUT)
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or(http::DEFAULT_TIMEOUT),
            command_timeout: std::env::var(env_vars::COMMAND_TIMEOUT)
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .map(Duration::from_secs)
                .unwrap_or(performance::COMMAND_TIMEOUT),
            verbose_logging: std::env::var(env_vars::VERBOSE_LOGGING)
                .map(|s| s.to_lowercase() == "true" || s == "1")
                .unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_constants() {
        assert_eq!(image::MAX_IMAGE_SIZE, 5 * 1024 * 1024);
        assert_eq!(image::PREVIEW_WIDTH, 1024);
        assert_eq!(image::PREVIEW_HEIGHT, 512);
        assert!(image::SUPPORTED_FORMATS.contains(&"PNG"));
        assert!(image::SUPPORTED_FORMATS.contains(&"JPEG"));
    }

    #[test]
    fn test_http_constants() {
        assert_eq!(http::DEFAULT_TIMEOUT, Duration::from_secs(30));
        assert_eq!(http::MAX_RETRIES, 3);
        assert_eq!(http::USER_AGENT, "DROAS-Bot/0.1.0");
    }

    #[test]
    fn test_security_constants() {
        assert!(security::FORBIDDEN_HOSTS.contains(&"localhost"));
        assert!(security::FORBIDDEN_IP_PREFIXES.contains(&"192.168."));
        assert!(security::ALLOWED_SCHEMES.contains(&"https"));
    }

    #[test]
    fn test_runtime_config_default() {
        let config = RuntimeConfig::default();
        assert!(!config.assets_dir.is_empty());
        assert!(config.max_image_size > 0);
        assert!(config.http_timeout > Duration::ZERO);
        assert!(config.command_timeout > Duration::ZERO);
    }

    #[test]
    fn test_magic_numbers() {
        assert_eq!(image::PNG_MAGIC.len(), 8);
        assert_eq!(image::JPEG_MAGIC.len(), 3);
        assert_eq!(image::GIF_MAGIC_87A, b"GIF87a");
        assert_eq!(image::WEBP_MAGIC, b"WEBP");
    }
}