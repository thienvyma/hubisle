//! Player position tracking over discrete samples. Port of `app/tracker.py`
//! with the Qt signals and disk writer factored out: `add_sample` returns a
//! [`SampleOutcome`] describing what happened, and the caller (the tauri app)
//! writes the trail JSONL and emits events from that.
//!
//! Samples are IRREGULAR by nature: the player copies coordinates whenever
//! they feel like it — two consecutive samples can be 5 seconds or 2 hours
//! apart. Everything here is designed around that fact:
//!
//!   - Heading is derived from the vector between the last two samples, and
//!     only trusted when they are close enough in time and far enough apart in
//!     distance. An arrow pointing the wrong way is worse than no arrow.
//!   - The trail breaks into segments when the gap in distance or time is too
//!     large, instead of drawing a straight line across it.

use crate::calibration::Calibration;
use crate::coords::{bearing_deg, distance_m};

/// Confidence thresholds for the heading arrow.
pub const HEADING_MIN_DISTANCE_M: f64 = 1.0;
pub const HEADING_MAX_AGE_S: f64 = 600.0; // 10 minutes
pub const SOURCE_HEADING_MAX_AGE_S: f64 = 15.0;

/// Re-copying the same spot only refreshes the timestamp below this distance.
pub const REFRESH_EPSILON_M: f64 = 0.01;

/// Where the displayed bearing came from. Camera sources are exact; movement
/// is the provider-neutral estimate used when no live camera angle is fresh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingSource {
    LocalCamera,
    ProviderCamera,
    Movement,
}

impl HeadingSource {
    pub const fn key(self) -> &'static str {
        match self {
            Self::LocalCamera => "local-camera",
            Self::ProviderCamera => "provider-camera",
            Self::Movement => "movement",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Seconds on whatever monotonic-enough clock the caller uses; only
    /// differences matter.
    pub at_s: f64,
}

impl Sample {
    pub fn age_s(&self, now_s: f64) -> f64 {
        now_s - self.at_s
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrailConfig {
    pub enabled: bool,
    pub break_after_s: f64,
    pub break_after_m: f64,
    pub min_node_m: f64,
}

impl Default for TrailConfig {
    fn default() -> Self {
        // Mirrors DEFAULT_SETTINGS["trail"] in the original config.py.
        Self {
            enabled: true,
            break_after_s: 15.0 * 60.0,
            break_after_m: 200.0,
            min_node_m: 5.0,
        }
    }
}

/// What one `add_sample` call did — drives the caller's JSONL writes and
/// change events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SampleOutcome {
    /// Same spot re-copied: only the timestamp was refreshed; no node was
    /// added and the sample must NOT be written to the trail file.
    pub refreshed_only: bool,
    /// A segment break happened — the caller writes a `break` record before
    /// the sample.
    pub broke_segment: bool,
    /// The visible trail changed (node added and/or segment broken).
    pub trail_changed: bool,
}

/// Holds the current position, the previous one, and the session trail.
#[derive(Debug, Clone)]
pub struct PositionTracker {
    cal: Calibration,
    config: TrailConfig,
    pub current: Option<Sample>,
    pub previous: Option<Sample>,
    /// Exact heading supplied by the selected server/provider.
    provider_heading: Option<(f64, f64)>,
    /// Exact heading supplied independently by a safe local game adapter.
    /// Kept separate from position so a 20 Hz camera source never creates
    /// fake trail nodes or changes the authoritative server coordinate.
    local_heading: Option<(f64, f64)>,
    /// Trail polylines in world cm; a new inner Vec starts after each break.
    pub segments: Vec<Vec<(f64, f64)>>,
}

impl PositionTracker {
    pub fn new(cal: Calibration, config: TrailConfig) -> Self {
        Self {
            cal,
            config,
            current: None,
            previous: None,
            provider_heading: None,
            local_heading: None,
            segments: Vec::new(),
        }
    }

    // -- receiving new samples --------------------------------------------

    pub fn add_sample(&mut self, x: f64, y: f64, z: f64, now_s: f64) -> SampleOutcome {
        self.add_sample_with_heading(x, y, z, None, now_s)
    }

    /// Add a coordinate and, when the provider exposes it, an exact north-up
    /// compass heading. A heading-only update at the same coordinate still
    /// refreshes the arrow immediately.
    pub fn add_sample_with_heading(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
        heading_deg: Option<f64>,
        now_s: f64,
    ) -> SampleOutcome {
        self.update_provider_heading_if_some(heading_deg, now_s);
        let sample = Sample {
            x,
            y,
            z,
            at_s: now_s,
        };
        let mut outcome = SampleOutcome::default();

        if let Some(current) = self.current {
            let moved = distance_m(current.x, current.y, x, y);
            if moved < REFRESH_EPSILON_M {
                // Re-copied the same spot: refresh the timestamp only.
                self.current = Some(sample);
                outcome.refreshed_only = true;
                return outcome;
            }

            let gap_s = now_s - current.at_s;
            if gap_s > self.config.break_after_s || moved > self.config.break_after_m {
                // Break the segment and start the new one AT this point — if
                // we waited for the next sample, the first point of the new
                // leg would be lost.
                self.start_new_segment();
                self.append_node(x, y);
                outcome.broke_segment = true;
                outcome.trail_changed = true;
            } else if self
                .segments
                .last()
                .and_then(|segment| segment.last())
                .map(|(last_x, last_y)| distance_m(*last_x, *last_y, x, y))
                .unwrap_or(moved)
                >= self.config.min_node_m
            {
                self.append_node(x, y);
                outcome.trail_changed = true;
            }
        } else {
            self.start_new_segment();
            self.append_node(x, y);
            outcome.broke_segment = true;
            outcome.trail_changed = true;
        }

        self.previous = self.current;
        if outcome.broke_segment {
            // A respawn/teleport must not use the dead character as the origin
            // for a movement heading. Keep only an exact angle in this sample.
            self.previous = None;
            self.provider_heading = None;
            self.local_heading = None;
            self.update_provider_heading_if_some(heading_deg, now_s);
        }
        self.current = Some(sample);
        outcome
    }

    fn start_new_segment(&mut self) {
        if matches!(self.segments.last(), Some(s) if s.is_empty()) {
            self.segments.pop();
        }
        self.segments.push(Vec::new());
    }

    fn append_node(&mut self, x: f64, y: f64) {
        if self.segments.is_empty() {
            self.segments.push(Vec::new());
        }
        self.segments.last_mut().unwrap().push((x, y));
    }

    fn update_provider_heading_if_some(&mut self, heading_deg: Option<f64>, now_s: f64) {
        if let Some(heading) = heading_deg.filter(|heading| heading.is_finite()) {
            self.provider_heading = Some((heading.rem_euclid(360.0), now_s));
        }
    }

    /// Update only the exact local camera bearing. This deliberately does not
    /// touch `current`, `previous`, or the trail: position remains owned by the
    /// authenticated server provider.
    pub fn update_local_heading(&mut self, heading_deg: f64, now_s: f64) {
        if heading_deg.is_finite() {
            self.local_heading = Some((heading_deg.rem_euclid(360.0), now_s));
        }
    }

    pub fn clear_trail(&mut self) {
        self.segments = vec![Vec::new()];
    }

    pub fn clear_position(&mut self) {
        self.current = None;
        self.previous = None;
        self.provider_heading = None;
        self.local_heading = None;
    }

    pub fn config(&self) -> &TrailConfig {
        &self.config
    }

    // -- derived state -----------------------------------------------------

    /// Compass bearing of travel, or None while not confident enough.
    pub fn heading(&self, now_s: f64) -> Option<f64> {
        self.heading_with_source(now_s).map(|(heading, _)| heading)
    }

    /// Heading together with its provenance. A fresh local camera adapter has
    /// first priority, then the server camera, then movement between server
    /// coordinates. Exact sources expire so a disconnected adapter cannot
    /// leave an authoritative-looking arrow frozen forever.
    pub fn heading_with_source(&self, now_s: f64) -> Option<(f64, HeadingSource)> {
        if let Some((heading, at_s)) = self.local_heading {
            if now_s - at_s <= SOURCE_HEADING_MAX_AGE_S {
                return Some((heading, HeadingSource::LocalCamera));
            }
        }
        if let Some((heading, at_s)) = self.provider_heading {
            if now_s - at_s <= SOURCE_HEADING_MAX_AGE_S {
                return Some((heading, HeadingSource::ProviderCamera));
            }
        }
        let current = self.current?;
        let previous = self.previous?;
        if current.age_s(now_s) > HEADING_MAX_AGE_S {
            return None;
        }
        let moved = distance_m(previous.x, previous.y, current.x, current.y);
        if moved < HEADING_MIN_DISTANCE_M {
            return None;
        }
        Some((
            bearing_deg(previous.x, previous.y, current.x, current.y, &self.cal),
            HeadingSource::Movement,
        ))
    }

    /// (bearing, distance in metres) from the current position to a point.
    pub fn bearing_to(&self, x: f64, y: f64) -> Option<(f64, f64)> {
        let current = self.current?;
        Some((
            bearing_deg(current.x, current.y, x, y, &self.cal),
            distance_m(current.x, current.y, x, y),
        ))
    }
}
