use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameWindow {
    pub hwnd: isize,
    pub title: String,
    pub client_x: i32,
    pub client_y: i32,
    pub width: u32,
    pub height: u32,
}
impl fmt::Display for GameWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({} × {})", self.title, self.width, self.height)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Windows API error: {0}")]
    Api(String),
}

#[cfg(windows)]
mod platform {
    use super::*;
    use windows::Win32::{
        Foundation::{BOOL, HWND, LPARAM, POINT, RECT},
        Graphics::Gdi::ClientToScreen,
        UI::{
            HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            WindowsAndMessaging::{
                EnumWindows, GetClientRect, GetWindowTextLengthW, GetWindowTextW, IsIconic,
                IsWindow, IsWindowVisible, SetProcessDPIAware,
            },
        },
    };
    pub fn enable_dpi_awareness() {
        unsafe {
            let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            let _ = SetProcessDPIAware();
        }
    }
    pub fn windows() -> Result<Vec<GameWindow>, WindowError> {
        unsafe extern "system" fn visit(hwnd: HWND, data: LPARAM) -> BOOL {
            let list = &mut *(data.0 as *mut Vec<GameWindow>);
            if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                return BOOL(1);
            }
            let length = GetWindowTextLengthW(hwnd);
            if length == 0 {
                return BOOL(1);
            }
            let mut buf = vec![0u16; length as usize + 1];
            let read = GetWindowTextW(hwnd, &mut buf);
            if read == 0 {
                return BOOL(1);
            }
            if let Ok(Some(item)) = describe(hwnd) {
                list.push(item);
            }
            BOOL(1)
        }
        let mut list: Vec<GameWindow> = Vec::new();
        unsafe {
            EnumWindows(Some(visit), LPARAM(&mut list as *mut _ as isize))
                .map_err(|e| WindowError::Api(e.to_string()))?;
        }
        list.sort_by_key(|item| item.title.to_lowercase());
        Ok(list)
    }
    pub fn describe(hwnd: HWND) -> Result<Option<GameWindow>, WindowError> {
        unsafe {
            if !IsWindow(hwnd).as_bool()
                || !IsWindowVisible(hwnd).as_bool()
                || IsIconic(hwnd).as_bool()
            {
                return Ok(None);
            }
            let mut rect = RECT::default();
            GetClientRect(hwnd, &mut rect).map_err(|e| WindowError::Api(e.to_string()))?;
            if rect.right <= rect.left || rect.bottom <= rect.top {
                return Ok(None);
            }
            let mut point = POINT { x: 0, y: 0 };
            if !ClientToScreen(hwnd, &mut point).as_bool() {
                return Err(WindowError::Api("ClientToScreen failed".into()));
            }
            let length = GetWindowTextLengthW(hwnd);
            let mut buf = vec![0u16; length as usize + 1];
            let read = GetWindowTextW(hwnd, &mut buf);
            let title = String::from_utf16_lossy(&buf[..read as usize]);
            Ok(Some(GameWindow {
                hwnd: hwnd.0 as isize,
                title,
                client_x: point.x,
                client_y: point.y,
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            }))
        }
    }
    pub fn valid(window: &GameWindow) -> bool {
        describe(HWND(window.hwnd as *mut _))
            .ok()
            .flatten()
            .is_some()
    }
}
#[cfg(windows)]
pub use platform::*;
#[cfg(not(windows))]
pub fn enable_dpi_awareness() {}
#[cfg(not(windows))]
pub fn windows() -> Result<Vec<GameWindow>, WindowError> {
    Err(WindowError::Api(
        "This feature is only available on Windows".into(),
    ))
}
#[cfg(not(windows))]
pub fn valid(_: &GameWindow) -> bool {
    false
}
