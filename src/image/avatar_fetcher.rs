//! Avatar fetching functionality for Discord avatars
//! 
//! This module handles fetching user avatars from Discord CDN with caching and error handling.

use crate::image::WelcomeImageResult;
use image::{DynamicImage, ImageFormat};
use reqwest::Client;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Error types for avatar fetching
#[derive(Debug, thiserror::Error)]
pub enum AvatarFetchError {
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("Image decode error: {0}")]
    ImageDecode(String),
    
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Invalid image data")]
    InvalidImageData,
}

/// Cached avatar entry
#[derive(Clone)]
struct CachedAvatar {
    image: DynamicImage,
    fetched_at: Instant,
}

/// Avatar fetcher for Discord CDN with LRU cache
pub struct AvatarFetcher {
    client: Client,
    cache: Mutex<HashMap<String, CachedAvatar>>,
    cache_duration: Duration,
    max_cache_size: usize,
}

impl AvatarFetcher {
    /// Create a new avatar fetcher with caching
    pub async fn new() -> WelcomeImageResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("DROAS-bot/1.0")
            .build()
            .map_err(|e| AvatarFetchError::Network(format!("Failed to create HTTP client: {}", e)))?;
            
        Ok(Self {
            client,
            cache: Mutex::new(HashMap::new()),
            cache_duration: Duration::from_secs(300), // 5 minutes cache
            max_cache_size: 100,
        })
    }
    
    /// Fetch avatar from Discord CDN URL
    pub async fn fetch_avatar(&self, avatar_url: &str) -> Result<DynamicImage, AvatarFetchError> {
        // Check cache first
        if let Some(cached) = self.get_cached_avatar(avatar_url).await {
            return Ok(cached);
        }
        
        // Validate URL format
        if !avatar_url.starts_with("https://") {
            return Err(AvatarFetchError::InvalidUrl("Avatar URL must use HTTPS".to_string()));
        }
        
        // Fetch avatar from Discord CDN
        let response = self.client
            .get(avatar_url)
            .send()
            .await?
            .error_for_status()?;
            
        let content_type = response.headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("image/png");
            
        // Validate content type
        if !content_type.starts_with("image/") {
            return Err(AvatarFetchError::InvalidImageData);
        }
        
        let bytes = response.bytes().await?;
        
        // Decode image
        let image = image::load_from_memory(&bytes)
            .map_err(|e| AvatarFetchError::ImageDecode(format!("Failed to decode image: {}", e)))?;
            
        // Cache the image
        self.cache_avatar(avatar_url.to_string(), image.clone()).await;
        
        Ok(image)
    }
    
    /// Get avatar from cache if available and not expired
    async fn get_cached_avatar(&self, url: &str) -> Option<DynamicImage> {
        let cache = self.cache.lock().await;
        if let Some(cached) = cache.get(url) {
            if cached.fetched_at.elapsed() < self.cache_duration {
                return Some(cached.image.clone());
            }
        }
        None
    }
    
    /// Cache avatar image with LRU eviction
    async fn cache_avatar(&self, url: String, image: DynamicImage) {
        let mut cache = self.cache.lock().await;
        
        // Evict old entries if cache is full
        if cache.len() >= self.max_cache_size {
            // Find oldest entry to evict
            let oldest_key = cache.iter()
                .min_by_key(|(_, cached)| cached.fetched_at)
                .map(|(k, _)| k.clone());
                
            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }
        
        cache.insert(url, CachedAvatar {
            image,
            fetched_at: Instant::now(),
        });
    }
    
    /// Clear expired cache entries
    pub async fn cleanup_cache(&self) {
        let mut cache = self.cache.lock().await;
        cache.retain(|_, cached| cached.fetched_at.elapsed() < self.cache_duration);
    }
    
    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().await;
        let total = cache.len();
        let valid = cache.values()
            .filter(|cached| cached.fetched_at.elapsed() < self.cache_duration)
            .count();
        (total, valid)
    }
}
