pub mod capture;
pub mod catalog;
pub mod detect;
pub mod frame_gate;
pub mod ocr;
pub mod window;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

use crate::settings::{self, GAME_PROCESS_NAME};
use crate::state::{AppState, LockExt};
use crate::win::game_window;

use self::capture::{CaptureError, GdiFrameSource, MutationFrameSource, NormalizedRect};
use self::detect::detect_mutation_with_threshold;
use self::frame_gate::FrameGate;
use self::ocr::{MutationOcr, OcrError, WindowsMutationOcr};

const TICK_MS: u64 = 250;
const CAPTURE_INTERVAL_MS: u64 = 500;
const AMBIGUOUS_GRACE_MS: u64 = 1500;
const PREVIEW_MS: u64 = 5000;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationOverlayStatus {
    pub state: String,
    pub name_en: Option<String>,
    pub confidence: Option<f32>,
    pub message: Option<String>,
}

impl MutationOverlayStatus {
    fn new(state: &str) -> Self {
        Self {
            state: state.to_string(),
            name_en: None,
            confidence: None,
            message: None,
        }
    }

    fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationOverlayPayload {
    pub name_en: String,
    pub description_vi: String,
    pub confidence: Option<f32>,
    pub source: String,
}

#[derive(Debug)]
struct RuntimeState {
    status: MutationOverlayStatus,
    manual_name: Option<String>,
    calibrating: bool,
    saved_rect: Option<NormalizedRect>,
    preview_until: Option<Instant>,
    last_payload: Option<MutationOverlayPayload>,
    last_good: Option<Instant>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            status: MutationOverlayStatus::new("disabled"),
            manual_name: None,
            calibrating: false,
            saved_rect: None,
            preview_until: None,
            last_payload: None,
            last_good: None,
        }
    }
}

static RUNTIME: LazyLock<Mutex<RuntimeState>> = LazyLock::new(|| Mutex::new(RuntimeState::default()));
static SUPERVISOR_STARTED: AtomicBool = AtomicBool::new(false);

fn with_runtime<R>(f: impl FnOnce(&mut RuntimeState) -> R) -> R {
    let mut runtime = RUNTIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut runtime)
}

#[derive(Clone, Copy)]
struct OverlayConfig {
    enabled: bool,
    auto_detect: bool,
    rect: NormalizedRect,
    confidence_threshold: f32,
}

fn config(app: &AppHandle) -> OverlayConfig {
    let state = app.state::<AppState>();
    let settings_value = state.settings.lock_safe();
    OverlayConfig {
        enabled: settings::get_bool(&settings_value, &["mutation_overlay", "enabled"], false),
        auto_detect: settings::get_bool(
            &settings_value,
            &["mutation_overlay", "auto_detect"],
            true,
        ),
        rect: NormalizedRect {
            x: settings::get_f64(&settings_value, &["mutation_overlay", "rect", "x"], 0.61),
            y: settings::get_f64(&settings_value, &["mutation_overlay", "rect", "y"], 0.28),
            w: settings::get_f64(&settings_value, &["mutation_overlay", "rect", "w"], 0.28),
            h: settings::get_f64(&settings_value, &["mutation_overlay", "rect", "h"], 0.22),
        }
        .clamped(),
        confidence_threshold: settings::get_f64(
            &settings_value,
            &["mutation_overlay", "confidence_threshold"],
            0.82,
        ) as f32,
    }
}

fn publish_status(app: &AppHandle, next: MutationOverlayStatus) -> MutationOverlayStatus {
    let changed = with_runtime(|runtime| {
        if runtime.status == next {
            false
        } else {
            runtime.status = next.clone();
            true
        }
    });
    if changed {
        let _ = app.emit("mutation-overlay://state", next.clone());
    }
    next
}

fn current_status() -> MutationOverlayStatus {
    with_runtime(|runtime| runtime.status.clone())
}

fn render(app: &AppHandle, payload: MutationOverlayPayload) {
    let changed = with_runtime(|runtime| {
        if runtime.last_payload.as_ref() == Some(&payload) {
            false
        } else {
            runtime.last_payload = Some(payload.clone());
            true
        }
    });
    if changed {
        window::emit_payload(app, &payload);
    }
    window::show(app);
}

fn clear_render(app: &AppHandle) {
    let had_payload = with_runtime(|runtime| runtime.last_payload.take().is_some());
    if had_payload {
        window::clear(app);
    }
    window::hide(app);
}

fn active_game() -> Option<(isize, (i32, i32, i32, i32))> {
    let hwnd = game_window::find_game_window(GAME_PROCESS_NAME)?;
    if game_window::is_iconic(hwnd) || !game_window::is_foreground(hwnd) {
        return None;
    }
    let rect = game_window::client_rect_on_screen(hwnd)?;
    Some((hwnd, rect))
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    window::create(app)?;
    if !SUPERVISOR_STARTED.swap(true, Ordering::SeqCst) {
        spawn_supervisor(app.clone());
    }
    Ok(())
}

fn spawn_supervisor(app: AppHandle) {
    std::thread::spawn(move || {
        let frame_source = GdiFrameSource;
        let ocr = WindowsMutationOcr;
        let mut gate = FrameGate::default();
        let mut last_capture = Instant::now()
            .checked_sub(Duration::from_millis(CAPTURE_INTERVAL_MS))
            .unwrap_or_else(Instant::now);

        loop {
            std::thread::sleep(Duration::from_millis(TICK_MS));
            let now = Instant::now();
            let cfg = config(&app);

            let (calibrating, manual_name, preview_active) = with_runtime(|runtime| {
                let preview_active = runtime.preview_until.is_some_and(|until| until > now);
                if runtime.preview_until.is_some_and(|until| until <= now) {
                    runtime.preview_until = None;
                }
                (
                    runtime.calibrating,
                    runtime.manual_name.clone(),
                    preview_active,
                )
            });

            if preview_active || calibrating {
                continue;
            }

            if !cfg.enabled {
                gate.reset();
                clear_render(&app);
                publish_status(&app, MutationOverlayStatus::new("disabled"));
                continue;
            }

            let Some((game_hwnd, game_rect)) = active_game() else {
                gate.reset();
                clear_render(&app);
                publish_status(
                    &app,
                    MutationOverlayStatus::new("waiting-game")
                        .message("Đang chờ The Isle ở foreground"),
                );
                continue;
            };

            if let Err(error) = window::anchor(&app, game_rect, cfg.rect) {
                clear_render(&app);
                publish_status(
                    &app,
                    MutationOverlayStatus::new("capture-unavailable")
                        .message(format!("Không thể đặt overlay: {error}")),
                );
                continue;
            }

            if let Some(name) = manual_name {
                if let Some(entry) = catalog::find_by_name(&name) {
                    let payload = MutationOverlayPayload {
                        name_en: entry.name_en.clone(),
                        description_vi: entry.description_vi.clone(),
                        confidence: None,
                        source: "manual".to_string(),
                    };
                    render(&app, payload);
                    let mut status = MutationOverlayStatus::new("manual");
                    status.name_en = Some(entry.name_en.clone());
                    publish_status(&app, status);
                }
                continue;
            }

            if !cfg.auto_detect {
                clear_render(&app);
                publish_status(
                    &app,
                    MutationOverlayStatus::new("manual").message("Chọn Mutation thủ công"),
                );
                continue;
            }

            if now.saturating_duration_since(last_capture)
                < Duration::from_millis(CAPTURE_INTERVAL_MS)
            {
                continue;
            }
            last_capture = now;

            let frame = match frame_source.capture(game_hwnd, game_rect, cfg.rect) {
                Ok(frame) => frame,
                Err(CaptureError::BlankFrame) => {
                    let should_clear = with_runtime(|runtime| {
                        runtime
                            .last_good
                            .is_none_or(|last| now.saturating_duration_since(last)
                                > Duration::from_millis(AMBIGUOUS_GRACE_MS))
                    });
                    if should_clear {
                        clear_render(&app);
                    }
                    publish_status(
                        &app,
                        MutationOverlayStatus::new("recognizing")
                            .message("Mở màn hình Mutations để nhận diện"),
                    );
                    continue;
                }
                Err(error) => {
                    clear_render(&app);
                    publish_status(
                        &app,
                        MutationOverlayStatus::new("capture-unavailable")
                            .message(format!("Không chụp được vùng Mutation: {error}")),
                    );
                    continue;
                }
            };

            if !gate.should_ocr(&frame, now) {
                continue;
            }

            let text = match ocr.recognize(&frame) {
                Ok(text) => text,
                Err(OcrError::Unavailable) => {
                    clear_render(&app);
                    publish_status(
                        &app,
                        MutationOverlayStatus::new("ocr-unavailable")
                            .message("Windows OCR không khả dụng — hãy chọn Mutation thủ công"),
                    );
                    continue;
                }
                Err(error) => {
                    publish_status(
                        &app,
                        MutationOverlayStatus::new("recognizing")
                            .message(format!("OCR tạm thời thất bại: {error}")),
                    );
                    continue;
                }
            };

            if let Some(found) = detect_mutation_with_threshold(&text, cfg.confidence_threshold) {
                let payload = MutationOverlayPayload {
                    name_en: found.name_en.clone(),
                    description_vi: found.description_vi,
                    confidence: Some(found.confidence),
                    source: "auto".to_string(),
                };
                render(&app, payload);
                with_runtime(|runtime| runtime.last_good = Some(now));
                let mut status = MutationOverlayStatus::new("recognized");
                status.name_en = Some(found.name_en);
                status.confidence = Some(found.confidence);
                publish_status(&app, status);
            } else {
                let should_clear = with_runtime(|runtime| {
                    runtime
                        .last_good
                        .is_none_or(|last| now.saturating_duration_since(last)
                            > Duration::from_millis(AMBIGUOUS_GRACE_MS))
                });
                if should_clear {
                    clear_render(&app);
                    publish_status(
                        &app,
                        MutationOverlayStatus::new("manual")
                            .message("Không nhận diện đủ chắc chắn — hãy chọn thủ công"),
                    );
                }
            }
        }
    });
}

#[tauri::command]
pub fn mutation_overlay_status() -> MutationOverlayStatus {
    current_status()
}

#[tauri::command]
pub fn mutation_overlay_set_manual(
    app: AppHandle,
    name_en: String,
) -> Result<MutationOverlayStatus, String> {
    let entry = catalog::find_by_name(&name_en)
        .ok_or_else(|| format!("Mutation không tồn tại: {name_en}"))?;
    with_runtime(|runtime| {
        runtime.manual_name = Some(entry.name_en.clone());
        runtime.preview_until = None;
    });
    let mut status = MutationOverlayStatus::new("manual");
    status.name_en = Some(entry.name_en.clone());
    Ok(publish_status(&app, status))
}

#[tauri::command]
pub fn mutation_overlay_clear_manual(app: AppHandle) -> MutationOverlayStatus {
    with_runtime(|runtime| runtime.manual_name = None);
    clear_render(&app);
    publish_status(
        &app,
        MutationOverlayStatus::new("recognizing").message("Đang nhận diện"),
    )
}

#[tauri::command]
pub fn mutation_overlay_preview(
    app: AppHandle,
    name_en: String,
) -> Result<MutationOverlayStatus, String> {
    let entry = catalog::find_by_name(&name_en)
        .ok_or_else(|| format!("Mutation không tồn tại: {name_en}"))?;
    let payload = MutationOverlayPayload {
        name_en: entry.name_en.clone(),
        description_vi: entry.description_vi.clone(),
        confidence: None,
        source: "preview".to_string(),
    };

    if let Some((_hwnd, game_rect)) = active_game() {
        let _ = window::anchor(&app, game_rect, config(&app).rect);
    } else {
        window::preview_without_game(&app);
    }
    render(&app, payload);
    with_runtime(|runtime| runtime.preview_until = Some(Instant::now() + Duration::from_millis(PREVIEW_MS)));

    let mut status = MutationOverlayStatus::new("recognized").message("Đang hiện thử trong 5 giây");
    status.name_en = Some(entry.name_en.clone());
    Ok(publish_status(&app, status))
}

#[tauri::command]
pub fn mutation_overlay_begin_calibration(app: AppHandle) -> Result<MutationOverlayStatus, String> {
    let (_hwnd, game_rect) = active_game()
        .ok_or_else(|| "Hãy mở The Isle và màn hình Mutations trước khi căn chỉnh".to_string())?;
    let rect = config(&app).rect;
    with_runtime(|runtime| {
        runtime.saved_rect = Some(rect);
        runtime.calibrating = true;
        runtime.preview_until = None;
    });
    window::begin_calibration(&app, game_rect, rect)?;
    Ok(publish_status(
        &app,
        MutationOverlayStatus::new("calibrating")
            .message("Kéo/resize khung rồi bấm LƯU VỊ TRÍ"),
    ))
}

#[tauri::command]
pub fn mutation_overlay_save_calibration(app: AppHandle) -> Result<MutationOverlayStatus, String> {
    let (_hwnd, game_rect) = active_game()
        .ok_or_else(|| "The Isle phải đang ở foreground để lưu căn chỉnh".to_string())?;
    let overlay_rect = window::current_client_rect(&app)
        .ok_or_else(|| "Không đọc được vị trí khung căn chỉnh".to_string())?;
    let rect = NormalizedRect::from_screen_rect(overlay_rect, game_rect);

    crate::commands::apply_settings_patch(
        &app,
        json!({
            "mutation_overlay": {
                "rect": { "x": rect.x, "y": rect.y, "w": rect.w, "h": rect.h }
            }
        }),
    );
    window::end_calibration(&app, game_rect, rect)?;
    with_runtime(|runtime| {
        runtime.calibrating = false;
        runtime.saved_rect = None;
    });
    Ok(publish_status(
        &app,
        MutationOverlayStatus::new("recognizing").message("Đã lưu vị trí — đang nhận diện"),
    ))
}

#[tauri::command]
pub fn mutation_overlay_cancel_calibration(
    app: AppHandle,
) -> Result<MutationOverlayStatus, String> {
    let (_hwnd, game_rect) = active_game()
        .ok_or_else(|| "The Isle phải đang ở foreground để huỷ căn chỉnh".to_string())?;
    let rect = with_runtime(|runtime| {
        runtime.calibrating = false;
        runtime.saved_rect.take().unwrap_or_else(|| config(&app).rect)
    });
    window::end_calibration(&app, game_rect, rect)?;
    Ok(publish_status(
        &app,
        MutationOverlayStatus::new("recognizing").message("Đã huỷ căn chỉnh"),
    ))
}
