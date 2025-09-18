//! Text rendering functionality
//!
//! Handles font loading, text rendering with proper contrast and WCAG compliance.

use crate::image::{WelcomeImageError, WelcomeImageResult};
use image::{Rgba, RgbaImage};
use once_cell::sync::Lazy;
use rusttype::{point, Font, PositionedGlyph, Scale};
use std::collections::HashMap;

/// Text style configuration
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_size: f32,
    pub color: Rgba<u8>,
    pub outline_color: Option<Rgba<u8>>,
    pub outline_width: f32,
    pub letter_spacing: f32,
    pub line_height: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 24.0,
            color: Rgba([255, 255, 255, 255]),
            outline_color: Some(Rgba([0, 0, 0, 255])),
            outline_width: 1.0,
            letter_spacing: 0.0,
            line_height: 1.2,
        }
    }
}

/// Text metrics for positioning
#[derive(Debug, Clone)]
pub struct TextMetrics {
    pub width: u32,
    pub height: u32,
    pub baseline: f32,
    pub ascent: f32,
    pub descent: f32,
}

/// Default font data (embedded for reliability)
static DEFAULT_FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/NotoSans-Regular.ttf");

/// Global font cache
static FONT_CACHE: Lazy<Font<'static>> =
    Lazy::new(|| Font::try_from_bytes(DEFAULT_FONT_DATA).expect("Failed to load default font"));

/// Text renderer for drawing text on images
pub struct TextRenderer {
    font: &'static Font<'static>,
    glyph_cache: HashMap<(char, u32), PositionedGlyph<'static>>, // Simple glyph cache
}

impl TextRenderer {
    /// Create a new text renderer with default font
    pub fn new() -> WelcomeImageResult<Self> {
        Ok(Self {
            font: &*FONT_CACHE,
            glyph_cache: HashMap::new(),
        })
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str, style: &TextStyle) -> TextMetrics {
        let scale = Scale::uniform(style.font_size);
        let v_metrics = self.font.v_metrics(scale);

        let glyphs: Vec<_> = self
            .font
            .layout(text, scale, point(0.0, v_metrics.ascent))
            .collect();

        let width = glyphs
            .iter()
            .rev()
            .find_map(|g| g.pixel_bounding_box().map(|bb| bb.max.x))
            .unwrap_or(0) as u32;

        let height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;

        TextMetrics {
            width: width + (style.outline_width * 2.0) as u32,
            height: height + (style.outline_width * 2.0) as u32,
            baseline: v_metrics.ascent,
            ascent: v_metrics.ascent,
            descent: v_metrics.descent,
        }
    }

    /// Render text onto an image buffer with anti-aliasing
    pub fn render_text(
        &mut self,
        target: &mut RgbaImage,
        text: &str,
        x: u32,
        y: u32,
        style: &TextStyle,
    ) -> WelcomeImageResult<TextMetrics> {
        let metrics = self.measure_text(text, style);
        let scale = Scale::uniform(style.font_size);
        let v_metrics = self.font.v_metrics(scale);

        // Adjust starting position for outline
        let start_x = x as f32 + style.outline_width;
        let start_y = y as f32 + v_metrics.ascent + style.outline_width;

        let glyphs: Vec<_> = self
            .font
            .layout(text, scale, point(start_x, start_y))
            .collect();

        // Render outline first (if specified)
        if let Some(outline_color) = style.outline_color {
            if style.outline_width > 0.0 {
                self.render_glyphs_with_outline(
                    target,
                    &glyphs,
                    outline_color,
                    style.outline_width,
                )?;
            }
        }

        // Render main text
        self.render_glyphs(target, &glyphs, style.color)?;

        Ok(metrics)
    }

    /// Render glyphs with outline effect
    fn render_glyphs_with_outline(
        &self,
        target: &mut RgbaImage,
        glyphs: &[PositionedGlyph<'static>],
        outline_color: Rgba<u8>,
        outline_width: f32,
    ) -> WelcomeImageResult<()> {
        // Simple outline by rendering text in multiple positions
        let offsets = [
            (-outline_width, -outline_width),
            (0.0, -outline_width),
            (outline_width, -outline_width),
            (-outline_width, 0.0),
            (outline_width, 0.0),
            (-outline_width, outline_width),
            (0.0, outline_width),
            (outline_width, outline_width),
        ];

        for (dx, dy) in offsets.iter() {
            for glyph in glyphs {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    glyph.draw(|x, y, v| {
                        let px = (x as i32 + bounding_box.min.x + *dx as i32) as u32;
                        let py = (y as i32 + bounding_box.min.y + *dy as i32) as u32;

                        if px < target.width() && py < target.height() {
                            let alpha = (v * outline_color[3] as f32 / 255.0) as u8;
                            if alpha > 0 {
                                self.blend_pixel(
                                    target,
                                    px,
                                    py,
                                    Rgba([
                                        outline_color[0],
                                        outline_color[1],
                                        outline_color[2],
                                        alpha,
                                    ]),
                                );
                            }
                        }
                    });
                }
            }
        }

        Ok(())
    }

    /// Render glyphs without outline
    fn render_glyphs(
        &self,
        target: &mut RgbaImage,
        glyphs: &[PositionedGlyph<'static>],
        color: Rgba<u8>,
    ) -> WelcomeImageResult<()> {
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|x, y, v| {
                    let px = (x as i32 + bounding_box.min.x) as u32;
                    let py = (y as i32 + bounding_box.min.y) as u32;

                    if px < target.width() && py < target.height() {
                        let alpha = (v * color[3] as f32 / 255.0) as u8;
                        if alpha > 0 {
                            self.blend_pixel(
                                target,
                                px,
                                py,
                                Rgba([color[0], color[1], color[2], alpha]),
                            );
                        }
                    }
                });
            }
        }

        Ok(())
    }

    /// Alpha blend a pixel onto the target image
    fn blend_pixel(&self, target: &mut RgbaImage, x: u32, y: u32, pixel: Rgba<u8>) {
        let target_pixel = target.get_pixel_mut(x, y);

        let src_alpha = pixel[3] as f32 / 255.0;
        let dst_alpha = target_pixel[3] as f32 / 255.0;
        let inv_src_alpha = 1.0 - src_alpha;

        // Alpha blending formula
        let final_alpha = src_alpha + dst_alpha * inv_src_alpha;

        if final_alpha > 0.0 {
            target_pixel[0] = ((pixel[0] as f32 * src_alpha
                + target_pixel[0] as f32 * dst_alpha * inv_src_alpha)
                / final_alpha) as u8;
            target_pixel[1] = ((pixel[1] as f32 * src_alpha
                + target_pixel[1] as f32 * dst_alpha * inv_src_alpha)
                / final_alpha) as u8;
            target_pixel[2] = ((pixel[2] as f32 * src_alpha
                + target_pixel[2] as f32 * dst_alpha * inv_src_alpha)
                / final_alpha) as u8;
            target_pixel[3] = (final_alpha * 255.0) as u8;
        }
    }

    /// Calculate optimal text positioning for centering
    pub fn center_text_position(
        &self,
        text: &str,
        style: &TextStyle,
        container_width: u32,
        container_height: u32,
    ) -> (u32, u32) {
        let metrics = self.measure_text(text, style);

        let x = container_width.saturating_sub(metrics.width) / 2;
        let y = container_height.saturating_sub(metrics.height) / 2;

        (x, y)
    }

    /// Create text style optimized for contrast
    pub fn create_contrasted_style(
        &self,
        base_color: Rgba<u8>,
        background_luminance: f32,
        font_size: f32,
    ) -> TextStyle {
        // Calculate if we need light or dark text
        let text_color = if background_luminance > 0.5 {
            // Dark text on light background
            Rgba([0, 0, 0, 255])
        } else {
            // Light text on dark background
            Rgba([255, 255, 255, 255])
        };

        // Create contrasting outline
        let outline_color = if text_color[0] > 128 {
            Rgba([0, 0, 0, 255]) // Dark outline for light text
        } else {
            Rgba([255, 255, 255, 255]) // Light outline for dark text
        };

        TextStyle {
            font_size,
            color: text_color,
            outline_color: Some(outline_color),
            outline_width: 1.5,
            letter_spacing: 0.0,
            line_height: 1.2,
        }
    }
}
