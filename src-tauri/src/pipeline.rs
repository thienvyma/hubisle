//! The one-way position flow: (clipboard | replay | debug) -> tracker -> both
//! windows. Port of the sample-handling wiring from the original `main.py`.

use overlay_core::{bearing_to_compass_key, world_to_pixel, Calibration};
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};
// Note: every px in these payloads is computed with state.active_calibration()
// at emit time — nothing px-shaped is cached, so a basemap switch only needs a
// resync to repaint everything in the new frame.
use tauri::{AppHandle, Manager};

use crate::events::{
    emit_all, HeadingUpdate, PositionUpdate, TrailPayload, HEADING_UPDATE, POSITION_CLEARED,
    POSITION_UPDATE, SETTINGS_CHANGED, TRAIL_CHANGED,
};
use crate::state::{AppState, LockExt};

static LAST_LOCAL_SAMPLE: LazyLock<Mutex<Option<Instant>>> = LazyLock::new(|| Mutex::new(None));
const LOCAL_POSITION_PRIORITY: Duration = Duration::from_secs(1);

/// Feed one accepted coordinate sample through the tracker and notify the UI.
pub fn ingest_sample(app: &AppHandle, x: f64, y: f64, z: f64) {
    ingest_sample_with_heading(app, x, y, z, None);
}

/// Feed a coordinate plus an exact provider heading through the same tracker
/// and UI event path. The fallback providers still call `ingest_sample` and
/// derive direction from successive positions.
pub fn ingest_sample_with_heading(
    app: &AppHandle,
    x: f64,
    y: f64,
    z: f64,
    heading_deg: Option<f64>,
) {
    if local_position_is_fresh() {
        return;
    }
    let state = app.state::<AppState>();
    let now_s = state.now_s();
    // Resolve the calibration BEFORE taking the tracker lock (active_calibration
    // briefly takes the settings lock).
    let cal = state.active_calibration();

    let (outcome, heading, heading_observed_at_ms, trail) = {
        let mut tracker = state.tracker.lock_safe();
        let outcome = tracker.add_sample_with_heading(x, y, z, heading_deg, now_s);
        let observed_at = state.now_s();
        let heading = tracker.heading_with_source(observed_at);
        let trail = outcome
            .trail_changed
            .then(|| trail_payload(&tracker.segments, cal));
        (outcome, heading, observed_at * 1000.0, trail)
    };

    // Persist AFTER releasing no locks out of order: trail writes follow the
    // same order the original used — break record first, then the sample.
    if !outcome.refreshed_only {
        if let Some(writer) = state.trail_writer.lock_safe().as_mut() {
            if outcome.broke_segment {
                writer.add_break();
            }
            writer.add(x, y, z);
        }
    }

    let (px, py) = world_to_pixel(x, y, cal);
    let payload = PositionUpdate {
        x_cm: x,
        y_cm: y,
        z_cm: z,
        px,
        py,
        heading_deg: heading.map(|(bearing, _)| bearing),
        heading_observed_at_ms,
        heading_source: heading.map(|(_, source)| source.key()),
        compass_key: heading.map(|(bearing, _)| bearing_to_compass_key(bearing)),
        in_bounds: overlay_core::is_in_bounds(px, py, cal),
    };
    emit_all(app, POSITION_UPDATE, payload);
    if let Some(trail) = trail {
        emit_all(app, TRAIL_CHANGED, trail);
    }
}

/// Feed the high-frequency position and exact camera bearing decoded by the
/// bundled local telemetry sidecar. Local bearing is kept separate so it wins
/// over delayed provider headings without changing their connection state.
pub fn ingest_local_sample_with_heading(app: &AppHandle, x: f64, y: f64, z: f64, heading_deg: f64) {
    *LAST_LOCAL_SAMPLE.lock_safe() = Some(Instant::now());
    let state = app.state::<AppState>();
    let now_s = state.now_s();
    let cal = state.active_calibration();

    let (outcome, heading, trail) = {
        let mut tracker = state.tracker.lock_safe();
        let outcome = tracker.add_sample(x, y, z, now_s);
        tracker.update_local_heading(heading_deg, now_s);
        let heading = tracker.heading_with_source(now_s);
        let trail = outcome
            .trail_changed
            .then(|| trail_payload(&tracker.segments, cal));
        (outcome, heading, trail)
    };

    if !outcome.refreshed_only {
        if let Some(writer) = state.trail_writer.lock_safe().as_mut() {
            if outcome.broke_segment {
                writer.add_break();
            }
            writer.add(x, y, z);
        }
    }

    let (px, py) = world_to_pixel(x, y, cal);
    emit_all(
        app,
        POSITION_UPDATE,
        PositionUpdate {
            x_cm: x,
            y_cm: y,
            z_cm: z,
            px,
            py,
            heading_deg: heading.map(|(bearing, _)| bearing),
            heading_observed_at_ms: now_s * 1000.0,
            heading_source: heading.map(|(_, source)| source.key()),
            compass_key: heading.map(|(bearing, _)| bearing_to_compass_key(bearing)),
            in_bounds: overlay_core::is_in_bounds(px, py, cal),
        },
    );
    if let Some(trail) = trail {
        emit_all(app, TRAIL_CHANGED, trail);
    }
}

fn local_position_is_fresh() -> bool {
    LAST_LOCAL_SAMPLE
        .lock_safe()
        .is_some_and(|observed_at| observed_at.elapsed() <= LOCAL_POSITION_PRIORITY)
}

/// Feed an exact camera bearing from a heading-only local adapter without
/// changing position or trail. The bundled UDP sidecar normally calls the
/// combined position/heading path above; this boundary remains useful for
/// frame-based adapters.
/// `captured_at_s` must use AppState's monotonic clock, NOT receive time for a
/// cached frame; a WGC adapter must translate its QPC timestamp first.
pub fn ingest_local_heading(app: &AppHandle, heading_deg: f64, captured_at_s: f64) {
    let state = app.state::<AppState>();
    let now_s = state.now_s();
    if !captured_at_s.is_finite()
        || !(0.0..=overlay_core::tracker::LOCAL_HEADING_MAX_AGE_S)
            .contains(&(now_s - captured_at_s))
    {
        return;
    }
    {
        state
            .tracker
            .lock_safe()
            .update_local_heading(heading_deg, captured_at_s);
    }
    emit_all(app, HEADING_UPDATE, current_heading(&state));
}

/// A hidden or occluded webview cannot be trusted to run an expiry timer.
/// Native ticks publish changes (including Some -> None) even if providers
/// are completely silent. No IPC is sent while the selected value is stable.
pub fn spawn_heading_watchdog(app: AppHandle) {
    std::thread::spawn(move || {
        let mut previous = None;
        loop {
            let payload = current_heading(&app.state::<AppState>());
            let value = (payload.heading_deg, payload.heading_source);
            if previous != Some(value) {
                previous = Some(value);
                emit_all(&app, HEADING_UPDATE, payload);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}

pub fn current_heading(state: &AppState) -> HeadingUpdate {
    let tracker = state.tracker.lock_safe();
    let now_s = state.now_s();
    let heading = tracker.heading_with_source(now_s);
    HeadingUpdate {
        heading_deg: heading.map(|(bearing, _)| bearing),
        heading_source: heading.map(|(_, source)| source.key()),
        compass_key: heading.map(|(bearing, _)| bearing_to_compass_key(bearing)),
        heading_observed_at_ms: now_s * 1000.0,
    }
}

/// A capture adapter must call this on loss of HUD/focus, not keep restamping
/// its cached angle. Coordinates and trail remain untouched.
pub fn clear_local_heading(app: &AppHandle) {
    let state = app.state::<AppState>();
    state.tracker.lock_safe().clear_local_heading();
    emit_all(app, HEADING_UPDATE, current_heading(&state));
}

/// Remove the active marker plus provider/movement heading while preserving
/// completed trail segments and any still-fresh local camera bearing. The
/// local adapter owns that bearing and clears it explicitly on capture loss.
/// The next accepted position starts a new segment, so switching servers can
/// never draw a line across unrelated positions.
pub fn clear_position(app: &AppHandle) {
    let state = app.state::<AppState>();
    let cal = state.active_calibration();
    let (trail, add_break) = {
        let mut tracker = state.tracker.lock_safe();
        let add_break = tracker.current.is_some()
            || tracker
                .segments
                .last()
                .is_some_and(|segment| !segment.is_empty());
        tracker.clear_position();
        if tracker
            .segments
            .last()
            .is_some_and(|segment| !segment.is_empty())
        {
            tracker.segments.push(Vec::new());
        }
        (trail_payload(&tracker.segments, cal), add_break)
    };
    if add_break {
        if let Some(writer) = state.trail_writer.lock_safe().as_mut() {
            writer.add_break();
        }
    }
    emit_all(app, POSITION_CLEARED, ());
    emit_all(app, HEADING_UPDATE, current_heading(&state));
    emit_all(app, TRAIL_CHANGED, trail);
}

/// The current tracker state as a PositionUpdate, or None before the first
/// sample. Shared by `resync` and the `get_current_position` command so a
/// freshly (re)loaded webview paints at once instead of waiting for the
/// player's next manual coordinate copy.
pub fn current_payload(state: &AppState) -> Option<PositionUpdate> {
    let cal = state.active_calibration();
    let (current, heading, heading_observed_at_ms) = {
        let tracker = state.tracker.lock_safe();
        let now_s = state.now_s();
        (
            tracker.current,
            tracker.heading_with_source(now_s),
            now_s * 1000.0,
        )
    };
    let cur = current?;
    let (px, py) = world_to_pixel(cur.x, cur.y, cal);
    Some(PositionUpdate {
        x_cm: cur.x,
        y_cm: cur.y,
        z_cm: cur.z,
        px,
        py,
        heading_deg: heading.map(|(bearing, _)| bearing),
        heading_observed_at_ms,
        heading_source: heading.map(|(_, source)| source.key()),
        compass_key: heading.map(|(bearing, _)| bearing_to_compass_key(bearing)),
        in_bounds: overlay_core::is_in_bounds(px, py, cal),
    })
}

/// Re-send the full current state to every window. Belt-and-braces: hidden
/// windows receive broadcasts and reloads fetch get_current_position, so
/// this mostly matters after a manual webview reload.
pub fn resync(app: &AppHandle) {
    let state = app.state::<AppState>();
    let cal = state.active_calibration();

    let trail = {
        let tracker = state.tracker.lock_safe();
        trail_payload(&tracker.segments, cal)
    };
    if let Some(payload) = current_payload(&state) {
        emit_all(app, POSITION_UPDATE, payload);
    }
    emit_all(app, HEADING_UPDATE, current_heading(&state));
    emit_all(app, TRAIL_CHANGED, trail);
    {
        let settings = state.settings.lock_safe().clone();
        emit_all(app, SETTINGS_CHANGED, settings);
    }
    emit_all(app, "waypoints://changed", ());
    crate::islepilot::emit_last(app);
    emit_all(
        app,
        crate::providers::orchestrator::PROVIDER_STATE,
        crate::providers::orchestrator::current_state(),
    );
    if let Some(snapshot) = crate::providers::orchestrator::current_snapshot() {
        emit_all(
            app,
            crate::providers::orchestrator::PROVIDER_SNAPSHOT,
            snapshot,
        );
    }
}

pub fn trail_payload(segments_cm: &[Vec<(f64, f64)>], cal: &Calibration) -> TrailPayload {
    TrailPayload {
        segments_cm: segments_cm.to_vec(),
        segments_px: segments_cm
            .iter()
            .map(|seg| {
                seg.iter()
                    .map(|&(x, y)| world_to_pixel(x, y, cal))
                    .collect()
            })
            .collect(),
    }
}
