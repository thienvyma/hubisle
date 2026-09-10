// The packaged app is always a GUI program. Diagnostics are written through
// the Tauri log plugin, so debug installers must not allocate a console either.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use std::path::PathBuf;

fn main() {
    // `--replay <file>` (or THEISLE_REPLAY=<file>, easier to thread through
    // `npm run tauri dev`): drive the UI from a fixture, no game needed.
    let mut args = std::env::args().skip(1);
    let mut replay: Option<PathBuf> = std::env::var_os("THEISLE_REPLAY").map(PathBuf::from);
    while let Some(arg) = args.next() {
        if arg == "--replay" {
            replay = args.next().map(PathBuf::from);
        }
    }
    theisle_overlay_lib::run(replay);
}
