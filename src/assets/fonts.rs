use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncReadExt;
use tracing::{debug, info, warn};

/// 支持的字體格式
#[derive(Debug, Clone, PartialEq)]
pub enum FontFormat {
    TrueType,
    OpenType,
}

impl FontFormat {
    /// 從檔案擴展名判斷格式
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "ttf" => Some(FontFormat::TrueType),
            "otf" => Some(FontFormat::OpenType),
            _ => None,
        }
    }

    /// 獲取檔案擴展名
    pub fn extension(&self) -> &'static str {
        match self {
            FontFormat::TrueType => "ttf",
            FontFormat::OpenType => "otf",
        }
    }
}

/// 字體信息
#[derive(Debug, Clone)]
pub struct FontInfo {
    /// 字體名稱
    pub name: String,
    /// 檔案路徑
    pub file_path: PathBuf,
    /// 字體格式
    pub format: FontFormat,
    /// 檔案大小（字節）
    pub file_size: u64,
}

/// 字體管理器
pub struct FontManager {
    /// 字體存儲目錄
    fonts_path: PathBuf,
}

impl FontManager {
    /// 創建新的字體管理器
    ///
    /// # Arguments
    /// * `fonts_path` - 字體存儲目錄
    pub async fn new<P: AsRef<Path>>(fonts_path: P) -> Result<Self> {
        let fonts_path = fonts_path.as_ref().to_path_buf();
        info!("初始化字體管理器: fonts_path={}", fonts_path.display());

        // 確保字體目錄存在
        fs::create_dir_all(&fonts_path).await?;

        // 檢查是否有預設字體，如果沒有則創建佔位符
        let manager = Self { fonts_path };
        manager.ensure_default_fonts().await?;

        Ok(manager)
    }

    /// 確保默認字體存在
    async fn ensure_default_fonts(&self) -> Result<()> {
        let default_font_path = self.fonts_path.join("default.ttf");

        if !default_font_path.exists() {
            info!("創建默認字體佔位符");

            // 創建一個README檔案指導用戶放置字體
            let readme_content = r#"# 字體目錄

請將您的字體檔案放置在此目錄中。

## 支持的格式
- TTF (TrueType Font) - 推薦
- OTF (OpenType Font)

## 建議的字體
- 思源黑體 (Noto Sans CJK)
- 微軟雅黑 (Microsoft YaHei)
- 或任何其他支援中文的字體

## 注意事項
- 確保字體檔案有適當的使用許可
- 字體檔案名應該是描述性的
- 避免使用特殊字符在檔案名中

## 默認字體
如果沒有找到其他字體，系統會嘗試使用系統默認字體。
"#;

            let readme_path = self.fonts_path.join("README.md");
            fs::write(readme_path, readme_content).await?;
        }

        Ok(())
    }

    /// 列出所有可用的字體
    pub async fn list_fonts(&self) -> Result<Vec<FontInfo>> {
        debug!("列出字體目錄中的字體");
        let mut fonts = Vec::new();

        if !self.fonts_path.exists() {
            return Ok(fonts);
        }

        let mut entries = fs::read_dir(&self.fonts_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if let Some(format) = FontFormat::from_extension(ext) {
                        if let Ok(metadata) = fs::metadata(&path).await {
                            let name = path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            fonts.push(FontInfo {
                                name,
                                file_path: path,
                                format,
                                file_size: metadata.len(),
                            });
                        }
                    }
                }
            }
        }

        debug!("找到 {} 個字體檔案", fonts.len());
        Ok(fonts)
    }

    /// 讀取字體數據
    ///
    /// # Arguments
    /// * `font_name` - 字體名稱
    ///
    /// # Returns
    /// * `Result<Vec<u8>>` - 字體二進制數據
    pub async fn load_font_data(&self, font_name: &str) -> Result<Vec<u8>> {
        debug!("載入字體數據: {}", font_name);

        // 嘗試多種檔案擴展名
        let possible_extensions = ["ttf", "otf"];

        for ext in &possible_extensions {
            let font_path = self.fonts_path.join(format!("{}.{}", font_name, ext));

            if font_path.exists() {
                let mut file = fs::File::open(&font_path).await?;
                let mut data = Vec::new();
                file.read_to_end(&mut data).await?;

                debug!(
                    "字體載入成功: {}, size={} bytes",
                    font_path.display(),
                    data.len()
                );
                return Ok(data);
            }
        }

        Err(anyhow::anyhow!("找不到字體檔案: {}", font_name))
    }

    /// 根據路徑讀取字體數據
    pub async fn load_font_data_by_path<P: AsRef<Path>>(&self, font_path: P) -> Result<Vec<u8>> {
        let font_path = font_path.as_ref();
        debug!("根據路徑載入字體數據: {}", font_path.display());

        if !font_path.exists() {
            return Err(anyhow::anyhow!("字體檔案不存在: {}", font_path.display()));
        }

        let mut file = fs::File::open(font_path).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        debug!("字體載入成功: size={} bytes", data.len());
        Ok(data)
    }

    /// 獲取首選字體
    ///
    /// 根據優先順序返回可用的字體：
    /// 1. 指定的字體名稱
    /// 2. 中文友好的字體
    /// 3. 任何可用的字體
    pub async fn get_preferred_font(
        &self,
        preferred_name: Option<&str>,
    ) -> Result<Option<FontInfo>> {
        let fonts = self.list_fonts().await?;

        if fonts.is_empty() {
            warn!("沒有找到任何字體檔案");
            return Ok(None);
        }

        // 如果指定了偏好字體
        if let Some(name) = preferred_name {
            if let Some(font) = fonts
                .iter()
                .find(|f| f.name.to_lowercase() == name.to_lowercase())
            {
                return Ok(Some(font.clone()));
            }
        }

        // 尋找中文友好的字體
        let chinese_friendly = [
            "noto",
            "microsoft",
            "yahei",
            "simhei",
            "simsun",
            "source",
            "sans",
        ];
        for keyword in &chinese_friendly {
            if let Some(font) = fonts
                .iter()
                .find(|f| f.name.to_lowercase().contains(keyword))
            {
                return Ok(Some(font.clone()));
            }
        }

        // 返回第一個可用字體
        Ok(fonts.into_iter().next())
    }

    /// 驗證字體檔案
    pub async fn validate_font<P: AsRef<Path>>(&self, font_path: P) -> Result<bool> {
        let font_path = font_path.as_ref();
        debug!("驗證字體檔案: {}", font_path.display());

        if !font_path.exists() {
            return Ok(false);
        }

        // 檢查檔案大小（至少要有基本的字體頭部）
        let metadata = fs::metadata(font_path).await?;
        if metadata.len() < 1024 {
            // 至少1KB
            warn!("字體檔案太小: {} bytes", metadata.len());
            return Ok(false);
        }

        // 基本的魔數檢查
        let mut file = fs::File::open(font_path).await?;
        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer).await?;

        // 檢查TTF/OTF魔數
        let is_valid = match &buffer {
            [0x00, 0x01, 0x00, 0x00] => true, // TTF
            [0x4F, 0x54, 0x54, 0x4F] => true, // OTF ("OTTO")
            [0x74, 0x74, 0x63, 0x66] => true, // TTC ("ttcf")
            _ => false,
        };

        if !is_valid {
            warn!("無效的字體檔案格式");
        }

        Ok(is_valid)
    }

    /// 獲取字體目錄總大小
    pub async fn get_total_size(&self) -> Result<u64> {
        let mut total_size = 0u64;

        if let Ok(mut entries) = fs::read_dir(&self.fonts_path).await {
            while let Some(entry) = entries.next_entry().await? {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(total_size)
    }

    /// 獲取字體目錄路徑
    pub fn get_fonts_path(&self) -> &Path {
        &self.fonts_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;

    async fn create_test_font_file(dir: &Path, name: &str, content: &[u8]) -> Result<PathBuf> {
        let font_path = dir.join(format!("{}.ttf", name));
        let mut file = fs::File::create(&font_path).await?;
        file.write_all(content).await?;
        file.flush().await?;
        Ok(font_path)
    }

    // 創建有效的TTF檔案頭部
    fn create_mock_ttf_header() -> Vec<u8> {
        let mut header = vec![0x00, 0x01, 0x00, 0x00]; // TTF magic number
        header.extend(vec![0; 1020]); // 填充到1KB
        header
    }

    #[tokio::test]
    async fn test_font_manager_creation() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = FontManager::new(temp_dir.path()).await;
        assert!(manager.is_ok());

        // 檢查README檔案是否被創建
        let readme_path = temp_dir.path().join("README.md");
        assert!(readme_path.exists());
    }

    #[tokio::test]
    async fn test_font_format_detection() {
        assert_eq!(
            FontFormat::from_extension("ttf"),
            Some(FontFormat::TrueType)
        );
        assert_eq!(
            FontFormat::from_extension("TTF"),
            Some(FontFormat::TrueType)
        );
        assert_eq!(
            FontFormat::from_extension("otf"),
            Some(FontFormat::OpenType)
        );
        assert_eq!(
            FontFormat::from_extension("OTF"),
            Some(FontFormat::OpenType)
        );
        assert_eq!(FontFormat::from_extension("woff"), None);
    }

    #[tokio::test]
    async fn test_list_fonts() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = FontManager::new(temp_dir.path()).await.unwrap();

        // 創建測試字體檔案
        let ttf_content = create_mock_ttf_header();
        create_test_font_file(temp_dir.path(), "test_font", &ttf_content)
            .await
            .unwrap();

        let fonts = manager.list_fonts().await.unwrap();
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0].name, "test_font");
        assert_eq!(fonts[0].format, FontFormat::TrueType);
    }

    #[tokio::test]
    async fn test_load_font_data() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = FontManager::new(temp_dir.path()).await.unwrap();

        let ttf_content = create_mock_ttf_header();
        create_test_font_file(temp_dir.path(), "test_font", &ttf_content)
            .await
            .unwrap();

        // 測試載入存在的字體
        let loaded_data = manager.load_font_data("test_font").await;
        assert!(loaded_data.is_ok());
        let loaded_data = loaded_data.unwrap();
        assert_eq!(loaded_data, ttf_content);

        // 測試載入不存在的字體
        let missing_data = manager.load_font_data("missing_font").await;
        assert!(missing_data.is_err());
    }

    #[tokio::test]
    async fn test_load_font_data_by_path() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = FontManager::new(temp_dir.path()).await.unwrap();

        let ttf_content = create_mock_ttf_header();
        let font_path = create_test_font_file(temp_dir.path(), "test_font", &ttf_content)
            .await
            .unwrap();

        let loaded_data = manager.load_font_data_by_path(&font_path).await;
        assert!(loaded_data.is_ok());
        let loaded_data = loaded_data.unwrap();
        assert_eq!(loaded_data, ttf_content);
    }

    #[tokio::test]
    async fn test_get_preferred_font() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = FontManager::new(temp_dir.path()).await.unwrap();

        let ttf_content = create_mock_ttf_header();

        // 創建多個測試字體
        create_test_font_file(temp_dir.path(), "arial", &ttf_content)
            .await
            .unwrap();
        create_test_font_file(temp_dir.path(), "noto_sans_cjk", &ttf_content)
            .await
            .unwrap();
        create_test_font_file(temp_dir.path(), "times", &ttf_content)
            .await
            .unwrap();

        // 測試指定偏好字體
        let preferred = manager.get_preferred_font(Some("arial")).await.unwrap();
        assert!(preferred.is_some());
        assert_eq!(preferred.unwrap().name, "arial");

        // 測試中文友好字體優先
        let preferred = manager.get_preferred_font(None).await.unwrap();
        assert!(preferred.is_some());
        assert_eq!(preferred.unwrap().name, "noto_sans_cjk");
    }

    #[tokio::test]
    async fn test_validate_font() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = FontManager::new(temp_dir.path()).await.unwrap();

        // 測試有效字體
        let ttf_content = create_mock_ttf_header();
        let valid_font_path = create_test_font_file(temp_dir.path(), "valid_font", &ttf_content)
            .await
            .unwrap();
        let is_valid = manager.validate_font(&valid_font_path).await.unwrap();
        assert!(is_valid);

        // 測試無效字體（太小）
        let small_content = vec![0, 1, 2, 3];
        let small_font_path = create_test_font_file(temp_dir.path(), "small_font", &small_content)
            .await
            .unwrap();
        let is_valid = manager.validate_font(&small_font_path).await.unwrap();
        assert!(!is_valid);

        // 測試無效魔數
        let mut invalid_content = vec![0xFF, 0xFF, 0xFF, 0xFF]; // 無效魔數
        invalid_content.extend(vec![0; 1020]);
        let invalid_font_path =
            create_test_font_file(temp_dir.path(), "invalid_font", &invalid_content)
                .await
                .unwrap();
        let is_valid = manager.validate_font(&invalid_font_path).await.unwrap();
        assert!(!is_valid);

        // 測試不存在的檔案
        let missing_path = temp_dir.path().join("missing.ttf");
        let is_valid = manager.validate_font(&missing_path).await.unwrap();
        assert!(!is_valid);
    }

    #[tokio::test]
    async fn test_get_total_size() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = FontManager::new(temp_dir.path()).await.unwrap();

        let ttf_content = create_mock_ttf_header();
        create_test_font_file(temp_dir.path(), "font1", &ttf_content)
            .await
            .unwrap();
        create_test_font_file(temp_dir.path(), "font2", &ttf_content)
            .await
            .unwrap();

        let total_size = manager.get_total_size().await.unwrap();

        // 總大小應該包括兩個字體檔案和README檔案
        // 每個字體檔案約1KB，README檔案可能幾百字節
        assert!(total_size > 2000); // 至少2KB
    }

    #[tokio::test]
    async fn test_get_fonts_path() {
        let temp_dir = TempDir::new().expect("無法創建臨時目錄");
        let manager = FontManager::new(temp_dir.path()).await.unwrap();

        assert_eq!(manager.get_fonts_path(), temp_dir.path());
    }
}
