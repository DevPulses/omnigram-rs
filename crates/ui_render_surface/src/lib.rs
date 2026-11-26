//! 2D software render surface and painter primitives for Telegram Desktop UI.
//!
//! Ported from `tdesktop/Telegram/lib_ui/ui/painter.{h,cpp}`
//! and `tdesktop/Telegram/SourceFiles/ui/widgets/box_content_divider.{h,cpp}`.

use styles_parser::types::Color;
use ui_main_window::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_style(color: &Color) -> Self {
        Self {
            r: color.r,
            g: color.g,
            b: color.b,
            a: color.a,
        }
    }

    pub fn to_argb_u32(self) -> u32 {
        ((self.a as u32) << 24)
            | ((self.r as u32) << 16)
            | ((self.g as u32) << 8)
            | (self.b as u32)
    }
}

/// CPU RGBA8888 pixel surface with clipping, alpha blending, and rounded-rect primitives.
#[derive(Debug, Clone)]
pub struct RenderSurface {
    width: i32,
    height: i32,
    pixels: Vec<u8>,
    clip: Option<Rect>,
}

impl RenderSurface {
    pub fn new(width: i32, height: i32) -> Self {
        let w = width.max(0);
        let h = height.max(0);
        Self {
            width: w,
            height: h,
            pixels: vec![0u8; (w as usize) * (h as usize) * 4],
            clip: None,
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn set_clip(&mut self, rect: Option<Rect>) {
        self.clip = rect;
    }

    pub fn clear(&mut self, color: RgbaColor) {
        self.clip = None;
        self.fill_rect(Rect::new(0, 0, self.width, self.height), color);
    }

    pub fn blend_pixel(&mut self, x: i32, y: i32, src: RgbaColor) {
        if x < 0 || y < 0 || x >= self.width || y >= self.height || src.a == 0 {
            return;
        }
        if let Some(c) = self.clip {
            if x < c.x || y < c.y || x >= c.x + c.width || y >= c.y + c.height {
                return;
            }
        }
        let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        if src.a == 255 {
            self.pixels[idx] = src.r;
            self.pixels[idx + 1] = src.g;
            self.pixels[idx + 2] = src.b;
            self.pixels[idx + 3] = 255;
        } else {
            let alpha = src.a as u16;
            let inv = 255 - alpha;
            self.pixels[idx] = ((src.r as u16 * alpha + self.pixels[idx] as u16 * inv) / 255) as u8;
            self.pixels[idx + 1] =
                ((src.g as u16 * alpha + self.pixels[idx + 1] as u16 * inv) / 255) as u8;
            self.pixels[idx + 2] =
                ((src.b as u16 * alpha + self.pixels[idx + 2] as u16 * inv) / 255) as u8;
            self.pixels[idx + 3] = 255;
        }
    }

    pub fn pixel_at(&self, x: i32, y: i32) -> Option<RgbaColor> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        let idx = ((y as usize) * (self.width as usize) + (x as usize)) * 4;
        Some(RgbaColor::new(
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ))
    }

    pub fn fill_rect(&mut self, rect: Rect, color: RgbaColor) {
        if rect.is_empty() || color.a == 0 {
            return;
        }
        let x0 = rect.x.max(0);
        let y0 = rect.y.max(0);
        let x1 = (rect.x + rect.width).min(self.width);
        let y1 = (rect.y + rect.height).min(self.height);
        for y in y0..y1 {
            for x in x0..x1 {
                self.blend_pixel(x, y, color);
            }
        }
    }

    pub fn fill_rounded_rect(&mut self, rect: Rect, radius: i32, color: RgbaColor) {
        if rect.is_empty() || color.a == 0 {
            return;
        }
        let r = radius.max(0).min(rect.width / 2).min(rect.height / 2);
        let x0 = rect.x.max(0);
        let y0 = rect.y.max(0);
        let x1 = (rect.x + rect.width).min(self.width);
        let y1 = (rect.y + rect.height).min(self.height);

        for y in y0..y1 {
            for x in x0..x1 {
                if r > 0 {
                    let dx = if x < rect.x + r {
                        (rect.x + r) - x
                    } else if x >= rect.x + rect.width - r {
                        x - (rect.x + rect.width - r - 1)
                    } else {
                        0
                    };
                    let dy = if y < rect.y + r {
                        (rect.y + r) - y
                    } else if y >= rect.y + rect.height - r {
                        y - (rect.y + rect.height - r - 1)
                    } else {
                        0
                    };
                    if dx * dx + dy * dy > r * r {
                        continue;
                    }
                }
                self.blend_pixel(x, y, color);
            }
        }
    }

    /// Draws a horizontal divider strip matching `Ui::BoxContentDivider`.
    pub fn draw_box_divider(&mut self, y: i32, width: i32, height: i32, bg: RgbaColor, border: RgbaColor) {
        self.fill_rect(Rect::new(0, y, width, height), bg);
        if height >= 2 {
            self.fill_rect(Rect::new(0, y, width, 1), border);
            self.fill_rect(Rect::new(0, y + height - 1, width, 1), border);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_fill_and_clip() {
        let mut surface = RenderSurface::new(100, 100);
        surface.clear(RgbaColor::new(255, 255, 255, 255));
        assert_eq!(surface.pixel_at(0, 0), Some(RgbaColor::new(255, 255, 255, 255)));

        surface.set_clip(Some(Rect::new(10, 10, 20, 20)));
        surface.fill_rect(Rect::new(0, 0, 50, 50), RgbaColor::new(0, 128, 255, 255));

        // Outside clip remains white
        assert_eq!(surface.pixel_at(5, 5), Some(RgbaColor::new(255, 255, 255, 255)));
        // Inside clip becomes blue
        assert_eq!(surface.pixel_at(15, 15), Some(RgbaColor::new(0, 128, 255, 255)));
    }

    #[test]
    fn test_rounded_rect_and_divider() {
        let mut surface = RenderSurface::new(60, 60);
        surface.clear(RgbaColor::new(0, 0, 0, 255));
        surface.fill_rounded_rect(Rect::new(10, 10, 40, 40), 8, RgbaColor::new(255, 0, 0, 255));

        // Corner (10, 10) is clipped by radius
        assert_eq!(surface.pixel_at(10, 10), Some(RgbaColor::new(0, 0, 0, 255)));
        // Center is red
        assert_eq!(surface.pixel_at(30, 30), Some(RgbaColor::new(255, 0, 0, 255)));

        surface.draw_box_divider(0, 60, 8, RgbaColor::new(240, 240, 240, 255), RgbaColor::new(200, 200, 200, 255));
        assert_eq!(surface.pixel_at(5, 0), Some(RgbaColor::new(200, 200, 200, 255)));
        assert_eq!(surface.pixel_at(5, 3), Some(RgbaColor::new(240, 240, 240, 255)));
    }
}
