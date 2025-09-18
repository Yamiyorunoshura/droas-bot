//! Image buffer pool implementation for efficient memory management
//!
//! Uses object pool pattern to reuse image buffers and reduce allocation overhead.

use crate::image::{
    WelcomeImageError, WelcomeImageResult, WELCOME_IMAGE_HEIGHT, WELCOME_IMAGE_WIDTH,
};
use image::{Rgba, RgbaImage};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;

/// Memory statistics for the buffer pool
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub active_buffers: usize,
    pub total_memory_bytes: usize,
}

/// Image buffer with unique ID and basic operations
pub struct ImageBuffer {
    id: Uuid,
    buffer: RgbaImage,
}

impl ImageBuffer {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            buffer: RgbaImage::new(WELCOME_IMAGE_WIDTH, WELCOME_IMAGE_HEIGHT),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn width(&self) -> u32 {
        self.buffer.width()
    }

    pub fn height(&self) -> u32 {
        self.buffer.height()
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, pixel: [u8; 4]) {
        if x < self.width() && y < self.height() {
            self.buffer.put_pixel(x, y, Rgba(pixel));
        }
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        if x < self.width() && y < self.height() {
            self.buffer.get_pixel(x, y).0
        } else {
            [0, 0, 0, 0]
        }
    }

    pub fn clear(&mut self, color: [u8; 4]) {
        // Optimized clear using fill
        let rgba = Rgba(color);
        for pixel in self.buffer.pixels_mut() {
            *pixel = rgba;
        }
    }

    /// Fill a rectangular area with a color
    pub fn fill_rect(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> WelcomeImageResult<()> {
        let end_x = (x + width).min(self.width());
        let end_y = (y + height).min(self.height());

        for py in y..end_y {
            for px in x..end_x {
                self.buffer.put_pixel(px, py, Rgba(color));
            }
        }
        Ok(())
    }

    /// Draw a circle (optimized for avatar rendering)
    pub fn draw_circle(
        &mut self,
        center_x: u32,
        center_y: u32,
        radius: u32,
        color: [u8; 4],
        antialiasing: bool,
    ) -> WelcomeImageResult<()> {
        let radius_sq = (radius * radius) as i32;
        let aa_radius_sq = if antialiasing {
            ((radius.saturating_sub(2)) * (radius.saturating_sub(2))) as i32
        } else {
            radius_sq
        };

        // Only iterate over the bounding box of the circle for efficiency
        let start_x = center_x.saturating_sub(radius);
        let end_x = (center_x + radius).min(self.width());
        let start_y = center_y.saturating_sub(radius);
        let end_y = (center_y + radius).min(self.height());

        for y in start_y..end_y {
            for x in start_x..end_x {
                let dx = x as i32 - center_x as i32;
                let dy = y as i32 - center_y as i32;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= radius_sq {
                    let alpha = if antialiasing && distance_sq > aa_radius_sq {
                        128 // Anti-aliasing on edges
                    } else {
                        color[3] // Use provided alpha
                    };

                    let pixel_color = [color[0], color[1], color[2], alpha];
                    self.buffer.put_pixel(x, y, Rgba(pixel_color));
                }
            }
        }
        Ok(())
    }

    /// Composite another buffer onto this one using alpha blending
    pub fn composite(&mut self, overlay: &ImageBuffer) -> WelcomeImageResult<()> {
        for y in 0..self.height().min(overlay.height()) {
            for x in 0..self.width().min(overlay.width()) {
                let base_pixel = self.buffer.get_pixel(x, y);
                let overlay_pixel = overlay.buffer.get_pixel(x, y);

                // Simple alpha blending
                let alpha = overlay_pixel.0[3] as f32 / 255.0;
                let inv_alpha = 1.0 - alpha;

                let blended = Rgba([
                    ((overlay_pixel.0[0] as f32 * alpha + base_pixel.0[0] as f32 * inv_alpha)
                        as u8),
                    ((overlay_pixel.0[1] as f32 * alpha + base_pixel.0[1] as f32 * inv_alpha)
                        as u8),
                    ((overlay_pixel.0[2] as f32 * alpha + base_pixel.0[2] as f32 * inv_alpha)
                        as u8),
                    ((overlay_pixel.0[3] as f32 + base_pixel.0[3] as f32 * inv_alpha) as u8)
                        .min(255),
                ]);

                self.buffer.put_pixel(x, y, blended);
            }
        }
        Ok(())
    }

    /// Get the underlying image buffer for encoding
    pub fn as_rgba_image(&self) -> &RgbaImage {
        &self.buffer
    }

    /// Get mutable access to underlying buffer for advanced operations
    pub fn as_rgba_image_mut(&mut self) -> &mut RgbaImage {
        &mut self.buffer
    }
}

/// Object pool for managing image buffers
pub struct BufferPool {
    available_buffers: Arc<Mutex<Vec<ImageBuffer>>>,
    max_pool_size: usize,
    _semaphore: Arc<Semaphore>, // For future rate limiting if needed
}

impl BufferPool {
    /// Create a new buffer pool with the specified maximum size
    pub async fn new(max_size: usize) -> WelcomeImageResult<Self> {
        let mut initial_buffers = Vec::with_capacity(max_size);

        // Pre-allocate some buffers
        for _ in 0..max_size.min(3) {
            // Start with a few buffers
            initial_buffers.push(ImageBuffer::new());
        }

        Ok(Self {
            available_buffers: Arc::new(Mutex::new(initial_buffers)),
            max_pool_size: max_size,
            _semaphore: Arc::new(Semaphore::new(max_size * 2)), // Allow some overflow
        })
    }

    /// Get a buffer from the pool, creating a new one if none are available
    pub async fn get_buffer(&self) -> WelcomeImageResult<ImageBuffer> {
        let mut buffers = self.available_buffers.lock().await;

        match buffers.pop() {
            Some(buffer) => Ok(buffer),
            None => {
                // No available buffers, create a new one
                Ok(ImageBuffer::new())
            }
        }
    }

    /// Return a buffer to the pool
    pub async fn return_buffer(&self, mut buffer: ImageBuffer) -> WelcomeImageResult<()> {
        let mut buffers = self.available_buffers.lock().await;

        // Clear the buffer before returning it to the pool
        buffer.clear([0, 0, 0, 0]);

        // Only return to pool if we haven't exceeded the limit
        if buffers.len() < self.max_pool_size {
            buffers.push(buffer);
        }
        // Otherwise, just drop the buffer (let it go out of scope)

        Ok(())
    }

    /// Get memory usage statistics
    pub async fn get_memory_stats(&self) -> WelcomeImageResult<MemoryStats> {
        let buffers = self.available_buffers.lock().await;

        let bytes_per_buffer = (WELCOME_IMAGE_WIDTH * WELCOME_IMAGE_HEIGHT * 4) as usize;

        Ok(MemoryStats {
            active_buffers: buffers.len(),
            total_memory_bytes: buffers.len() * bytes_per_buffer,
        })
    }
}
