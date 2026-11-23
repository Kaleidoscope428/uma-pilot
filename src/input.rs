use crate::{vision::Match, window::{self, GameWindow}};

#[derive(Debug, thiserror::Error)] pub enum InputError { #[error("Target window is no longer valid")] InvalidWindow, #[error("Windows API error: {0}")] Api(String), #[error("This feature is only available on Windows")] Unsupported }
pub fn target_point(window: &GameWindow, hit: &Match) -> Result<(i32, i32), InputError> {
    if !window::valid(window) { return Err(InputError::InvalidWindow); }
    Ok((window.client_x + hit.x as i32 + (hit.width / 2) as i32, window.client_y + hit.y as i32 + (hit.height / 2) as i32))
}
#[cfg(windows)] pub fn click_once(point: (i32,i32)) -> Result<(), InputError> {
    use windows::Win32::UI::{Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEINPUT}, WindowsAndMessaging::SetCursorPos};
    unsafe {
        SetCursorPos(point.0, point.1).map_err(|e| InputError::Api(e.to_string()))?;
        let inputs = [INPUT { r#type: INPUT_MOUSE, Anonymous: INPUT_0 { mi: MOUSEINPUT { dwFlags: MOUSEEVENTF_LEFTDOWN, ..Default::default() } } }, INPUT { r#type: INPUT_MOUSE, Anonymous: INPUT_0 { mi: MOUSEINPUT { dwFlags: MOUSEEVENTF_LEFTUP, ..Default::default() } } }];
        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32); if sent != inputs.len() as u32 { return Err(InputError::Api(format!("SendInput sent {sent}/2 events"))); } Ok(())
    }
}
#[cfg(not(windows))] pub fn click_once(_: (i32,i32)) -> Result<(), InputError> { Err(InputError::Unsupported) }
