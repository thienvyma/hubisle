//! Tests for the tracker logic. The original Python app had none — these pin
//! the behaviors documented in tracker.py's comments before the port gets
//! built on.

use overlay_core::calibration::Calibration;
use overlay_core::tracker::{
    HeadingSource, PositionTracker, TrailConfig, HEADING_MAX_AGE_S, LOCAL_HEADING_MAX_AGE_S,
    SOURCE_HEADING_MAX_AGE_S,
};

fn tracker() -> PositionTracker {
    PositionTracker::new(Calibration::gateway().clone(), TrailConfig::default())
}

#[test]
fn first_sample_starts_a_segment_with_one_node() {
    let mut t = tracker();
    let out = t.add_sample(1000.0, 2000.0, 0.0, 0.0);
    assert!(out.trail_changed);
    assert!(out.broke_segment);
    assert!(!out.refreshed_only);
    assert_eq!(t.segments, vec![vec![(1000.0, 2000.0)]]);
    assert!(t.current.is_some());
    assert!(t.previous.is_none());
}

#[test]
fn respawn_does_not_inherit_camera_or_movement_heading() {
    let mut t = tracker();
    t.add_sample_with_heading(0.0, 0.0, 0.0, Some(45.0), 0.0);
    t.update_local_heading(90.0, 0.0);
    let out = t.add_sample(10_000_000.0, 0.0, 0.0, 1.0);
    assert!(out.broke_segment);
    assert!(t.previous.is_none());
    assert_eq!(t.heading(1.0), None);
    t.add_sample_with_heading(10_000_000.0, 0.0, 0.0, Some(180.0), 1.1);
    assert_eq!(t.heading(1.1), Some(180.0));
}

#[test]
fn same_spot_only_refreshes_timestamp() {
    let mut t = tracker();
    t.add_sample(1000.0, 2000.0, 0.0, 0.0);
    // < 1 cm away: timestamp refresh only, no node, nothing written.
    let out = t.add_sample(1000.0, 2000.5, 0.0, 100.0);
    assert!(out.refreshed_only);
    assert!(!out.trail_changed);
    assert_eq!(t.segments, vec![vec![(1000.0, 2000.0)]]);
    assert_eq!(t.current.unwrap().at_s, 100.0, "timestamp must refresh");
    assert!(t.previous.is_none(), "refresh must not rotate previous");
}

#[test]
fn small_move_updates_position_without_new_node() {
    let mut t = tracker();
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    // 3 m: above refresh epsilon, below min_node_m (5 m).
    let out = t.add_sample(300.0, 0.0, 0.0, 10.0);
    assert!(!out.refreshed_only);
    assert!(!out.trail_changed);
    assert_eq!(t.segments, vec![vec![(0.0, 0.0)]]);
    assert!(t.previous.is_some());
}

#[test]
fn normal_move_appends_node_to_current_segment() {
    let mut t = tracker();
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    let out = t.add_sample(10_000.0, 0.0, 0.0, 10.0); // 100 m
    assert!(out.trail_changed);
    assert!(!out.broke_segment);
    assert_eq!(t.segments, vec![vec![(0.0, 0.0), (10_000.0, 0.0)]]);
}

#[test]
fn long_jump_breaks_segment_and_starts_new_one_at_the_point() {
    let mut t = tracker();
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    // 300 m > break_after_m (200 m): new segment starts AT this point so the
    // first point of the new leg is not lost.
    let out = t.add_sample(30_000.0, 0.0, 0.0, 10.0);
    assert!(out.broke_segment);
    assert!(out.trail_changed);
    assert_eq!(t.segments, vec![vec![(0.0, 0.0)], vec![(30_000.0, 0.0)]]);
}

#[test]
fn long_time_gap_breaks_segment() {
    let mut t = tracker();
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    // 100 m moved but 16 minutes elapsed (> break_after_minutes = 15).
    let out = t.add_sample(10_000.0, 0.0, 0.0, 16.0 * 60.0);
    assert!(out.broke_segment);
    assert_eq!(t.segments, vec![vec![(0.0, 0.0)], vec![(10_000.0, 0.0)]]);
}

#[test]
fn heading_needs_two_samples_and_enough_distance() {
    let mut t = tracker();
    assert_eq!(t.heading(0.0), None);
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    assert_eq!(t.heading(0.0), None, "one sample is not a direction");
    // 0.5 m: ignore coordinate jitter.
    t.add_sample(50.0, 0.0, 0.0, 10.0);
    assert_eq!(t.heading(10.0), None);
    // A normal 3 m step must already rotate the realtime marker. Waiting for
    // the old 20 m threshold made walking players look stationary.
    t.add_sample(350.0, 0.0, 0.0, 20.0);
    let h = t.heading(20.0).expect("heading must be available");
    assert!((h - 180.0).abs() < 1e-6, "got {h}");
}

#[test]
fn high_frequency_small_steps_accumulate_into_trail_nodes() {
    let mut t = tracker();
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    for step in 1..=12 {
        t.add_sample(step as f64 * 50.0, 0.0, 0.0, step as f64 * 0.05);
    }
    assert!(
        t.segments[0].len() >= 2,
        "20 Hz samples must still produce a trail after five accumulated metres"
    );
}

#[test]
fn provider_heading_is_available_without_movement_and_normalized() {
    let mut t = tracker();
    t.add_sample_with_heading(1000.0, 2000.0, 0.0, Some(-10.0), 5.0);
    assert_eq!(t.heading(5.0), Some(350.0));

    // A fresh provider update at the same coordinate may still carry a new
    // camera/character direction and must rotate the marker immediately.
    t.add_sample_with_heading(1000.0, 2000.0, 0.0, Some(725.0), 10.0);
    assert_eq!(t.heading(10.0), Some(5.0));
}

#[test]
fn local_camera_heading_updates_without_touching_server_position_or_trail() {
    let mut t = tracker();
    t.add_sample(1000.0, 2000.0, 3000.0, 5.0);
    let current = t.current;
    let segments = t.segments.clone();

    t.update_local_heading(-10.0, 6.0);

    assert_eq!(
        t.current, current,
        "heading-only input must not move the marker"
    );
    assert_eq!(
        t.segments, segments,
        "heading-only input must not write trail nodes"
    );
    assert_eq!(
        t.heading_with_source(6.0),
        Some((350.0, HeadingSource::LocalCamera))
    );
}

#[test]
fn local_camera_wins_while_fresh_then_falls_back_to_provider_camera() {
    let mut t = tracker();
    t.add_sample_with_heading(1000.0, 2000.0, 0.0, Some(90.0), 5.0);
    t.update_local_heading(180.0, 10.0);
    assert_eq!(
        t.heading_with_source(10.0),
        Some((180.0, HeadingSource::LocalCamera))
    );

    // Refresh the server camera after the local adapter has gone quiet. Once
    // the local sample expires, the exact provider value becomes authoritative
    // again instead of dropping straight to movement-derived direction.
    t.add_sample_with_heading(1000.0, 2000.0, 0.0, Some(120.0), 20.0);
    assert_eq!(
        t.heading_with_source(10.0 + SOURCE_HEADING_MAX_AGE_S + 1.0),
        Some((120.0, HeadingSource::ProviderCamera))
    );
}

#[test]
fn local_heading_expires_independently_without_another_position_packet() {
    let mut t = tracker();
    t.add_sample_with_heading(1000.0, 2000.0, 0.0, Some(90.0), 5.0);
    let position = t.current;
    let trail = t.segments.clone();
    t.update_local_heading(180.0, 5.0);
    assert_eq!(
        t.heading_with_source(5.0 + LOCAL_HEADING_MAX_AGE_S),
        Some((180.0, HeadingSource::LocalCamera))
    );
    assert_eq!(
        t.heading_with_source(5.0 + LOCAL_HEADING_MAX_AGE_S + 0.001),
        Some((90.0, HeadingSource::ProviderCamera))
    );
    assert_eq!(
        t.heading_with_source(5.0 + SOURCE_HEADING_MAX_AGE_S + 0.001),
        None
    );
    assert_eq!(t.current, position);
    assert_eq!(t.segments, trail);
}

#[test]
fn local_heading_can_exist_before_position_and_can_be_cleared_without_it() {
    let mut t = tracker();
    t.update_local_heading(270.0, 1.0);
    assert_eq!(
        t.heading_with_source(1.05),
        Some((270.0, HeadingSource::LocalCamera))
    );
    assert!(t.current.is_none());
    assert!(t.segments.is_empty());
    t.clear_local_heading();
    assert_eq!(t.heading_with_source(1.06), None);
}

#[test]
fn delayed_or_invalid_local_frames_cannot_replace_a_newer_bearing() {
    let mut t = tracker();
    t.update_local_heading(90.0, 2.0);
    t.update_local_heading(180.0, 1.0);
    t.update_local_heading(f64::NAN, 2.05);
    t.update_local_heading(180.0, f64::NAN);
    assert_eq!(t.heading(2.1), Some(90.0));
    assert_eq!(t.heading(1.9), None);
    assert_eq!(t.heading(f64::NAN), None);
}

#[test]
fn movement_heading_reports_its_estimated_source() {
    let mut t = tracker();
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    t.add_sample(0.0, 10_000.0, 0.0, 10.0);
    assert_eq!(
        t.heading_with_source(10.0),
        Some((90.0, HeadingSource::Movement))
    );
}

#[test]
fn stale_provider_heading_does_not_freeze_the_arrow() {
    let mut t = tracker();
    t.add_sample_with_heading(1000.0, 2000.0, 0.0, Some(90.0), 5.0);
    assert_eq!(
        t.heading(5.0 + SOURCE_HEADING_MAX_AGE_S + 1.0),
        None,
        "a disconnected live stream must not leave an exact-looking stale arrow"
    );
}

#[test]
fn clearing_position_also_clears_provider_heading() {
    let mut t = tracker();
    t.add_sample_with_heading(1000.0, 2000.0, 0.0, Some(90.0), 5.0);
    t.clear_position();
    assert_eq!(t.current, None);
    assert_eq!(t.heading(5.0), None);
}

#[test]
fn heading_expires_after_max_age() {
    let mut t = tracker();
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    t.add_sample(10_000.0, 0.0, 0.0, 10.0);
    assert!(t.heading(10.0).is_some());
    assert_eq!(
        t.heading(10.0 + HEADING_MAX_AGE_S + 1.0),
        None,
        "a stale sample must not keep an arrow pointing"
    );
}

#[test]
fn bearing_to_reports_bearing_and_metres() {
    let mut t = tracker();
    assert_eq!(t.bearing_to(0.0, 0.0), None);
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    let (bearing, dist) = t.bearing_to(0.0, 50_000.0).unwrap();
    assert!((bearing - 90.0).abs() < 1e-6, "east, got {bearing}");
    assert!((dist - 500.0).abs() < 1e-6, "500 m, got {dist}");
}

#[test]
fn clear_trail_resets_segments() {
    let mut t = tracker();
    t.add_sample(0.0, 0.0, 0.0, 0.0);
    t.add_sample(10_000.0, 0.0, 0.0, 10.0);
    t.clear_trail();
    assert_eq!(t.segments, vec![Vec::<(f64, f64)>::new()]);
}
