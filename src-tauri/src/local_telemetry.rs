//! Starts the bundled local telemetry sidecar and feeds its verified movement
//! stream into the existing tracker. The sidecar is an islemap-thienvyma component;
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct InternalMovement {
    x_cm: f64,
    y_cm: f64,
    z_cm: f64,
    heading_deg: f64,
}

/// Convert the sidecar's native Unreal axes to this app's established
/// Lat/Long axis convention.
///
/// The Npcap decoder emits the raw Unreal vector used by IsleLiveMap:
/// Unreal X runs horizontally over the Gateway texture and Unreal Y runs
/// vertically. The rest of this app intentionally follows the coordinate
/// strings and IslePilot adapters instead: internal X is Lat/vertical and
/// internal Y is Long/horizontal. Swapping once at this boundary keeps every
/// existing provider, trail, waypoint, and basemap calibration in one frame.
/// Camera bearing is already north-up and must pass through unchanged.
fn normalize_sidecar_movement(
    unreal_x_cm: f64,
    unreal_y_cm: f64,
    unreal_z_cm: f64,
    heading_deg: f64,
) -> InternalMovement {
    InternalMovement {
        x_cm: unreal_y_cm,
        y_cm: unreal_x_cm,
        z_cm: unreal_z_cm,
        heading_deg,
    }
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
            "islemap-thienvyma cần Npcap để đọc vị trí và góc camera realtime trên mọi server. "
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
            "Npcap đã sẵn sàng. islemap-thienvyma sẽ tự bắt đầu telemetry realtime.",
        ),
        Ok(status) if status.code() == Some(4) => (
            MessageDialogKind::Warning,
            "Npcap đã cài. Hãy khởi động lại Windows để bật telemetry realtime.",
        ),
        Ok(status) if status.code() == Some(3) => (
            MessageDialogKind::Warning,
            "Cài đặt Npcap đã bị hủy. islemap-thienvyma vẫn dùng dữ liệu dự phòng từ server.",
        ),
        _ => (
            MessageDialogKind::Error,
            "Chưa cài được Npcap. Kiểm tra mạng rồi mở lại islemap-thienvyma để thử lại.",
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
                let (Some(unreal_x_cm), Some(unreal_y_cm), Some(unreal_z_cm), Some(heading_deg)) =
                    (message.x, message.y, message.z, message.heading_deg)
                else {
                    continue;
                };
                if [unreal_x_cm, unreal_y_cm, unreal_z_cm, heading_deg]
                    .iter()
                    .all(|value| value.is_finite())
                {
                    let movement = normalize_sidecar_movement(
                        unreal_x_cm,
                        unreal_y_cm,
                        unreal_z_cm,
                        heading_deg,
                    );
                    crate::pipeline::ingest_local_sample_with_heading(
                        app,
                        movement.x_cm,
                        movement.y_cm,
                        movement.z_cm,
                        movement.heading_deg,
                    );
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
    const INSTALLED_NAME: &str = "islemap-thienvyma-telemetry.exe";
    const BUILD_NAME: &str = "islemap-thienvyma-telemetry-x86_64-pc-windows-msvc.exe";

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

#[cfg(test)]
mod tests {
    use super::normalize_sidecar_movement;
    use overlay_core::{world_to_pixel, Calibration};

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "got {actual}, expected {expected} (+/- {tolerance})"
        );
    }

    #[test]
    fn sidecar_unreal_axes_land_at_the_gateway_texture_center() {
        // IsleLiveMap's audited GatewayMapProjection center is Unreal
        // X=51,000 and Y=-49,000. In this app's Lat/Long convention that is
        // internal X=-49,000 (vertical) and Y=51,000 (horizontal).
        let movement = normalize_sidecar_movement(51_000.0, -49_000.0, 12_345.0, 261.5);
        let calibration = Calibration::gateway();
        let (px, py) = world_to_pixel(movement.x_cm, movement.y_cm, calibration);

        assert_close(px, calibration.image_width_px as f64 / 2.0, 1e-9);
        assert_close(py, calibration.image_height_px as f64 / 2.0, 1e-9);
        assert_eq!(movement.z_cm, 12_345.0);
        assert_eq!(movement.heading_deg, 261.5);
    }

    #[test]
    fn dinovietnam_capture_matches_the_reference_gateway_projection() {
        // Captured from DinoVietnam (31.58.143.164:7777). The reference
        // projection places raw Unreal X on the horizontal axis and raw
        // Unreal Y on the vertical axis. The bridge must produce that same
        // pixel while retaining the independently decoded camera bearing.
        let movement = normalize_sidecar_movement(232_414.14, -17_468.84, 23_732.46, 2.0);
        let (px, py) = world_to_pixel(movement.x_cm, movement.y_cm, Calibration::gateway());

        assert_close(px, 5_172.51, 0.01);
        assert_close(py, 4_129.36, 0.01);
        assert_eq!(movement.heading_deg, 2.0);
    }
}
