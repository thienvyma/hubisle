//! Win32 calls. Port of `app/winapi.py` — including its safety contract.
//!
//! EVERY FUNCTION IN THIS MODULE IS ON THE ALLOWED LIST:
//!   - Read-only: EnumWindows, GetWindowRect, GetClientRect, ClientToScreen,
//!     GetWindowThreadProcessId, IsWindowVisible, IsIconic,
//!     GetForegroundWindow, the Toolhelp32 process snapshot, and
//!     GetClipboardSequenceNumber / clipboard READS (in clipboard.rs).
//!   - Writes: ONLY to this app's own windows (SetWindowLongPtr, SetWindowPos).
//!   - RegisterHotKey and PeekMessageW on our own hotkey thread (in
//!     hotkeys.rs) — documented, cooperative OS APIs, not keyboard hooks.
//!   - A fixed `/unstuck` chat macro (in unstuck.rs), only while the verified
//!     The Isle window is foreground. It exposes no generic text/key surface.
//!
//! The game runs kernel-level Easy Anti-Cheat. The bundled local telemetry
//! sidecar may passively capture outbound UDP owned by the game through Npcap;
//! it does not open or modify the game process. ABSOLUTELY NEVER add:
//! OpenProcess / ReadProcessMemory aimed at the game, SetParent into the game
//! window, DLL injection, DirectX hooks, arbitrary synthetic input,
//! keybd_event / PostMessage to the game, or low-level keyboard hooks
//! (SetWindowsHookEx). CI greps for those names and fails the build.

pub mod game_window;
pub mod overlay;
pub mod unstuck;
pub mod vis;
