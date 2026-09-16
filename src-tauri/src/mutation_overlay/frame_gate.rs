use std::time::{Duration, Instant};

use super::capture::GrayFrame;

#[derive(Debug)]
pub struct FrameGate {
    min_interval: Duration,
    unchanged_retry_interval: Duration,
    last_attempt: Option<Instant>,
    last_ocr: Option<Instant>,
    last_hash: Option<u64>,
}

impl Default for FrameGate {
    fn default() -> Self {
        Self::new(Duration::from_millis(500), Duration::from_secs(2))
    }
}

impl FrameGate {
    pub fn new(min_interval: Duration, unchanged_retry_interval: Duration) -> Self {
        Self {
            min_interval,
            unchanged_retry_interval,
            last_attempt: None,
            last_ocr: None,
            last_hash: None,
        }
    }

    pub fn should_ocr(&mut self, frame: &GrayFrame, now: Instant) -> bool {
        if self
            .last_attempt
            .is_some_and(|previous| now.saturating_duration_since(previous) < self.min_interval)
        {
            return false;
        }
        self.last_attempt = Some(now);

        let hash = perceptual_hash(frame);
        let unchanged = self
            .last_hash
            .is_some_and(|previous| hamming_distance(previous, hash) <= 3);

        if unchanged
            && self.last_ocr.is_some_and(|previous| {
                now.saturating_duration_since(previous) < self.unchanged_retry_interval
            })
        {
            return false;
        }

        // Record the latest frame whenever OCR is actually allowed. This means
        // a static Mutation screen is retried periodically instead of getting
        // stuck forever after one transient/empty Windows OCR result.
        self.last_hash = Some(hash);
        self.last_ocr = Some(now);
        true
    }

    pub fn reset(&mut self) {
        self.last_attempt = None;
        self.last_ocr = None;
        self.last_hash = None;
    }
}

fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

fn perceptual_hash(frame: &GrayFrame) -> u64 {
    if frame.width == 0 || frame.height == 0 || frame.pixels.is_empty() {
        return 0;
    }
    let mut cells = [0u32; 64];
    let mut counts = [0u32; 64];
    let width = frame.width as usize;
    let height = frame.height as usize;
    for y in 0..height {
        let cy = (y * 8 / height).min(7);
        for x in 0..width {
            let cx = (x * 8 / width).min(7);
            let cell = cy * 8 + cx;
            let index = y * width + x;
            if let Some(value) = frame.pixels.get(index) {
                cells[cell] += *value as u32;
                counts[cell] += 1;
            }
        }
    }
    let averages: [u8; 64] = std::array::from_fn(|i| {
        if counts[i] == 0 {
            0
        } else {
            (cells[i] / counts[i]) as u8
        }
    });
    let mean = averages.iter().map(|value| *value as u32).sum::<u32>() / 64;
    averages.iter().enumerate().fold(0u64, |hash, (index, value)| {
        if *value as u32 >= mean {
            hash | (1u64 << index)
        } else {
            hash
        }
    })
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::FrameGate;
    use crate::mutation_overlay::capture::GrayFrame;

    fn frame(seed: u8) -> GrayFrame {
        GrayFrame {
            width: 16,
            height: 16,
            pixels: (0..256)
                .map(|index| seed.wrapping_add((index * 17) as u8))
                .collect(),
        }
    }

    #[test]
    fn identical_frames_are_skipped_between_periodic_retries() {
        let start = Instant::now();
        let mut gate = FrameGate::default();
        let sample = frame(3);
        assert!(gate.should_ocr(&sample, start));
        assert!(!gate.should_ocr(&sample, start + Duration::from_millis(600)));
        assert!(!gate.should_ocr(&sample, start + Duration::from_millis(1500)));
        assert!(gate.should_ocr(&sample, start + Duration::from_millis(2100)));
    }

    #[test]
    fn two_hertz_limit_applies_even_when_pixels_change() {
        let start = Instant::now();
        let mut gate = FrameGate::default();
        assert!(gate.should_ocr(&frame(1), start));
        assert!(!gate.should_ocr(&frame(80), start + Duration::from_millis(200)));
    }

    #[test]
    fn materially_changed_frame_runs_after_interval() {
        let start = Instant::now();
        let mut gate = FrameGate::default();
        assert!(gate.should_ocr(&frame(1), start));
        assert!(gate.should_ocr(&frame(80), start + Duration::from_millis(600)));
    }

    #[test]
    fn reset_allows_immediate_retry() {
        let start = Instant::now();
        let mut gate = FrameGate::default();
        let sample = frame(9);
        assert!(gate.should_ocr(&sample, start));
        gate.reset();
        assert!(gate.should_ocr(&sample, start + Duration::from_millis(10)));
    }
}
