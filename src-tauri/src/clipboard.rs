//! Clipboard watcher for the explicit manual-position fallback. Port of
//! `app/clipboard.py`; automatic providers bypass this source entirely.
//!
//! In game the player presses Tab and clicks "Asset Location"; the game
//! itself copies the coordinates to the Windows clipboard. We only read them
//! back. No memory reads, no hooks, no synthetic input — that is why this
//! tool is safe next to anti-cheat, and also why position only updates when
//! the player copies it MANUALLY.
//!
//! Technique: poll `GetClipboardSequenceNumber()` — a user32 counter that
//! increments on every clipboard write. Calling it does NOT open or lock the
//! clipboard, so it never contends with other apps. Only when the number
//! changes do we actually read the content.
//!
//! Non-coordinate content is SILENTLY ignored — no log, no UI flash. The
//! user's normal copy/paste must feel untouched. The only write path is an
//! explicit click on “Copy SteamID” in the Friends tab.

use std::time::Duration;

use overlay_core::parse::MAX_CLIPBOARD_LEN;
use overlay_core::{parse_coordinates, NumberFormat};
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, OpenClipboard,
    SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};

use crate::pipeline;
use crate::settings;
use crate::state::{AppState, LockExt};

const CF_UNICODETEXT: u32 = 13;

/// Read the clipboard text.
///
/// - `Err(())`  — could not OPEN the clipboard (another app holds it): the
///   caller must retry next tick without consuming the sequence number.
/// - `Ok(None)` — opened fine but no text on it: consumed, nothing to do.
fn read_clipboard_text() -> Result<Option<String>, ()> {
    unsafe {
        if OpenClipboard(None).is_err() {
            return Err(());
        }
        let text = (|| {
            let handle: HANDLE = GetClipboardData(CF_UNICODETEXT).ok()?;
            let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u16;
            if ptr.is_null() {
                return None;
            }
            let mut len = 0usize;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
            let _ = GlobalUnlock(HGLOBAL(handle.0));
            Some(text)
        })();
        let _ = CloseClipboard();
        Ok(text)
    }
}

/// Write a short, user-requested string as Unicode text. Ownership of the
/// allocation transfers to Windows only after SetClipboardData succeeds.
pub fn write_text(text: &str) -> Result<(), String> {
    let wide = text
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    unsafe {
        OpenClipboard(None).map_err(|error| error.to_string())?;
        let result = (|| {
            EmptyClipboard().map_err(|error| error.to_string())?;
            let memory = GlobalAlloc(GMEM_MOVEABLE, wide.len() * size_of::<u16>())
                .map_err(|error| error.to_string())?;
            let ptr = GlobalLock(memory) as *mut u16;
            if ptr.is_null() {
                let _ = GlobalFree(Some(memory));
                return Err("clipboard-allocation-lock-failed".to_string());
            }
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
            let _ = GlobalUnlock(memory);
            if let Err(error) = SetClipboardData(CF_UNICODETEXT, Some(HANDLE(memory.0))) {
                let _ = GlobalFree(Some(memory));
                return Err(error.to_string());
            }
            Ok(())
        })();
        let _ = CloseClipboard();
        result
    }
}

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last_seq = unsafe { GetClipboardSequenceNumber() };
        loop {
            let (interval_ms, number_format, automatic_position) = {
                let state = app.state::<AppState>();
                let s = state.settings.lock_safe();
                (
                    settings::get_f64(&s, &["poll", "clipboard_ms"], 400.0) as u64,
                    NumberFormat::from_setting(settings::get_str(&s, &["number_format"], "auto")),
                    settings::get_bool(&s, &["provider", "automatic_position"], true),
                )
            };
            std::thread::sleep(Duration::from_millis(interval_ms.max(100)));

            let seq = unsafe { GetClipboardSequenceNumber() };
            if seq == last_seq {
                continue;
            }
            if automatic_position {
                // Automatic providers own position updates. Consume only
                // the sequence number and never open the clipboard.
                last_seq = seq;
                continue;
            }
            match read_clipboard_text() {
                Err(()) => continue, // clipboard busy — retry next tick
                Ok(None) => last_seq = seq,
                Ok(Some(text)) => {
                    last_seq = seq;
                    if text.is_empty() || text.chars().count() > MAX_CLIPBOARD_LEN {
                        continue;
                    }
                    if let Some((x, y, z)) = parse_coordinates(&text, number_format) {
                        pipeline::ingest_sample(&app, x, y, z);
                    }
                    // else: not coordinates — silently ignore.
                }
            }
        }
    });
}
