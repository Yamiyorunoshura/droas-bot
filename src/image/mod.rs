//! Image processing module for welcome image generation
//! 
//! This module provides functionality for generating personalized welcome images
//! including avatar processing, text rendering, and image composition.

pub mod renderer;
pub mod buffer_pool;
pub mod avatar_fetcher;
pub mod avatar_processor;
pub mod text_renderer;
pub mod contrast_calculator;

// Re-export main types for easier usage
pub use renderer::{ImageRenderer, WelcomeImageConfig};
pub use buffer_pool::{BufferPool, ImageBuffer};
pub use avatar_fetcher::{AvatarFetcher, AvatarFetchError};
pub use avatar_processor::{AvatarProcessor, ProcessedAvatar};
pub use text_renderer::{TextRenderer, TextStyle};
pub use contrast_calculator::{ContrastCalculator, ColorContrast};

/// Standard welcome image dimensions as specified in F-002
pub const WELCOME_IMAGE_WIDTH: u32 = 1024;
pub const WELCOME_IMAGE_HEIGHT: u32 = 512;

/// Welcome image generation result
pub type WelcomeImageResult<T> = Result<T, WelcomeImageError>;

/// Errors that can occur during welcome image generation
#[derive(Debug, thiserror::Error)]
pub enum WelcomeImageError {
    #[error("Image processing error: {0}")]
    ImageProcessing(#[from] image::ImageError),
    
    #[error("Avatar fetch error: {0}")]
    AvatarFetch(#[from] AvatarFetchError),
    
    #[error("Text rendering error: {0}")]
    TextRendering(String),
    
    #[error("Buffer pool error: {0}")]
    BufferPool(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}