use droas_bot::image::{BufferPool, ImageBuffer, WELCOME_IMAGE_HEIGHT, WELCOME_IMAGE_WIDTH};
use tokio_test;

#[tokio::test]
async fn test_buffer_pool_creation() {
    let pool = BufferPool::new(5).await; // Pool with 5 buffers
    assert!(pool.is_ok());
}

#[tokio::test]
async fn test_buffer_pool_get_buffer() {
    let pool = BufferPool::new(3).await.unwrap();

    let buffer = pool.get_buffer().await;
    assert!(buffer.is_ok());

    let img_buffer = buffer.unwrap();
    assert_eq!(img_buffer.width(), WELCOME_IMAGE_WIDTH);
    assert_eq!(img_buffer.height(), WELCOME_IMAGE_HEIGHT);
}

#[tokio::test]
async fn test_buffer_pool_return_buffer() {
    let pool = BufferPool::new(3).await.unwrap();

    let buffer = pool.get_buffer().await.unwrap();
    let buffer_id = buffer.id(); // Assuming buffers have IDs for tracking

    let return_result = pool.return_buffer(buffer).await;
    assert!(return_result.is_ok());

    // Get another buffer and check if we got the same one back (reuse)
    let reused_buffer = pool.get_buffer().await.unwrap();
    assert_eq!(reused_buffer.id(), buffer_id);
}

#[tokio::test]
async fn test_buffer_pool_exhaust_and_create_new() {
    let pool = BufferPool::new(2).await.unwrap(); // Small pool

    // Get all buffers from pool
    let buffer1 = pool.get_buffer().await.unwrap();
    let buffer2 = pool.get_buffer().await.unwrap();

    // Pool should be empty now, but should create new buffer if needed
    let buffer3 = pool.get_buffer().await.unwrap();
    assert!(buffer3.id() != buffer1.id() && buffer3.id() != buffer2.id());
}

#[tokio::test]
async fn test_buffer_pool_concurrent_access() {
    let pool = std::sync::Arc::new(BufferPool::new(3).await.unwrap());
    let mut handles = vec![];

    // Spawn multiple tasks that concurrently get and return buffers
    for i in 0..10 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let buffer = pool_clone.get_buffer().await.unwrap();

            // Do some "work" with the buffer
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;

            pool_clone.return_buffer(buffer).await.unwrap();
            i
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_image_buffer_basic_operations() {
    let pool = BufferPool::new(1).await.unwrap();
    let mut buffer = pool.get_buffer().await.unwrap();

    // Test basic pixel operations
    buffer.set_pixel(0, 0, [255, 0, 0, 255]); // Red pixel
    let pixel = buffer.get_pixel(0, 0);
    assert_eq!(pixel, [255, 0, 0, 255]);

    // Test clearing buffer
    buffer.clear([0, 0, 0, 0]); // Clear to transparent
    let cleared_pixel = buffer.get_pixel(0, 0);
    assert_eq!(cleared_pixel, [0, 0, 0, 0]);
}

#[tokio::test]
async fn test_image_buffer_layer_composition() {
    let pool = BufferPool::new(2).await.unwrap();
    let mut base_buffer = pool.get_buffer().await.unwrap();
    let mut overlay_buffer = pool.get_buffer().await.unwrap();

    // Create base layer (blue background)
    base_buffer.clear([0, 0, 255, 255]);

    // Create overlay layer (semi-transparent red)
    overlay_buffer.clear([255, 0, 0, 128]);

    // Composite overlay onto base
    let composite_result = base_buffer.composite(&overlay_buffer);
    assert!(composite_result.is_ok());

    // Check that composition worked (should be purple-ish)
    let result_pixel = base_buffer.get_pixel(100, 100);
    // Simple check - should not be pure blue anymore
    assert_ne!(result_pixel, [0, 0, 255, 255]);
}

#[tokio::test]
async fn test_buffer_pool_memory_management() {
    let pool = BufferPool::new(5).await.unwrap();

    // Get memory usage before
    let initial_stats = pool.get_memory_stats().await.unwrap();

    // Allocate and return many buffers
    for _ in 0..20 {
        let buffer = pool.get_buffer().await.unwrap();
        pool.return_buffer(buffer).await.unwrap();
    }

    // Check memory stats after
    let final_stats = pool.get_memory_stats().await.unwrap();

    // Pool should maintain reasonable memory usage
    assert!(final_stats.active_buffers <= 5);
    assert!(final_stats.total_memory_bytes < 100 * 1024 * 1024); // Less than 100MB
}

#[tokio::test]
async fn test_buffer_pool_performance() {
    let pool = BufferPool::new(10).await.unwrap();

    let start = std::time::Instant::now();

    // Perform many get/return cycles
    for _ in 0..100 {
        let buffer = pool.get_buffer().await.unwrap();
        pool.return_buffer(buffer).await.unwrap();
    }

    let duration = start.elapsed();

    // Should complete quickly (buffer reuse should be fast)
    assert!(
        duration.as_millis() < 1000,
        "Buffer operations took too long: {:?}",
        duration
    );
}
