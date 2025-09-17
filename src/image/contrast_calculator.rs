//! Color contrast calculation for accessibility
//! 
//! Implements WCAG 2.1 AA contrast ratio calculations.

use crate::image::WelcomeImageResult;

/// Color contrast information
#[derive(Debug, Clone)]
pub struct ColorContrast {
    pub ratio: f64,
    pub meets_aa: bool,
    pub meets_aaa: bool,
    pub recommended_text_color: [u8; 4],
}

/// RGB color for calculations
#[derive(Debug, Clone, Copy)]
pub struct RgbColor {
    pub r: f64, // 0.0 - 1.0
    pub g: f64,
    pub b: f64,
}

impl From<[u8; 3]> for RgbColor {
    fn from(rgb: [u8; 3]) -> Self {
        Self {
            r: rgb[0] as f64 / 255.0,
            g: rgb[1] as f64 / 255.0,
            b: rgb[2] as f64 / 255.0,
        }
    }
}

impl From<[u8; 4]> for RgbColor {
    fn from(rgba: [u8; 4]) -> Self {
        Self {
            r: rgba[0] as f64 / 255.0,
            g: rgba[1] as f64 / 255.0,
            b: rgba[2] as f64 / 255.0,
        }
    }
}

/// Contrast calculator for determining text colors
pub struct ContrastCalculator;

impl ContrastCalculator {
    /// Create a new contrast calculator
    pub fn new() -> Self {
        Self
    }
    
    /// Calculate the relative luminance of a color according to WCAG 2.1
    /// https://www.w3.org/TR/WCAG21/#dfn-relative-luminance
    pub fn relative_luminance(color: RgbColor) -> f64 {
        fn linearize(c: f64) -> f64 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        
        let r_linear = linearize(color.r);
        let g_linear = linearize(color.g);
        let b_linear = linearize(color.b);
        
        0.2126 * r_linear + 0.7152 * g_linear + 0.0722 * b_linear
    }
    
    /// Calculate contrast ratio between two colors according to WCAG 2.1
    /// https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio
    pub fn contrast_ratio(color1: RgbColor, color2: RgbColor) -> f64 {
        let l1 = Self::relative_luminance(color1);
        let l2 = Self::relative_luminance(color2);
        
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        
        (lighter + 0.05) / (darker + 0.05)
    }
    
    /// Calculate contrast information for text on background
    pub fn calculate_contrast(&self, text_color: RgbColor, background_color: RgbColor) -> ColorContrast {
        let ratio = Self::contrast_ratio(text_color, background_color);
        
        ColorContrast {
            ratio,
            meets_aa: ratio >= 4.5,   // WCAG 2.1 AA standard
            meets_aaa: ratio >= 7.0,  // WCAG 2.1 AAA standard
            recommended_text_color: if ratio >= 4.5 {
                // Current text color is fine
                [(text_color.r * 255.0) as u8, (text_color.g * 255.0) as u8, (text_color.b * 255.0) as u8, 255]
            } else {
                // Recommend better contrast color
                self.get_best_text_color(background_color)
            },
        }
    }
    
    /// Get the best text color (black or white) for given background
    pub fn get_best_text_color(&self, background_color: RgbColor) -> [u8; 4] {
        let white = RgbColor { r: 1.0, g: 1.0, b: 1.0 };
        let black = RgbColor { r: 0.0, g: 0.0, b: 0.0 };
        
        let white_contrast = Self::contrast_ratio(white, background_color);
        let black_contrast = Self::contrast_ratio(black, background_color);
        
        if white_contrast > black_contrast {
            [255, 255, 255, 255] // White text
        } else {
            [0, 0, 0, 255]       // Black text
        }
    }
    
    /// Sample background color from an image area to determine text placement
    pub fn sample_background_luminance(&self, image_buffer: &crate::image::ImageBuffer, 
                                      x: u32, y: u32, sample_size: u32) -> WelcomeImageResult<f64> {
        let mut total_luminance = 0.0;
        let mut sample_count = 0;
        
        let start_x = x.saturating_sub(sample_size / 2);
        let end_x = (x + sample_size / 2).min(image_buffer.width());
        let start_y = y.saturating_sub(sample_size / 2);
        let end_y = (y + sample_size / 2).min(image_buffer.height());
        
        for sample_y in start_y..end_y {
            for sample_x in start_x..end_x {
                let pixel = image_buffer.get_pixel(sample_x, sample_y);
                let color = RgbColor::from([pixel[0], pixel[1], pixel[2]]);
                total_luminance += Self::relative_luminance(color);
                sample_count += 1;
            }
        }
        
        if sample_count > 0 {
            Ok(total_luminance / sample_count as f64)
        } else {
            Ok(0.0) // Fallback
        }
    }
    
    /// Get text color with optimal contrast for specific area of image
    pub fn get_optimal_text_color(&self, image_buffer: &crate::image::ImageBuffer, 
                                 text_x: u32, text_y: u32, text_width: u32, text_height: u32) -> WelcomeImageResult<[u8; 4]> {
        // Sample the background where text will be placed
        let sample_size = 20; // Sample 20x20 area
        let center_x = text_x + text_width / 2;
        let center_y = text_y + text_height / 2;
        
        let avg_luminance = self.sample_background_luminance(image_buffer, center_x, center_y, sample_size)?;
        
        // Convert average luminance back to approximate RGB for contrast calculation
        let bg_approx = RgbColor { r: avg_luminance, g: avg_luminance, b: avg_luminance };
        
        Ok(self.get_best_text_color(bg_approx))
    }
    
    /// Calculate average luminance for a rectangular area
    pub fn calculate_average_luminance(
        &self,
        image_buffer: &crate::image::ImageBuffer,
        x: u32,
        y: u32,
        width: u32,
        height: u32
    ) -> WelcomeImageResult<f32> {
        let mut total_luminance = 0.0f64;
        let mut pixel_count = 0;
        
        let end_x = (x + width).min(image_buffer.width());
        let end_y = (y + height).min(image_buffer.height());
        
        for sample_y in y..end_y {
            for sample_x in x..end_x {
                let pixel = image_buffer.get_pixel(sample_x, sample_y);
                let color = RgbColor::from([pixel[0], pixel[1], pixel[2]]);
                total_luminance += Self::relative_luminance(color);
                pixel_count += 1;
            }
        }
        
        Ok(if pixel_count > 0 {
            (total_luminance / pixel_count as f64) as f32
        } else {
            0.0
        })
    }
}

impl Default for ContrastCalculator {
    fn default() -> Self {
        Self::new()
    }
}
