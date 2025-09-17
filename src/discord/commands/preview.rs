//! /preview 命令實現
//! 
//! 允許管理員預覽渲染的歡迎圖片，使用調用者的頭像和用戶名。

use crate::discord::commands::framework::{
    CommandHandler, CommandContext, CommandResult, CommandError, PermissionLevel,
};
use async_trait::async_trait;
use reqwest::Client;
use serenity::builder::CreateApplicationCommand;
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// /preview 命令處理器
pub struct PreviewHandler {
    http_client: Client,
    assets_dir: String,
}

impl PreviewHandler {
    /// 創建新的 /preview 命令處理器
    pub fn new(assets_dir: String) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("DROAS-Bot/0.1.0")
            .build()
            .expect("無法創建 HTTP 客戶端");

        Self {
            http_client,
            assets_dir,
        }
    }
}

#[async_trait]
impl CommandHandler for PreviewHandler {
    async fn handle(&self, ctx: CommandContext) -> CommandResult<()> {
        let start_time = Instant::now();
        debug!("開始處理 /preview 命令");
        
        // 延遲回應，因為圖片渲染可能需要時間
        ctx.defer_response().await?;
        
        let guild_id = ctx
            .guild_id()
            .ok_or_else(|| CommandError::ExecutionFailed("此命令只能在伺服器中使用".to_string()))?;
        
        // 獲取用戶信息
        let user = &ctx.interaction.user;
        let username = &user.name;
        let user_id = user.id;
        
        debug!("為用戶生成預覽 - ID: {}, 用戶名: {}", user_id, username);
        
        // 獲取用戶頭像URL
        let avatar_url = self.get_user_avatar_url(user).await?;
        
        // 獲取公會配置以取得背景圖片
        let background_path = self.get_background_path(&ctx, guild_id.0 as i64).await?;
        
        // 生成預覽圖片
        let preview_image = self.generate_preview_image(
            username,
            &avatar_url,
            background_path.as_deref(),
        ).await?;
        
        // 計算處理時間
        let processing_time = start_time.elapsed();
        debug!("預覽圖片生成完成，耗時: {:?}", processing_time);
        
        // 發送預覽圖片
        let filename = format!("preview_{}.png", user_id);
        ctx.respond_with_file(
            &format!(
                "🖼️ **歡迎圖片預覽**\n👤 用戶：{}\n⏱️ 生成時間：{:.2}秒",
                username,
                processing_time.as_secs_f64()
            ),
            &filename,
            preview_image,
        ).await?;
        
        info!(
            "成功生成預覽圖片 - 用戶: {}, 公會: {}, 耗時: {:?}",
            user_id, guild_id.0, processing_time
        );
        
        // 檢查性能目標（P95 < 3秒）
        if processing_time.as_secs() >= 3 {
            warn!(
                "預覽生成時間過長: {:?} (目標 < 3秒) - 用戶: {}, 公會: {}",
                processing_time, user_id, guild_id.0
            );
        }
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "preview"
    }
    
    fn description(&self) -> &'static str {
        "預覽歡迎圖片，使用您的頭像和用戶名"
    }
    
    fn required_permissions(&self) -> PermissionLevel {
        PermissionLevel::Everyone // 所有人都可以預覽
    }
    
    fn register(&self, command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(self.name())
            .description(self.description())
    }
}

impl PreviewHandler {
    /// 獲取用戶頭像URL
    async fn get_user_avatar_url(&self, user: &serenity::model::user::User) -> CommandResult<String> {
        // 優先使用用戶自定義頭像
        if let Some(avatar_hash) = &user.avatar {
            let avatar_url = format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png?size=256",
                user.id.0, avatar_hash
            );
            debug!("使用用戶頭像: {}", avatar_url);
            return Ok(avatar_url);
        }
        
        // 使用默認頭像
        let default_avatar_url = format!(
            "https://cdn.discordapp.com/embed/avatars/{}.png",
            user.discriminator % 5
        );
        debug!("使用默認頭像: {}", default_avatar_url);
        Ok(default_avatar_url)
    }
    
    /// 獲取公會背景圖片路徑
    async fn get_background_path(&self, ctx: &CommandContext, guild_id: i64) -> CommandResult<Option<String>> {
        debug!("獲取公會 {} 的背景配置", guild_id);
        
        match ctx.config_service.get_config(guild_id).await {
            Ok(Some(config)) => {
                if let Some(background_ref) = config.background_ref {
                    let background_path = format!("{}/backgrounds/{}.png", self.assets_dir, background_ref);
                    
                    // 檢查文件是否存在
                    if tokio::fs::metadata(&background_path).await.is_ok() {
                        debug!("找到背景圖片: {}", background_path);
                        Ok(Some(background_path))
                    } else {
                        // 嘗試 .jpg 檔案
                        let jpg_path = format!("{}/backgrounds/{}.jpg", self.assets_dir, background_ref);
                        if tokio::fs::metadata(&jpg_path).await.is_ok() {
                            debug!("找到背景圖片 (JPG): {}", jpg_path);
                            Ok(Some(jpg_path))
                        } else {
                            warn!("背景圖片文件不存在: {} 或 {}", background_path, jpg_path);
                            Ok(None)
                        }
                    }
                } else {
                    debug!("公會 {} 未設置背景圖片", guild_id);
                    Ok(None)
                }
            }
            Ok(None) => {
                debug!("公會 {} 沒有配置", guild_id);
                Ok(None)
            }
            Err(e) => {
                error!("獲取公會配置失敗: {}", e);
                // 配置獲取失敗不應該阻止預覽功能，使用默認背景
                Ok(None)
            }
        }
    }
    
    /// 生成預覽圖片
    async fn generate_preview_image(
        &self,
        username: &str,
        avatar_url: &str,
        background_path: Option<&str>,
    ) -> CommandResult<Vec<u8>> {
        debug!("開始生成預覽圖片 - 用戶名: {}, 頭像: {}, 背景: {:?}", 
               username, avatar_url, background_path);
        
        // 下載用戶頭像
        let avatar_data = self.download_avatar(avatar_url).await?;
        
        // 載入背景圖片（如果有）
        let background_data = if let Some(bg_path) = background_path {
            Some(self.load_background_image(bg_path).await?)
        } else {
            None
        };
        
        // 使用簡單的圖片合成生成預覽
        // 注意：在實際實現中，這裡應該集成 CORE-003 圖像渲染引擎
        let preview_image = self.compose_welcome_image(
            username,
            &avatar_data,
            background_data.as_deref(),
        ).await?;
        
        Ok(preview_image)
    }
    
    /// 下載用戶頭像
    async fn download_avatar(&self, avatar_url: &str) -> CommandResult<Vec<u8>> {
        debug!("下載頭像: {}", avatar_url);
        
        let response = self
            .http_client
            .get(avatar_url)
            .send()
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("下載頭像失敗: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(CommandError::ExecutionFailed(format!(
                "下載頭像失敗，HTTP狀態: {}",
                response.status()
            )));
        }
        
        let data = response
            .bytes()
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("讀取頭像數據失敗: {}", e)))?
            .to_vec();
        
        debug!("成功下載頭像: {} bytes", data.len());
        Ok(data)
    }
    
    /// 載入背景圖片
    async fn load_background_image(&self, background_path: &str) -> CommandResult<Vec<u8>> {
        debug!("載入背景圖片: {}", background_path);
        
        let data = tokio::fs::read(background_path)
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("讀取背景圖片失敗: {}", e)))?;
        
        debug!("成功載入背景圖片: {} bytes", data.len());
        Ok(data)
    }
    
    /// 合成歡迎圖片
    async fn compose_welcome_image(
        &self,
        username: &str,
        avatar_data: &[u8],
        background_data: Option<&[u8]>,
    ) -> CommandResult<Vec<u8>> {
        debug!("合成歡迎圖片 - 用戶名: {}", username);
        
        // 注意：這是一個簡化的實現
        // 在實際項目中，這裡應該：
        // 1. 集成 CORE-003 圖像渲染引擎
        // 2. 使用專業的圖像處理庫（如 image crate）
        // 3. 支持複雜的佈局和字體渲染
        
        // 為了演示目的，這裡創建一個簡單的佔位符圖片
        let placeholder_image = self.create_placeholder_image(username).await?;
        
        Ok(placeholder_image)
    }
    
    /// 創建佔位符圖片（用於演示）
    async fn create_placeholder_image(&self, username: &str) -> CommandResult<Vec<u8>> {
        // 這是一個非常簡化的實現，創建一個基本的PNG圖片
        // 實際實現應該使用專業的圖像處理庫
        
        // 創建一個簡單的1024x512的PNG圖片數據
        // 這裡使用一個最小的PNG文件作為模板
        let mut png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG 簽名
            0x00, 0x00, 0x00, 0x0D, // IHDR 長度
            0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x04, 0x00, // 寬度 1024
            0x00, 0x00, 0x02, 0x00, // 高度 512
            0x08, 0x06, 0x00, 0x00, 0x00, // 位深度, 顏色類型
            0xA4, 0x61, 0xE2, 0x2E, // CRC
        ];
        
        // 添加一個簡單的數據塊（IDAT）
        let idat_data = vec![0x78, 0x9C, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01]; // 壓縮數據
        png_data.extend_from_slice(&(idat_data.len() as u32).to_be_bytes());
        png_data.extend_from_slice(b"IDAT");
        png_data.extend_from_slice(&idat_data);
        png_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // CRC 佔位符
        
        // 添加 IEND
        png_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // IEND 長度
        png_data.extend_from_slice(b"IEND");
        png_data.extend_from_slice(&[0xAE, 0x42, 0x60, 0x82]); // IEND CRC
        
        debug!("創建佔位符圖片完成 - 用戶名: {}, 大小: {} bytes", username, png_data.len());
        
        Ok(png_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_handler() -> (PreviewHandler, TempDir) {
        let temp_dir = TempDir::new().expect("創建臨時目錄失敗");
        let handler = PreviewHandler::new(temp_dir.path().to_string_lossy().to_string());
        (handler, temp_dir)
    }
    
    #[test]
    fn test_handler_properties() {
        let (handler, _temp_dir) = create_test_handler();
        
        assert_eq!(handler.name(), "preview");
        assert_eq!(handler.required_permissions(), PermissionLevel::Everyone);
        assert!(!handler.description().is_empty());
    }
    
    #[tokio::test]
    async fn test_create_placeholder_image() {
        let (handler, _temp_dir) = create_test_handler();
        
        let result = handler.create_placeholder_image("TestUser").await;
        assert!(result.is_ok());
        
        let image_data = result.unwrap();
        assert!(!image_data.is_empty());
        
        // 驗證PNG文件頭
        assert_eq!(&image_data[0..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }
    
    #[test]
    fn test_get_user_avatar_url() {
        use serenity::model::user::User;
        use serenity::model::id::UserId;
        
        let (handler, _temp_dir) = create_test_handler();
        
        // 創建測試用戶（有頭像）
        let mut user_with_avatar = User::default();
        user_with_avatar.id = UserId(123456789);
        user_with_avatar.avatar = Some("abcdef123456".to_string());
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let avatar_url = runtime.block_on(handler.get_user_avatar_url(&user_with_avatar)).unwrap();
        
        assert!(avatar_url.contains("https://cdn.discordapp.com/avatars/"));
        assert!(avatar_url.contains("123456789"));
        assert!(avatar_url.contains("abcdef123456"));
        
        // 創建測試用戶（無頭像）
        let mut user_without_avatar = User::default();
        user_without_avatar.id = UserId(987654321);
        user_without_avatar.discriminator = 1234;
        user_without_avatar.avatar = None;
        
        let default_avatar_url = runtime.block_on(handler.get_user_avatar_url(&user_without_avatar)).unwrap();
        
        assert!(default_avatar_url.contains("https://cdn.discordapp.com/embed/avatars/"));
        assert!(default_avatar_url.contains(&(1234 % 5).to_string()));
    }
}