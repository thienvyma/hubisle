use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};

use crate::settings;
use crate::win::{game_window, overlay, vis};

use super::capture::NormalizedRect;
use super::MutationOverlayPayload;

pub const WINDOW_LABEL: &str = "mutation-overlay";
const DEFAULT_WIDTH: f64 = 540.0;
const DEFAULT_HEIGHT: f64 = 190.0;

// Keep the transparent WebView2 renderer alive while The Isle covers it.
// The minimap uses the same flags: without them, WebView2 can classify a
// passive HUD as occluded and stop presenting pixels even though the native
// window remains visible/topmost.
const WEBVIEW_ARGS: &str = "--disable-background-timer-throttling \
                            --disable-backgrounding-occluded-windows \
                            --disable-renderer-backgrounding \
                            --disable-features=CalculateNativeWinOcclusion,msWebOOUI,msPdfOOUI,msSmartScreenProtection";

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    if app.get_webview_window(WINDOW_LABEL).is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        WINDOW_LABEL,
        WebviewUrl::App("mutation-overlay.html".into()),
    )
    .title("Mutation Translation")
    .data_directory(settings::local_dir().join("mutation-overlay-webview2"))
    .additional_browser_args(WEBVIEW_ARGS)
    .inner_size(DEFAULT_WIDTH, DEFAULT_HEIGHT)
    .transparent(true)
    .decorations(false)
    .shadow(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .resizable(false)
    .focused(false)
    .focusable(false)
    .visible(false)
    .build()?;

    if let Ok(hwnd) = window.hwnd() {
        let raw = hwnd.0 as isize;
        vis::register(WINDOW_LABEL, raw);
        overlay::assert_overlay_styles(raw);
        overlay::set_click_through(raw, true);
    }
    let _ = window.set_ignore_cursor_events(true);
    Ok(())
}

pub fn window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window(WINDOW_LABEL)
}

pub fn anchor(
    app: &AppHandle,
    game_rect: (i32, i32, i32, i32),
    rect: NormalizedRect,
) -> tauri::Result<()> {
    let Some(window) = window(app) else {
        return Ok(());
    };
    let (x, y, width, height) = rect.to_screen_rect(game_rect);
    window.set_position(PhysicalPosition::new(x, y))?;
    window.set_size(PhysicalSize::new(width.max(1) as u32, height.max(1) as u32))?;
    Ok(())
}

pub fn show(app: &AppHandle) {
    let Some(window) = window(app) else {
        return;
    };
    if window.show().is_ok() {
        if let Ok(hwnd) = window.hwnd() {
            overlay::force_topmost(hwnd.0 as isize);
        }
    }
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = window(app) {
        let _ = window.hide();
    }
}

pub fn emit_payload(app: &AppHandle, payload: &MutationOverlayPayload) {
    if let Some(window) = window(app) {
        let _ = window.emit("mutation-overlay://payload", payload);
    }
}

pub fn clear(app: &AppHandle) {
    if let Some(window) = window(app) {
        let _ = window.emit("mutation-overlay://clear", ());
    }
}

pub fn begin_calibration(
    app: &AppHandle,
    game_rect: (i32, i32, i32, i32),
    rect: NormalizedRect,
) -> Result<(), String> {
    anchor(app, game_rect, rect).map_err(|error| error.to_string())?;
    let window = window(app).ok_or_else(|| "Mutation overlay window is unavailable".to_string())?;

    // Decorations are temporary and deliberate: they give Windows-native
    // drag/resize affordances without implementing synthetic input over the
    // game. We save the WebView client rectangle, so title-bar dimensions do
    // not pollute the normalized game coordinates.
    window
        .set_decorations(true)
        .map_err(|error| error.to_string())?;
    window
        .set_resizable(true)
        .map_err(|error| error.to_string())?;
    window
        .set_focusable(true)
        .map_err(|error| error.to_string())?;
    window
        .set_ignore_cursor_events(false)
        .map_err(|error| error.to_string())?;
    if let Ok(hwnd) = window.hwnd() {
        overlay::set_click_through(hwnd.0 as isize, false);
    }
    let _ = window.emit("mutation-overlay://calibration", true);
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

pub fn end_calibration(
    app: &AppHandle,
    game_rect: (i32, i32, i32, i32),
    rect: NormalizedRect,
) -> Result<(), String> {
    let window = window(app).ok_or_else(|| "Mutation overlay window is unavailable".to_string())?;
    let _ = window.emit("mutation-overlay://calibration", false);
    window
        .set_decorations(false)
        .map_err(|error| error.to_string())?;
    window
        .set_resizable(false)
        .map_err(|error| error.to_string())?;
    window
        .set_focusable(false)
        .map_err(|error| error.to_string())?;
    window
        .set_ignore_cursor_events(true)
        .map_err(|error| error.to_string())?;
    if let Ok(hwnd) = window.hwnd() {
        overlay::assert_overlay_styles(hwnd.0 as isize);
        overlay::set_click_through(hwnd.0 as isize, true);
    }
    anchor(app, game_rect, rect).map_err(|error| error.to_string())?;
    Ok(())
}

pub fn current_client_rect(app: &AppHandle) -> Option<(i32, i32, i32, i32)> {
    let window = window(app)?;
    let hwnd = window.hwnd().ok()?;
    game_window::client_rect_on_screen(hwnd.0 as isize)
}

pub fn preview_without_game(app: &AppHandle) {
    let Some(window) = window(app) else {
        return;
    };
    let _ = window.set_position(PhysicalPosition::new(80, 80));
    let _ = window.set_size(PhysicalSize::new(
        DEFAULT_WIDTH as u32,
        DEFAULT_HEIGHT as u32,
    ));
    show(app);
}
