use crate::window::{self, GameWindow};
use image::RgbaImage;

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("Target window is missing, minimized, or has no client area")]
    InvalidWindow,
    #[error("Windows API error: {0}")]
    Api(String),
    #[error("Capture produced a black frame; the game may be protected or unavailable")]
    BlackFrame,
}

#[cfg(windows)]
pub fn capture(window: &GameWindow) -> Result<RgbaImage, CaptureError> {
    use windows::Win32::{
        Foundation::{HWND, RECT},
        Graphics::Gdi::{
            CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
            ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, SRCCOPY,
        },
        Storage::Xps::{PrintWindow, PW_CLIENTONLY},
        UI::WindowsAndMessaging::GetClientRect,
    };
    if !window::valid(window) {
        return Err(CaptureError::InvalidWindow);
    }
    unsafe {
        let hwnd = HWND(window.hwnd as *mut _);
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect).map_err(|e| CaptureError::Api(e.to_string()))?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return Err(CaptureError::InvalidWindow);
        }
        let source = GetDC(hwnd);
        if source.0.is_null() {
            return Err(CaptureError::Api("GetDC returned null".into()));
        }
        let memory = CreateCompatibleDC(source);
        let bitmap = CreateCompatibleBitmap(source, width, height);
        if memory.0.is_null() || bitmap.0.is_null() {
            if !memory.0.is_null() {
                let _ = DeleteDC(memory);
            }
            let _ = ReleaseDC(hwnd, source);
            return Err(CaptureError::Api(
                "could not allocate capture bitmap".into(),
            ));
        }
        let old = SelectObject(memory, bitmap);
        let printed = PrintWindow(hwnd, memory, PW_CLIENTONLY).as_bool();
        if !printed {
            let _ = windows::Win32::Graphics::Gdi::BitBlt(
                memory, 0, 0, width, height, source, 0, 0, SRCCOPY,
            );
        }
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bgra = vec![0u8; (width * height * 4) as usize];
        let rows = GetDIBits(
            memory,
            bitmap,
            0,
            height as u32,
            Some(bgra.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        let _ = SelectObject(memory, old);
        let _ = DeleteObject(bitmap);
        let _ = DeleteDC(memory);
        let _ = ReleaseDC(hwnd, source);
        if rows == 0 {
            return Err(CaptureError::Api("GetDIBits failed".into()));
        }
        let (pixels, remainder) = bgra.as_chunks_mut::<4>();
        debug_assert!(remainder.is_empty());
        for pixel in pixels {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        let image = RgbaImage::from_raw(width as u32, height as u32, bgra)
            .ok_or_else(|| CaptureError::Api("invalid bitmap dimensions".into()))?;
        let non_black = image
            .pixels()
            .filter(|p| p[0] > 4 || p[1] > 4 || p[2] > 4)
            .count();
        if non_black < (width as usize * height as usize / 500) {
            return Err(CaptureError::BlackFrame);
        }
        Ok(image)
    }
}
#[cfg(not(windows))]
pub fn capture(_: &GameWindow) -> Result<RgbaImage, CaptureError> {
    Err(CaptureError::Api(
        "This feature is only available on Windows".into(),
    ))
}
