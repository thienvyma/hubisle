use std::ffi::c_void;
use std::fmt;
use std::mem::size_of;

use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS,
    HGDIOBJ, SRCCOPY,
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct NormalizedRect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Default for NormalizedRect {
    fn default() -> Self {
        Self { x: 0.61, y: 0.28, w: 0.28, h: 0.22 }
    }
}

impl NormalizedRect {
    pub fn clamped(self) -> Self {
        let x = self.x.clamp(0.0, 0.999);
        let y = self.y.clamp(0.0, 0.999);
        let w = self.w.clamp(0.001, 1.0 - x);
        let h = self.h.clamp(0.001, 1.0 - y);
        Self { x, y, w, h }
    }

    pub fn to_screen_rect(self, game_rect: (i32, i32, i32, i32)) -> (i32, i32, i32, i32) {
        let (game_x, game_y, game_w, game_h) = game_rect;
        let rect = self.clamped();
        let x = game_x + (rect.x * game_w as f64).round() as i32;
        let y = game_y + (rect.y * game_h as f64).round() as i32;
        let mut w = (rect.w * game_w as f64).round() as i32;
        let mut h = (rect.h * game_h as f64).round() as i32;
        w = w.max(1).min((game_x + game_w - x).max(1));
        h = h.max(1).min((game_y + game_h - y).max(1));
        (x, y, w, h)
    }

    pub fn from_screen_rect(
        overlay_rect: (i32, i32, i32, i32),
        game_rect: (i32, i32, i32, i32),
    ) -> Self {
        let (x, y, w, h) = overlay_rect;
        let (game_x, game_y, game_w, game_h) = game_rect;
        if game_w <= 0 || game_h <= 0 {
            return Self::default();
        }
        Self {
            x: (x - game_x) as f64 / game_w as f64,
            y: (y - game_y) as f64 / game_h as f64,
            w: w as f64 / game_w as f64,
            h: h as f64 / game_h as f64,
        }
        .clamped()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GrayFrame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl GrayFrame {
    pub fn is_blank(&self) -> bool {
        if self.pixels.is_empty() {
            return true;
        }
        let mut histogram = [0usize; 256];
        for value in &self.pixels {
            histogram[*value as usize] += 1;
        }
        let dominant = histogram.into_iter().max().unwrap_or(0);
        if dominant * 100 >= self.pixels.len() * 99 {
            return true;
        }
        let min = self.pixels.iter().copied().min().unwrap_or(0);
        let max = self.pixels.iter().copied().max().unwrap_or(0);
        max.saturating_sub(min) <= 2
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CaptureError {
    InvalidBounds,
    DeviceContext,
    Bitmap,
    Blit,
    Readback,
    BlankFrame,
}

impl fmt::Display for CaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidBounds => "invalid capture bounds",
            Self::DeviceContext => "Windows game capture context is unavailable",
            Self::Bitmap => "Windows game capture bitmap could not be created",
            Self::Blit => "Windows could not copy the requested game region",
            Self::Readback => "Windows could not read back the captured pixels",
            Self::BlankFrame => "captured game region is blank",
        };
        f.write_str(message)
    }
}

impl std::error::Error for CaptureError {}

pub trait MutationFrameSource: Send + Sync {
    fn capture(
        &self,
        hwnd: isize,
        game_rect: (i32, i32, i32, i32),
        rect: NormalizedRect,
    ) -> Result<GrayFrame, CaptureError>;
}

#[derive(Default)]
pub struct GdiFrameSource;

impl MutationFrameSource for GdiFrameSource {
    fn capture(
        &self,
        hwnd: isize,
        game_rect: (i32, i32, i32, i32),
        rect: NormalizedRect,
    ) -> Result<GrayFrame, CaptureError> {
        let (screen_x, screen_y, width, height) = rect.to_screen_rect(game_rect);
        if width <= 0 || height <= 0 {
            return Err(CaptureError::InvalidBounds);
        }
        let (game_x, game_y, _, _) = game_rect;
        let source_x = screen_x - game_x;
        let source_y = screen_y - game_y;
        let game_hwnd = HWND(hwnd as *mut c_void);

        unsafe {
            // Capture the target window's client DC, not the composited
            // desktop. That keeps our own topmost translation overlay out of
            // the next OCR frame and avoids a visual hide/show flicker.
            let game_dc = GetDC(Some(game_hwnd));
            if game_dc.is_invalid() {
                return Err(CaptureError::DeviceContext);
            }
            let memory_dc = CreateCompatibleDC(Some(game_dc));
            if memory_dc.is_invalid() {
                ReleaseDC(Some(game_hwnd), game_dc);
                return Err(CaptureError::DeviceContext);
            }
            let bitmap = CreateCompatibleBitmap(game_dc, width, height);
            if bitmap.is_invalid() {
                let _ = DeleteDC(memory_dc);
                ReleaseDC(Some(game_hwnd), game_dc);
                return Err(CaptureError::Bitmap);
            }
            let old_object = SelectObject(memory_dc, HGDIOBJ(bitmap.0));

            let copied = BitBlt(
                memory_dc,
                0,
                0,
                width,
                height,
                Some(game_dc),
                source_x,
                source_y,
                SRCCOPY | CAPTUREBLT,
            );
            if copied.is_err() {
                let _ = SelectObject(memory_dc, old_object);
                let _ = DeleteObject(HGDIOBJ(bitmap.0));
                let _ = DeleteDC(memory_dc);
                ReleaseDC(Some(game_hwnd), game_dc);
                return Err(CaptureError::Blit);
            }

            let mut info = BITMAPINFO::default();
            info.bmiHeader = BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            };
            let mut bgra = vec![0u8; width as usize * height as usize * 4];
            let lines = GetDIBits(
                memory_dc,
                bitmap,
                0,
                height as u32,
                Some(bgra.as_mut_ptr().cast::<c_void>()),
                &mut info,
                DIB_RGB_COLORS,
            );

            let _ = SelectObject(memory_dc, old_object);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(memory_dc);
            ReleaseDC(Some(game_hwnd), game_dc);

            if lines == 0 {
                return Err(CaptureError::Readback);
            }

            let mut pixels = Vec::with_capacity(width as usize * height as usize);
            for pixel in bgra.chunks_exact(4) {
                let b = pixel[0] as u16;
                let g = pixel[1] as u16;
                let r = pixel[2] as u16;
                pixels.push(((77 * r + 150 * g + 29 * b) >> 8) as u8);
            }
            let frame = GrayFrame {
                width: width as u32,
                height: height as u32,
                pixels,
            };
            if frame.is_blank() {
                Err(CaptureError::BlankFrame)
            } else {
                Ok(frame)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GrayFrame, NormalizedRect};

    #[test]
    fn normalized_rect_scales_across_common_resolutions() {
        let rect = NormalizedRect { x: 0.50, y: 0.25, w: 0.25, h: 0.20 };
        assert_eq!(rect.to_screen_rect((0, 0, 1920, 1080)), (960, 270, 480, 216));
        assert_eq!(rect.to_screen_rect((100, 50, 2560, 1440)), (1380, 410, 640, 288));
        assert_eq!(rect.to_screen_rect((-200, 20, 3440, 1440)), (1520, 380, 860, 288));
    }

    #[test]
    fn normalized_rect_is_clamped_inside_game_client() {
        let rect = NormalizedRect { x: -0.2, y: 0.95, w: 2.0, h: 0.5 };
        let (x, y, w, h) = rect.to_screen_rect((10, 20, 1000, 500));
        assert!(x >= 10 && y >= 20);
        assert!(x + w <= 1010);
        assert!(y + h <= 520);
    }

    #[test]
    fn screen_rect_round_trips_to_normalized_coordinates() {
        let normalized = NormalizedRect::from_screen_rect((1060, 290, 480, 216), (100, 20, 1920, 1080));
        assert!((normalized.x - 0.5).abs() < 0.001);
        assert!((normalized.y - 0.25).abs() < 0.001);
        assert!((normalized.w - 0.25).abs() < 0.001);
        assert!((normalized.h - 0.20).abs() < 0.001);
    }

    #[test]
    fn blank_frames_are_detected_without_disk_io() {
        let blank = GrayFrame { width: 20, height: 20, pixels: vec![3; 400] };
        assert!(blank.is_blank());

        let mut pixels = vec![0; 400];
        for (index, value) in pixels.iter_mut().enumerate() {
            *value = (index % 255) as u8;
        }
        let detailed = GrayFrame { width: 20, height: 20, pixels };
        assert!(!detailed.is_blank());
    }
}
