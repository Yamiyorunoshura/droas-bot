//! Main image renderer for welcome images
//!
//! Coordinates all image processing components to generate welcome images.

use crate::image::{
    AvatarFetcher, AvatarProcessor, BufferPool, ContrastCalculator, ImageBuffer, ProcessedAvatar,
    TextRenderer, TextStyle, WelcomeImageError, WelcomeImageResult, WELCOME_IMAGE_HEIGHT,
    WELCOME_IMAGE_WIDTH,
};
use image::{DynamicImage, ImageFormat, Rgba};
use std::io::Cursor;
use std::path::{Path, PathBuf};

/// Configuration for rendering a welcome image
#[derive(Debug, Clone)]
pub struct WelcomeImageConfig {
    pub username: String,
    pub avatar_url: Option<String>,
    pub background_path: Option<PathBuf>,
}

/// Main image renderer that coordinates all image processing
pub struct ImageRenderer {
    buffer_pool: BufferPool,
    cache_dir: PathBuf,
    contrast_calculator: ContrastCalculator,
    avatar_fetcher: AvatarFetcher,
    avatar_processor: AvatarProcessor,
    text_renderer: TextRenderer,
}

impl ImageRenderer {
    /// Create a new image renderer
    pub async fn new(cache_dir: &Path) -> WelcomeImageResult<Self> {
        let buffer_pool = BufferPool::new(10).await?; // Pool of 10 buffers

        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            tokio::fs::create_dir_all(cache_dir).await.map_err(|e| {
                WelcomeImageError::InvalidConfig(format!("Failed to create cache dir: {}", e))
            })?;
        }

        // Initialize components
        let avatar_fetcher = AvatarFetcher::new().await?;
        let avatar_processor = AvatarProcessor::new(120); // Default 120px avatar
        let text_renderer = TextRenderer::new()?;

        Ok(Self {
            buffer_pool,
            cache_dir: cache_dir.to_path_buf(),
            contrast_calculator: ContrastCalculator::new(),
            avatar_fetcher,
            avatar_processor,
            text_renderer,
        })
    }

    /// Create a new image buffer for rendering
    pub async fn create_image_buffer(&self) -> WelcomeImageResult<ImageBuffer> {
        self.buffer_pool.get_buffer().await
    }

    /// Render a complete welcome image with error handling and performance monitoring
    pub async fn render_welcome_image(
        &mut self,
        config: &WelcomeImageConfig,
    ) -> WelcomeImageResult<Vec<u8>> {
        let start_time = std::time::Instant::now();

        // Validate configuration
        self.validate_config(config)?;

        let mut buffer =
            self.buffer_pool.get_buffer().await.map_err(|e| {
                WelcomeImageError::BufferPool(format!("Failed to get buffer: {:?}", e))
            })?;

        // Step 1: Load and apply background
        self.apply_background(&mut buffer, &config.background_path)
            .await
            .map_err(|e| {
                WelcomeImageError::ImageProcessing(image::ImageError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Background processing failed: {:?}", e),
                )))
            })?;

        // Step 2: Render avatar (if available)
        if let Some(avatar_url) = &config.avatar_url {
            // Gracefully handle avatar rendering failures
            if let Err(e) = self.render_avatar(&mut buffer, avatar_url).await {
                tracing::warn!("Avatar rendering failed, using default: {:?}", e);
                self.render_default_avatar(&mut buffer).await?;
            }
        } else {
            // Render default avatar
            self.render_default_avatar(&mut buffer).await?;
        }

        // Step 3: Render username text
        self.render_username(&mut buffer, &config.username)
            .await
            .map_err(|e| {
                WelcomeImageError::TextRendering(format!("Text rendering failed: {:?}", e))
            })?;

        // Step 4: Convert to PNG bytes
        let image_bytes = self.encode_to_png(&buffer).await?;

        // Performance monitoring
        let render_time = start_time.elapsed();
        if render_time.as_millis() > 500 {
            tracing::warn!("Slow render detected: {:?}", render_time);
        }

        // Return buffer to pool
        self.buffer_pool.return_buffer(buffer).await.map_err(|e| {
            WelcomeImageError::BufferPool(format!("Failed to return buffer: {:?}", e))
        })?;

        Ok(image_bytes)
    }

    /// Validate configuration before processing
    fn validate_config(&self, config: &WelcomeImageConfig) -> WelcomeImageResult<()> {
        // Check username length
        if config.username.len() > 100 {
            return Err(WelcomeImageError::InvalidConfig(
                "Username too long (max 100 characters)".to_string(),
            ));
        }

        // Validate background path if provided
        if let Some(bg_path) = &config.background_path {
            if !bg_path.exists() {
                return Err(WelcomeImageError::InvalidConfig(format!(
                    "Background file does not exist: {:?}",
                    bg_path
                )));
            }
        }

        Ok(())
    }

    /// Apply background image or use default
    async fn apply_background(
        &self,
        buffer: &mut ImageBuffer,
        background_path: &Option<PathBuf>,
    ) -> WelcomeImageResult<()> {
        match background_path {
            Some(path) if path.exists() => {
                // Load custom background
                let background_img =
                    image::open(path).map_err(|e| WelcomeImageError::ImageProcessing(e))?;

                // Resize to fit our dimensions
                let resized = background_img.resize_exact(
                    WELCOME_IMAGE_WIDTH,
                    WELCOME_IMAGE_HEIGHT,
                    image::imageops::FilterType::Lanczos3,
                );

                // Copy to buffer
                let rgba_img = resized.to_rgba8();
                *buffer.as_rgba_image_mut() = rgba_img;
            }
            _ => {
                // Use default gradient background
                self.create_default_background(buffer).await?;
            }
        }
        Ok(())
    }

    /// Create a simple gradient background (optimized)
    async fn create_default_background(&self, buffer: &mut ImageBuffer) -> WelcomeImageResult<()> {
        // Create a simple blue gradient background with optimized fill_rect calls
        const GRADIENT_STEPS: u32 = 32; // Use steps for better performance
        let step_height = WELCOME_IMAGE_HEIGHT / GRADIENT_STEPS;

        for step in 0..GRADIENT_STEPS {
            let y_start = step * step_height;
            let y_end = if step == GRADIENT_STEPS - 1 {
                WELCOME_IMAGE_HEIGHT
            } else {
                (step + 1) * step_height
            };

            let intensity = (255.0 * (1.0 - (step as f32 / GRADIENT_STEPS as f32) * 0.3)) as u8;
            let color = [30, 60, intensity, 255]; // Blue gradient

            buffer.fill_rect(0, y_start, WELCOME_IMAGE_WIDTH, y_end - y_start, color)?;
        }
        Ok(())
    }

    /// Render avatar from URL with full Discord CDN support
    async fn render_avatar(
        &self,
        buffer: &mut ImageBuffer,
        avatar_url: &str,
    ) -> WelcomeImageResult<()> {
        // Fetch avatar from Discord CDN
        let avatar_image = self
            .avatar_fetcher
            .fetch_avatar(avatar_url)
            .await
            .map_err(|e| WelcomeImageError::AvatarFetch(e))?;

        // Process avatar with circular mask
        let processed_avatar = self
            .avatar_processor
            .process_avatar(&avatar_image, Some(120))?;

        // Position avatar on left side of image
        let avatar_x = WELCOME_IMAGE_WIDTH / 4;
        let avatar_y = WELCOME_IMAGE_HEIGHT / 2;

        // Blend avatar onto the buffer
        self.avatar_processor.blend_avatar_onto(
            buffer.as_rgba_image_mut(),
            &processed_avatar,
            avatar_x,
            avatar_y,
        )?;

        Ok(())
    }

    /// Render default avatar placeholder using avatar processor
    async fn render_default_avatar(&self, buffer: &mut ImageBuffer) -> WelcomeImageResult<()> {
        // Create default avatar with nice styling
        let bg_color = Rgba([100, 150, 200, 255]); // Nice blue background
        let processed_avatar = self.avatar_processor.create_default_avatar(120, bg_color)?;

        // Position avatar on left side of image
        let avatar_x = WELCOME_IMAGE_WIDTH / 4;
        let avatar_y = WELCOME_IMAGE_HEIGHT / 2;

        // Blend avatar onto the buffer
        self.avatar_processor.blend_avatar_onto(
            buffer.as_rgba_image_mut(),
            &processed_avatar,
            avatar_x,
            avatar_y,
        )?;

        Ok(())
    }

    /// Render username text with real font and optimal contrast
    async fn render_username(
        &mut self,
        buffer: &mut ImageBuffer,
        username: &str,
    ) -> WelcomeImageResult<()> {
        if username.is_empty() {
            return Ok(()); // Skip empty usernames
        }

        // Define text style
        let font_size = 32.0;
        let base_style = TextStyle {
            font_size,
            color: Rgba([255, 255, 255, 255]),
            outline_color: Some(Rgba([0, 0, 0, 255])),
            outline_width: 1.5,
            letter_spacing: 0.0,
            line_height: 1.2,
        };

        // Calculate text position for centering
        let text_area_width = WELCOME_IMAGE_WIDTH - 200; // Leave margins
        let text_area_height = 60; // Height for text

        // Calculate background luminance for contrast optimization
        let sample_x = WELCOME_IMAGE_WIDTH / 2;
        let sample_y = WELCOME_IMAGE_HEIGHT - 80;
        let sample_width = 200;
        let sample_height = 40;

        let bg_luminance = self.contrast_calculator.calculate_average_luminance(
            buffer,
            sample_x,
            sample_y,
            sample_width,
            sample_height,
        )?;

        // Create contrasted text style
        let text_style = self.text_renderer.create_contrasted_style(
            Rgba([255, 255, 255, 255]),
            bg_luminance,
            font_size,
        );

        // Calculate centered position
        let (text_x, text_y) = self.text_renderer.center_text_position(
            username,
            &text_style,
            text_area_width,
            text_area_height,
        );

        // Render text with proper positioning
        let final_x = (WELCOME_IMAGE_WIDTH - text_area_width) / 2 + text_x;
        let final_y = WELCOME_IMAGE_HEIGHT - 100 + text_y;

        self.text_renderer.render_text(
            buffer.as_rgba_image_mut(),
            username,
            final_x,
            final_y,
            &text_style,
        )?;

        Ok(())
    }

    /// Encode image buffer to PNG bytes
    async fn encode_to_png(&self, buffer: &ImageBuffer) -> WelcomeImageResult<Vec<u8>> {
        let mut bytes = Vec::new();
        let dynamic_img = DynamicImage::ImageRgba8(buffer.as_rgba_image().clone());

        let mut cursor = Cursor::new(&mut bytes);
        dynamic_img
            .write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| WelcomeImageError::ImageProcessing(e))?;

        Ok(bytes)
    }
}
