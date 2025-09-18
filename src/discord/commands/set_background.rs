//! /set-background 命令實現
//! 
//! 允許具有管理公會權限的用戶設置或更新公會的歡迎背景圖片。

use crate::config::models::{GuildConfig, BackgroundAsset};
use crate::discord::commands::framework::{
    CommandHandler, CommandContext, CommandResult, CommandError, PermissionLevel, BoxFuture,
};
use crate::discord::commands::services::http_service::HttpService;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::CommandDataOptionValue;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// /set-background 命令處理器
#[derive(Clone)]
pub struct SetBackgroundHandler {
    http_service: Arc<HttpService>,
    assets_dir: String,
}

impl SetBackgroundHandler {
    /// 創建新的 /set-background 命令處理器
    pub fn new(assets_dir: String, http_service: Arc<HttpService>) -> Self {
        Self {
            http_service,
            assets_dir,
        }
    }
}

impl CommandHandler for SetBackgroundHandler {
    fn handle(&self, ctx: CommandContext) -> BoxFuture<'_, CommandResult<()>> {
        let this = self.clone();
        Box::pin(async move {
            this.handle_impl(ctx).await
        })
    }
}

impl SetBackgroundHandler {
    async fn handle_impl(&self, ctx: CommandContext) -> CommandResult<()> {
        debug!("開始處理 /set-background 命令");
        
        // 延遲回應，因為圖片處理可能需要時間
        ctx.defer_response().await?;
        
        let guild_id = ctx
            .guild_id()
            .ok_or_else(|| CommandError::ExecutionFailed("此命令只能在伺服器中使用".to_string()))?;
        
        // 解析命令參數
        let (image_source, image_data) = self.parse_image_source(&ctx).await?;
        
        // 驗證圖片
        let validated_image = self.validate_image(image_data).await?;
        
        // 保存圖片到資產目錄
        let asset_id = self.save_image_asset(&validated_image).await?;
        
        // 更新公會配置
        self.update_guild_config(&ctx, guild_id.0 as i64, &asset_id).await?;
        
        // 回應成功訊息
        let success_message = format!(
            "✅ 成功設置背景圖片！\n📄 來源：{}\n🆔 資源ID：{}\n💾 檔案大小：{} KB",
            image_source,
            asset_id,
            validated_image.data.len() / 1024
        );
        
        ctx.edit_response(&success_message).await?;
        
        info!(
            "成功設置背景圖片 - 公會: {}, 資源ID: {}, 大小: {} bytes",
            guild_id.0,
            asset_id,
            validated_image.data.len()
        );
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "set-background"
    }
    
    fn description(&self) -> &'static str {
        "設置伺服器歡迎訊息的背景圖片（需要管理伺服器權限）"
    }
    
    fn required_permissions(&self) -> PermissionLevel {
        PermissionLevel::ManageGuild
    }
    
    fn register(&self, command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name(self.name())
            .description(self.description())
            .create_option(|option| {
                option
                    .name("attachment")
                    .description("上傳背景圖片檔案（PNG或JPEG，最大5MB）")
                    .kind(CommandOptionType::Attachment)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name("url")
                    .description("背景圖片的HTTPS網址（PNG或JPEG，最大5MB）")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
    }
}

impl SetBackgroundHandler {
    /// 解析圖片來源（附件或URL）
    async fn parse_image_source(&self, ctx: &CommandContext) -> CommandResult<(String, Vec<u8>)> {
        // 檢查附件參數
        if let Some(attachment_option) = ctx.get_option("attachment") {
            if let CommandDataOptionValue::Attachment(attachment_id) = &attachment_option.resolved {
                let attachment = ctx
                    .interaction
                    .data
                    .resolved
                    .attachments
                    .get(attachment_id)
                    .ok_or_else(|| CommandError::InvalidArguments("找不到指定的附件".to_string()))?;
                
                debug!("處理附件: {}, 大小: {} bytes", attachment.filename, attachment.size);
                
                // 使用HttpService下載附件（Discord附件URL為安全URL）
                let data = self.http_service.download_data(&attachment.url).await?;
                return Ok((format!("附件：{}", attachment.filename), data));
            }
        }
        
        // 檢查URL參數
        if let Some(url_option) = ctx.get_option("url") {
            if let CommandDataOptionValue::String(url) = &url_option.resolved {
                debug!("處理URL: {}", url);
                
                // 使用HttpService下載圖片（包含URL驗證）
                let data = self.http_service.download_image(url).await?;
                return Ok((format!("URL：{}", url), data));
            }
        }
        
        Err(CommandError::InvalidArguments(
            "請提供圖片附件或URL。使用方法：\n• /set-background attachment:<上傳圖片>\n• /set-background url:<圖片網址>".to_string()
        ))
    }
    
    
    /// 驗證圖片數據
    async fn validate_image(&self, data: Vec<u8>) -> CommandResult<ValidatedImage> {
        debug!("驗證圖片數據：{} bytes", data.len());
        
        // 檢查檔案大小
        const MAX_SIZE: usize = 5 * 1024 * 1024; // 5MB
        if data.len() > MAX_SIZE {
            return Err(CommandError::InvalidArguments(format!(
                "圖片檔案過大：{} bytes（最大允許：{} bytes）",
                data.len(),
                MAX_SIZE
            )));
        }
        
        if data.is_empty() {
            return Err(CommandError::InvalidArguments("圖片檔案為空".to_string()));
        }
        
        // 檢查檔案格式
        let (format, mime_type) = self.detect_image_format(&data)?;
        
        debug!("檢測到圖片格式：{}, MIME類型：{}", format, mime_type);
        
        Ok(ValidatedImage {
            data,
            format,
            mime_type,
        })
    }
    
    /// 檢測圖片格式
    fn detect_image_format(&self, data: &[u8]) -> CommandResult<(String, String)> {
        if data.len() < 8 {
            return Err(CommandError::InvalidArguments(
                "檔案過小，無法識別格式".to_string(),
            ));
        }
        
        // PNG 格式檢測
        if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Ok(("PNG".to_string(), "image/png".to_string()));
        }
        
        // JPEG 格式檢測
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Ok(("JPEG".to_string(), "image/jpeg".to_string()));
        }
        
        // 檢查是否為其他常見但不支援的格式
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return Err(CommandError::InvalidArguments(
                "不支援GIF格式，請使用PNG或JPEG".to_string(),
            ));
        }
        
        if data.starts_with(b"RIFF") && data.len() > 12 && &data[8..12] == b"WEBP" {
            return Err(CommandError::InvalidArguments(
                "不支援WebP格式，請使用PNG或JPEG".to_string(),
            ));
        }
        
        Err(CommandError::InvalidArguments(
            "不支援的圖片格式，請使用PNG或JPEG".to_string(),
        ))
    }
    
    /// 保存圖片到資產目錄
    async fn save_image_asset(&self, image: &ValidatedImage) -> CommandResult<String> {
        // 生成唯一的資產ID
        let asset_id = format!(
            "bg_{}_{}", 
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            Uuid::new_v4().to_simple()
        );
        
        // 確定檔案副檔名
        let extension = match image.format.as_str() {
            "PNG" => "png",
            "JPEG" => "jpg",
            _ => return Err(CommandError::Internal(
                crate::error::DroasError::validation("未知圖片格式"),
            )),
        };
        
        let filename = format!("{}.{}", asset_id, extension);
        let file_path = format!("{}/backgrounds/{}", self.assets_dir, filename);
        
        debug!("保存圖片到：{}", file_path);
        
        // 確保目錄存在
        let dir_path = format!("{}/backgrounds", self.assets_dir);
        fs::create_dir_all(&dir_path)
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("創建目錄失敗：{}", e)))?;
        
        // 寫入檔案
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("創建檔案失敗：{}", e)))?;
        
        file.write_all(&image.data)
            .await
            .map_err(|e| CommandError::ExecutionFailed(format!("寫入檔案失敗：{}", e)))?;
        
        info!("成功保存圖片資產：{} ({} bytes)", asset_id, image.data.len());
        
        Ok(asset_id)
    }
    
    /// 更新公會配置
    async fn update_guild_config(
        &self,
        ctx: &CommandContext,
        guild_id: i64,
        asset_id: &str,
    ) -> CommandResult<()> {
        debug!("更新公會 {} 的背景配置為：{}", guild_id, asset_id);
        
        // 獲取現有配置或創建新配置
        let mut config = ctx
            .config_service
            .get_config(guild_id)
            .await
            .map_err(CommandError::Internal)?
            .unwrap_or_else(|| GuildConfig::new(guild_id, None, None));
        
        // 更新背景引用
        config.update_background(Some(asset_id.to_string()));
        
        // 保存配置
        match ctx.config_service.update_config(&config).await {
            Ok(crate::config::service::ConfigUpdateResult::Success) => {
                debug!("配置更新成功");
                Ok(())
            }
            Ok(crate::config::service::ConfigUpdateResult::Failed(error)) => {
                error!("配置更新失敗：{}", error);
                Err(CommandError::ExecutionFailed(format!(
                    "更新配置失敗：{}", error
                )))
            }
            Ok(crate::config::service::ConfigUpdateResult::Timeout) => {
                error!("配置更新超時");
                Err(CommandError::ExecutionFailed(
                    "更新配置超時，請稍後再試".to_string(),
                ))
            }
            Err(e) => {
                error!("配置服務錯誤：{}", e);
                Err(CommandError::Internal(
                    crate::error::DroasError::database(format!("配置服務錯誤：{}", e)),
                ))
            }
        }
    }
}

/// 已驗證的圖片數據
struct ValidatedImage {
    data: Vec<u8>,
    format: String,
    mime_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discord::commands::services::http_service::HttpService;
    use tempfile::TempDir;
    use std::sync::Arc;
    
    fn create_test_handler() -> (SetBackgroundHandler, TempDir) {
        let temp_dir = TempDir::new().expect("創建臨時目錄失敗");
        let http_service = Arc::new(HttpService::with_default_config().expect("創建 HTTP 服務失敗"));
        let handler = SetBackgroundHandler::new(temp_dir.path().to_string_lossy().to_string(), http_service);
        (handler, temp_dir)
    }
    
    #[test]
    fn test_png_format_detection() {
        let (handler, _temp_dir) = create_test_handler();
        
        // PNG 文件頭
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let result = handler.detect_image_format(&png_header);
        
        assert!(result.is_ok());
        let (format, mime_type) = result.unwrap();
        assert_eq!(format, "PNG");
        assert_eq!(mime_type, "image/png");
    }
    
    #[test]
    fn test_jpeg_format_detection() {
        let (handler, _temp_dir) = create_test_handler();
        
        // JPEG 文件頭
        let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0];
        let result = handler.detect_image_format(&jpeg_header);
        
        assert!(result.is_ok());
        let (format, mime_type) = result.unwrap();
        assert_eq!(format, "JPEG");
        assert_eq!(mime_type, "image/jpeg");
    }
    
    #[test]
    fn test_unsupported_format_detection() {
        let (handler, _temp_dir) = create_test_handler();
        
        // GIF 文件頭
        let gif_header = b"GIF87a";
        let result = handler.detect_image_format(gif_header);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不支援GIF格式"));
    }
    
    
    #[tokio::test]
    async fn test_image_size_validation() {
        let (handler, _temp_dir) = create_test_handler();
        
        // 有效大小的PNG
        let small_png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert!(handler.validate_image(small_png).await.is_ok());
        
        // 過大的檔案（超過5MB）
        let large_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
            .into_iter()
            .cycle()
            .take(6 * 1024 * 1024) // 6MB
            .collect::<Vec<u8>>();
        
        assert!(handler.validate_image(large_data).await.is_err());
    }
}