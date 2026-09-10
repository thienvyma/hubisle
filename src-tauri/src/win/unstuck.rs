//! Fixed, foreground-gated chat macro for The Isle.
//!
//! This module deliberately has no generic public "send text" function. The
//! only exported action sends `/unstuck`, and every step stops if focus leaves
//! the verified game window.

use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

use crate::settings::GAME_PROCESS_NAME;

const VK_RETURN: u16 = 0x0D;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacroStep {
    Key(u16),
    Text(&'static str),
    DelayMs(u64),
}

pub fn unstuck_macro_plan() -> Vec<MacroStep> {
    vec![
        MacroStep::Key(VK_RETURN),
        MacroStep::DelayMs(70),
        MacroStep::Text("/unstuck"),
        MacroStep::DelayMs(30),
        MacroStep::Key(VK_RETURN),
    ]
}

fn key_input(vk: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

fn unicode_input(unit: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wScan: unit,
                dwFlags: KEYEVENTF_UNICODE | flags,
                ..Default::default()
            },
        },
    }
}

fn emit_key(vk: u16) -> bool {
    let inputs = [
        key_input(vk, KEYBD_EVENT_FLAGS(0)),
        key_input(vk, KEYEVENTF_KEYUP),
    ];
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == inputs.len() as u32 }
}

fn emit_unstuck_text() -> bool {
    let mut inputs = Vec::with_capacity("/unstuck".encode_utf16().count() * 2);
    for unit in "/unstuck".encode_utf16() {
        inputs.push(unicode_input(unit, KEYBD_EVENT_FLAGS(0)));
        inputs.push(unicode_input(unit, KEYEVENTF_KEYUP));
    }
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) == inputs.len() as u32 }
}

/// Open chat, type `/unstuck`, and submit it. Focus is checked before every
/// output step, so an Alt-Tab during the short sequence cannot type into a
/// different application.
pub fn send_unstuck_if_game_foreground() {
    let Some(game) = super::game_window::find_game_window(GAME_PROCESS_NAME) else {
        return;
    };
    for step in unstuck_macro_plan() {
        if !super::game_window::is_foreground(game) {
            return;
        }
        let ok = match step {
            MacroStep::Key(vk) => emit_key(vk),
            MacroStep::Text("/unstuck") => emit_unstuck_text(),
            MacroStep::Text(_) => false,
            MacroStep::DelayMs(ms) => {
                std::thread::sleep(Duration::from_millis(ms));
                true
            }
        };
        if !ok {
            log::warn!("failed to emit the fixed /unstuck chat macro");
            return;
        }
    }
}
