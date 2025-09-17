use droas_bot::image::{
    ImageRenderer, WelcomeImageConfig, WelcomeImageError,
    WELCOME_IMAGE_WIDTH, WELCOME_IMAGE_HEIGHT,
};
use tempfile::TempDir;
use tokio_test;

#[tokio::test]
async fn test_image_renderer_creation() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await;
    assert!(renderer.is_ok());
}

#[tokio::test]
async fn test_create_welcome_image_buffer() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let buffer = renderer.create_image_buffer().await;
    assert!(buffer.is_ok());
    
    let img_buffer = buffer.unwrap();
    assert_eq!(img_buffer.width(), WELCOME_IMAGE_WIDTH);
    assert_eq!(img_buffer.height(), WELCOME_IMAGE_HEIGHT);
}

#[tokio::test]
async fn test_render_welcome_image_with_default_config() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let config = WelcomeImageConfig {
        username: "TestUser".to_string(),
        avatar_url: None, // Test default avatar
        background_path: None, // Test default background
    };
    
    let result = renderer.render_welcome_image(&config).await;
    assert!(result.is_ok());
    
    let image_data = result.unwrap();
    assert!(!image_data.is_empty());
    
    // Verify it's a valid PNG image by trying to decode it
    let img = image::load_from_memory(&image_data);
    assert!(img.is_ok());
    let decoded = img.unwrap();
    assert_eq!(decoded.width(), WELCOME_IMAGE_WIDTH);
    assert_eq!(decoded.height(), WELCOME_IMAGE_HEIGHT);
}

#[tokio::test]
async fn test_render_welcome_image_with_custom_background() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    // Create a simple test background image
    let bg_path = temp_dir.path().join("test_bg.png");
    let test_bg = image::RgbImage::new(WELCOME_IMAGE_WIDTH, WELCOME_IMAGE_HEIGHT);
    let dynamic_img = image::DynamicImage::ImageRgb8(test_bg);
    dynamic_img.save(&bg_path).unwrap();
    
    let config = WelcomeImageConfig {
        username: "TestUser".to_string(),
        avatar_url: None,
        background_path: Some(bg_path),
    };
    
    let result = renderer.render_welcome_image(&config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_render_welcome_image_with_avatar_url() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let config = WelcomeImageConfig {
        username: "TestUser".to_string(),
        avatar_url: Some("https://cdn.discordapp.com/avatars/user/avatar.png".to_string()),
        background_path: None,
    };
    
    // This test might fail due to network, but should handle gracefully
    let result = renderer.render_welcome_image(&config).await;
    // Should either succeed or fail gracefully with avatar fetch error
    match result {
        Ok(_) => {}, // Success case
        Err(WelcomeImageError::AvatarFetch(_)) => {}, // Expected failure case
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[tokio::test]
async fn test_render_empty_username_handling() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let config = WelcomeImageConfig {
        username: "".to_string(), // Empty username
        avatar_url: None,
        background_path: None,
    };
    
    let result = renderer.render_welcome_image(&config).await;
    // Should handle empty username gracefully
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_render_long_username_handling() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let config = WelcomeImageConfig {
        username: "A".repeat(100), // Very long username
        avatar_url: None,
        background_path: None,
    };
    
    let result = renderer.render_welcome_image(&config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_render_username_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let config = WelcomeImageConfig {
        username: "用戶名123!@#$%".to_string(), // Unicode and special chars
        avatar_url: None,
        background_path: None,
    };
    
    let result = renderer.render_welcome_image(&config).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_performance_rendering_multiple_images() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let config = WelcomeImageConfig {
        username: "TestUser".to_string(),
        avatar_url: None,
        background_path: None,
    };
    
    let start = std::time::Instant::now();
    
    // Render 5 images to test basic performance
    for i in 0..5 {
        let mut test_config = config.clone();
        test_config.username = format!("TestUser{}", i);
        let result = renderer.render_welcome_image(&test_config).await;
        assert!(result.is_ok(), "Failed to render image {}", i);
    }
    
    let duration = start.elapsed();
    
    // Should complete 5 images in reasonable time (adjust based on requirements)
    // For now, just ensure it doesn't take more than 30 seconds
    assert!(duration.as_secs() < 30, "Rendering took too long: {:?}", duration);
}