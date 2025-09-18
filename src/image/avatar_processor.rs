//! Avatar processing functionality
//!
//! Handles circular masking, anti-aliasing, and avatar transformations.

use crate::image::{WelcomeImageError, WelcomeImageResult};
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::draw_filled_circle_mut;

/// Processed avatar with circular mask and positioning info
#[derive(Debug, Clone)]
pub struct ProcessedAvatar {
    pub image: RgbaImage,
    pub diameter: u32,
    pub center_x: u32,
    pub center_y: u32,
}

/// Avatar processor for applying circular masks and transformations
pub struct AvatarProcessor {
    default_size: u32,
    anti_alias_samples: u32,
}

impl AvatarProcessor {
    /// Create a new avatar processor
    pub fn new(default_size: u32) -> Self {
        Self {
            default_size,
            anti_alias_samples: 4, // 4x supersampling for anti-aliasing
        }
    }

    /// Process avatar into circular format with anti-aliasing
    pub fn process_avatar(
        &self,
        avatar_image: &DynamicImage,
        target_diameter: Option<u32>,
    ) -> WelcomeImageResult<ProcessedAvatar> {
        let diameter = target_diameter.unwrap_or(self.default_size);

        // Resize avatar to square format
        let square_avatar =
            avatar_image.resize_exact(diameter, diameter, image::imageops::FilterType::Lanczos3);

        let rgba_avatar = square_avatar.to_rgba8();

        // Create circular mask with anti-aliasing
        let circular_avatar = self.apply_circular_mask(&rgba_avatar, diameter)?;

        Ok(ProcessedAvatar {
            image: circular_avatar,
            diameter,
            center_x: diameter / 2,
            center_y: diameter / 2,
        })
    }

    /// Apply circular mask with anti-aliasing
    fn apply_circular_mask(
        &self,
        image: &RgbaImage,
        diameter: u32,
    ) -> WelcomeImageResult<RgbaImage> {
        let mut result = RgbaImage::new(diameter, diameter);
        let radius = diameter as f32 / 2.0;
        let center = radius;

        // Apply circular mask with anti-aliasing
        for y in 0..diameter {
            for x in 0..diameter {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance <= radius {
                    // Calculate anti-aliased alpha
                    let alpha_factor = if distance > radius - 1.0 {
                        // Anti-aliasing at edge
                        (radius - distance).max(0.0)
                    } else {
                        1.0
                    };

                    let source_pixel = image.get_pixel(x, y);
                    let mut new_pixel = *source_pixel;

                    // Apply anti-aliasing to alpha channel
                    let original_alpha = source_pixel[3] as f32 / 255.0;
                    let final_alpha = (original_alpha * alpha_factor * 255.0) as u8;
                    new_pixel[3] = final_alpha;

                    result.put_pixel(x, y, new_pixel);
                } else {
                    // Outside circle - transparent
                    result.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                }
            }
        }

        Ok(result)
    }

    /// Create a default avatar placeholder
    pub fn create_default_avatar(
        &self,
        diameter: u32,
        bg_color: Rgba<u8>,
    ) -> WelcomeImageResult<ProcessedAvatar> {
        let mut avatar = RgbaImage::from_pixel(diameter, diameter, Rgba([0, 0, 0, 0]));

        // Draw background circle
        let radius = diameter as i32 / 2;
        let center = radius;

        draw_filled_circle_mut(&mut avatar, (center, center), radius, bg_color);

        // Add simple user icon (simplified representation)
        let icon_size = radius / 2;
        let head_radius = icon_size / 3;
        let body_width = icon_size;
        let body_height = icon_size * 2 / 3;

        let icon_color = Rgba([255, 255, 255, 255]);

        // Draw head
        draw_filled_circle_mut(
            &mut avatar,
            (center, center - icon_size / 2),
            head_radius,
            icon_color,
        );

        // Draw body (approximated as circle)
        draw_filled_circle_mut(
            &mut avatar,
            (center, center + icon_size / 3),
            body_width / 2,
            icon_color,
        );

        Ok(ProcessedAvatar {
            image: avatar,
            diameter,
            center_x: diameter / 2,
            center_y: diameter / 2,
        })
    }

    /// Blend processed avatar onto a target image at specified position
    pub fn blend_avatar_onto(
        &self,
        target: &mut RgbaImage,
        avatar: &ProcessedAvatar,
        pos_x: u32,
        pos_y: u32,
    ) -> WelcomeImageResult<()> {
        let start_x = pos_x.saturating_sub(avatar.center_x);
        let start_y = pos_y.saturating_sub(avatar.center_y);

        for y in 0..avatar.diameter {
            for x in 0..avatar.diameter {
                let target_x = start_x + x;
                let target_y = start_y + y;

                // Check bounds
                if target_x >= target.width() || target_y >= target.height() {
                    continue;
                }

                let avatar_pixel = avatar.image.get_pixel(x, y);

                // Only blend if avatar pixel is not fully transparent
                if avatar_pixel[3] > 0 {
                    let target_pixel = target.get_pixel_mut(target_x, target_y);

                    // Alpha blending
                    let alpha = avatar_pixel[3] as f32 / 255.0;
                    let inv_alpha = 1.0 - alpha;

                    target_pixel[0] =
                        (avatar_pixel[0] as f32 * alpha + target_pixel[0] as f32 * inv_alpha) as u8;
                    target_pixel[1] =
                        (avatar_pixel[1] as f32 * alpha + target_pixel[1] as f32 * inv_alpha) as u8;
                    target_pixel[2] =
                        (avatar_pixel[2] as f32 * alpha + target_pixel[2] as f32 * inv_alpha) as u8;
                    target_pixel[3] = ((avatar_pixel[3] as f32
                        + target_pixel[3] as f32 * inv_alpha)
                        .min(255.0)) as u8;
                }
            }
        }

        Ok(())
    }

    /// Get recommended avatar size for given image dimensions
    pub fn get_recommended_size(image_width: u32, image_height: u32) -> u32 {
        let min_dimension = image_width.min(image_height);
        (min_dimension / 6).max(64).min(256) // Between 64-256px
    }
}
