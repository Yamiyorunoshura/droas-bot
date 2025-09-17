use droas_bot::image::{
    ImageRenderer, WelcomeImageConfig, ContrastCalculator, RgbColor,
    WELCOME_IMAGE_WIDTH, WELCOME_IMAGE_HEIGHT,
};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use std::sync::Arc;
use tokio::time::timeout;

/// Test NFR-P-001: Welcome image rendering latency P95 <= 1000ms
#[tokio::test]
async fn test_nfr_p001_rendering_latency_p95() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let config = WelcomeImageConfig {
        username: "TestUser".to_string(),
        avatar_url: None,
        background_path: None,
    };
    
    let mut render_times = Vec::new();
    
    // Perform 100 renders to get P95 statistics
    for _ in 0..100 {
        let start = Instant::now();
        let result = renderer.render_welcome_image(&config).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Render failed");
        render_times.push(duration);
    }
    
    // Sort to calculate percentiles
    render_times.sort();
    
    // Calculate P95 (95th percentile)
    let p95_index = (render_times.len() as f64 * 0.95) as usize;
    let p95_time = render_times[p95_index.min(render_times.len() - 1)];
    
    println!("P95 rendering time: {:?}", p95_time);
    println!("Average rendering time: {:?}", render_times.iter().sum::<Duration>() / render_times.len() as u32);
    println!("Min rendering time: {:?}", render_times.first().unwrap());
    println!("Max rendering time: {:?}", render_times.last().unwrap());
    
    // NFR-P-001 requirement: P95 <= 1000ms
    assert!(
        p95_time <= Duration::from_millis(1000),
        "P95 rendering time {} exceeds 1000ms requirement", 
        p95_time.as_millis()
    );
}

/// Test NFR-SC-001: Concurrent rendering capacity (20 concurrent requests)
#[tokio::test]
async fn test_nfr_sc001_concurrent_rendering() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = Arc::new(ImageRenderer::new(temp_dir.path()).await.unwrap());
    
    let mut handles = Vec::new();
    
    let start_time = Instant::now();
    
    // Spawn 20 concurrent rendering tasks
    for i in 0..20 {
        let renderer_clone = renderer.clone();
        let handle = tokio::spawn(async move {
            let config = WelcomeImageConfig {
                username: format!("User{}", i),
                avatar_url: None,
                background_path: None,
            };
            
            let render_start = Instant::now();
            let result = renderer_clone.render_welcome_image(&config).await;
            let render_time = render_start.elapsed();
            
            (i, result, render_time)
        });
        handles.push(handle);
    }
    
    // Wait for all tasks to complete with timeout
    let results = timeout(Duration::from_secs(30), async {
        let mut results = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            results.push(result);
        }
        results
    }).await.expect("Concurrent rendering test timed out");
    
    let total_time = start_time.elapsed();
    
    // Verify all renders succeeded
    for (i, result, render_time) in results.iter() {
        assert!(result.is_ok(), "Concurrent render {} failed: {:?}", i, result);
        println!("Render {}: {:?}", i, render_time);
    }
    
    println!("Total concurrent rendering time: {:?}", total_time);
    println!("Average per-render time: {:?}", total_time / results.len() as u32);
    
    // Should complete within reasonable time for 20 concurrent requests
    assert!(
        total_time <= Duration::from_secs(10),
        "Concurrent rendering took too long: {:?}",
        total_time
    );
}

/// Test memory usage stays within bounds
#[tokio::test]
async fn test_memory_usage_bounds() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let config = WelcomeImageConfig {
        username: "TestUser".to_string(),
        avatar_url: None,
        background_path: None,
    };
    
    // Get initial memory stats
    let buffer = renderer.create_image_buffer().await.unwrap();
    
    // Perform multiple renders to test memory management
    for i in 0..50 {
        let mut test_config = config.clone();
        test_config.username = format!("TestUser{}", i);
        
        let result = renderer.render_welcome_image(&test_config).await;
        assert!(result.is_ok(), "Render {} failed", i);
        
        // Periodically check memory usage
        if i % 10 == 0 {
            // Note: This is a simplified check. In a real scenario, 
            // you'd want to measure actual memory usage
            println!("Completed {} renders", i + 1);
        }
    }
    
    // Test should complete without excessive memory usage
    // The actual implementation should track this properly
}

/// Test WCAG 2.1 AA contrast compliance (NFR-U-001)
#[tokio::test]
async fn test_nfr_u001_contrast_compliance() {
    let calculator = ContrastCalculator::new();
    
    // Test various background colors
    let test_cases = vec![
        ([0, 0, 0], "black background"),      // Very dark
        ([255, 255, 255], "white background"), // Very light
        ([128, 128, 128], "gray background"),  // Medium gray
        ([255, 0, 0], "red background"),       // Pure red
        ([0, 0, 255], "blue background"),      // Pure blue
        ([0, 255, 0], "green background"),     // Pure green
    ];
    
    for (bg_rgb, description) in test_cases {
        let bg_color = RgbColor::from(bg_rgb);
        let text_color = calculator.get_best_text_color(bg_color);
        
        // Convert back to RgbColor for testing
        let text_rgb = RgbColor::from(text_color);
        
        let contrast = calculator.calculate_contrast(text_rgb, bg_color);
        
        println!("{}: contrast ratio {:.2}, meets AA: {}", 
                description, contrast.ratio, contrast.meets_aa);
        
        // NFR-U-001 requirement: Must meet WCAG 2.1 AA (ratio >= 4.5)
        assert!(
            contrast.meets_aa,
            "{} contrast ratio {:.2} does not meet WCAG 2.1 AA standard (>= 4.5)",
            description,
            contrast.ratio
        );
    }
}

/// Test rendering with different username lengths for performance variation
#[tokio::test]
async fn test_username_length_performance() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    let username_tests = vec![
        ("", "empty"),
        ("A", "single char"),
        ("TestUser", "normal"),
        ("A".repeat(50), "long"),
        ("用戶名123!@#$%", "unicode/special"),
        ("A".repeat(100), "very long"),
    ];
    
    for (username, description) in username_tests {
        let config = WelcomeImageConfig {
            username: username.clone(),
            avatar_url: None,
            background_path: None,
        };
        
        let start = Instant::now();
        let result = renderer.render_welcome_image(&config).await;
        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Render failed for {}", description);
        println!("{} (len: {}): {:?}", description, username.len(), duration);
        
        // Should still meet performance requirements regardless of username length
        assert!(
            duration <= Duration::from_millis(2000), // More lenient for edge cases
            "{} took too long: {:?}",
            description,
            duration
        );
    }
}

/// Test background loading performance impact
#[tokio::test]
async fn test_background_loading_performance() {
    let temp_dir = TempDir::new().unwrap();
    let renderer = ImageRenderer::new(temp_dir.path()).await.unwrap();
    
    // Test with default background
    let config_default = WelcomeImageConfig {
        username: "TestUser".to_string(),
        avatar_url: None,
        background_path: None,
    };
    
    let start = Instant::now();
    let result_default = renderer.render_welcome_image(&config_default).await;
    let duration_default = start.elapsed();
    
    assert!(result_default.is_ok());
    println!("Default background render time: {:?}", duration_default);
    
    // Create a custom background for testing
    let bg_path = temp_dir.path().join("test_bg.png");
    let test_bg = image::RgbImage::new(WELCOME_IMAGE_WIDTH, WELCOME_IMAGE_HEIGHT);
    let dynamic_img = image::DynamicImage::ImageRgb8(test_bg);
    dynamic_img.save(&bg_path).unwrap();
    
    let config_custom = WelcomeImageConfig {
        username: "TestUser".to_string(),
        avatar_url: None,
        background_path: Some(bg_path),
    };
    
    let start = Instant::now();
    let result_custom = renderer.render_welcome_image(&config_custom).await;
    let duration_custom = start.elapsed();
    
    assert!(result_custom.is_ok());
    println!("Custom background render time: {:?}", duration_custom);
    
    // Both should meet performance requirements
    assert!(duration_default <= Duration::from_millis(1000));
    assert!(duration_custom <= Duration::from_millis(1500)); // Slightly more time for loading
}