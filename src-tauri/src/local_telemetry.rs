//! Starts the bundled local telemetry sidecar and feeds its verified movement
//! stream into the existing tracker. The sidecar is an Isle Pulse component;
//! it has no runtime dependency on IsleLiveMap or a server website.

use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SidecarMessage {
    #[serde(rename = "type")]
    kind: String,
    x: Option<f64>,
    y: Option<f64>,
    z: Option<f64>,
    heading_deg: Option<f64>,
    status: Option<String>,
    message: Option<String>,
}

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let Some(sidecar) = find_sidecar(&app) else {
            log::warn!("local telemetry sidecar was not found");
            return;
        };

        ensure_npcap(&app, &sidecar);

        loop {
            if let Err(error) = run_once(&app, &sidecar) {
                log::warn!("local telemetry stopped: {error}");
            }
            crate::pipeline::clear_local_heading(&app);
            std::thread::sleep(Duration::from_secs(5));
        }
    });
}

fn ensure_npcap(app: &AppHandle, sidecar: &PathBuf) {
    // Checking the DLLs alone misses a disabled or broken capture driver.
    // Let the bundled sidecar ask libpcap for the device list, which verifies
    // both the runtime files and the running Windows driver.
    if npcap_runtime_ready(sidecar) {
        return;
    }

    let install = app
        .dialog()
        .message(
            "Isle Pulse cần Npcap để đọc vị trí và góc camera realtime trên mọi server. "
                .to_owned()
                + "App sẽ tải Npcap 1.88 từ npcap.com, kiểm tra mã băm và chữ ký số trước khi mở bộ cài.\n\n"
                + "Trong cửa sổ Npcap, đừng chọn chế độ chỉ cho Administrator.",
        )
        .title("Bật telemetry realtime")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Cài Npcap".into(),
            "Để sau".into(),
        ))
        .blocking_show();
    if !install {
        log::warn!("Npcap setup was deferred; local telemetry remains unavailable");
        return;
    }

    let mut command = Command::new(sidecar);
    command
        .arg("--install-npcap")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let outcome = command.status();
    let (kind, message) = match outcome {
        Ok(status) if status.success() => (
            MessageDialogKind::Info,
            "Npcap đã sẵn sàng. Isle Pulse sẽ tự bắt đầu telemetry realtime.",
        ),
        Ok(status) if status.code() == Some(4) => (
            MessageDialogKind::Warning,
            "Npcap đã cài. Hãy khởi động lại Windows để bật telemetry realtime.",
        ),
        Ok(status) if status.code() == Some(3) => (
            MessageDialogKind::Warning,
            "Cài đặt Npcap đã bị hủy. Isle Pulse vẫn dùng dữ liệu dự phòng từ server.",
        ),
        _ => (
            MessageDialogKind::Error,
            "Chưa cài được Npcap. Kiểm tra mạng rồi mở lại Isle Pulse để thử lại.",
        ),
    };
    app.dialog()
        .message(message)
        .title("Telemetry realtime")
        .kind(kind)
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();
}

fn npcap_runtime_ready(sidecar: &PathBuf) -> bool {
    let mut command = Command::new(sidecar);
    command
        .arg("--check-npcap")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    command.status().is_ok_and(|status| status.success())
}

fn run_once(app: &AppHandle, sidecar: &PathBuf) -> Result<(), String> {
    let mut command = Command::new(sidecar);
    command
        .arg("--parent-pid")
        .arg(std::process::id().to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);

    let mut child = command
        .spawn()
        .map_err(|error| format!("cannot start {}: {error}", sidecar.display()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "sidecar stdout was unavailable".to_string())?;

    log::info!("local telemetry started from {}", sidecar.display());
    for line in BufReader::new(stdout).lines() {
        let line = line.map_err(|error| format!("cannot read sidecar output: {error}"))?;
        let message: SidecarMessage = match serde_json::from_str(&line) {
            Ok(message) => message,
            Err(error) => {
                log::debug!("ignored malformed local telemetry message: {error}");
                continue;
            }
        };

        match message.kind.as_str() {
            "movement" => {
                let (Some(x), Some(y), Some(z), Some(heading_deg)) =
                    (message.x, message.y, message.z, message.heading_deg)
                else {
                    continue;
                };
                if [x, y, z, heading_deg].iter().all(|value| value.is_finite()) {
                    crate::pipeline::ingest_local_sample_with_heading(app, x, y, z, heading_deg);
                }
            }
            "status" => {
                log::info!(
                    "local telemetry status: {}",
                    message.status.as_deref().unwrap_or("unknown")
                );
            }
            "error" => {
                log::warn!(
                    "local telemetry error ({}): {}",
                    message.status.as_deref().unwrap_or("unknown"),
                    message.message.as_deref().unwrap_or("no details")
                );
            }
            _ => {}
        }
    }

    let status = child
        .wait()
        .map_err(|error| format!("cannot wait for sidecar: {error}"))?;
    Err(format!("sidecar exited with {status}"))
}

fn find_sidecar(app: &AppHandle) -> Option<PathBuf> {
    const INSTALLED_NAME: &str = "isle-pulse-local-telemetry.exe";
    const BUILD_NAME: &str = "isle-pulse-local-telemetry-x86_64-pc-windows-msvc.exe";

    let mut candidates = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join(INSTALLED_NAME));
        candidates.push(resource_dir.join(BUILD_NAME));
    }
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(directory) = current_exe.parent() {
            candidates.push(directory.join(INSTALLED_NAME));
            candidates.push(directory.join(BUILD_NAME));
        }
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(BUILD_NAME),
    );

    candidates.into_iter().find(|path| path.is_file())
}
