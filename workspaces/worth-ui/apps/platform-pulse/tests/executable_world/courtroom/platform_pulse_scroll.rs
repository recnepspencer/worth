use crate::adjudication::physical_px;
use crate::native_platform::certified_physical_extent;
use crate::product_process::{
    PlatformPulseNativeSampleFrameEvidence, PlatformPulseScrollJourneyEvidence,
};

use super::platform_pulse_cleanup::close_recovered_at_sequence;
use super::platform_pulse_dashboard::launch_dashboard_at_rest;

/// Physical pixels of slack for one snapped content edge.
const SHIFT_TOLERANCE_PX: i64 = 1;
#[test]
fn native_wheel_notch_and_thumb_drag_move_recent_activity_by_declared_geometry() {
    let ready = launch_dashboard_at_rest();
    let completed = ready.complete_scroll_journey().unwrap_or_else(|failure| {
        panic!("native wheel, thumb drag and post-move hit-testing must remain causal: {failure}")
    });
    assert_scroll_geometry(completed.evidence());
    assert_hit_testing(completed.evidence());
    let shutdown_sequence = completed.evidence().expected_shutdown_sequence();
    let closed = close_recovered_at_sequence(completed.into_ready(), shutdown_sequence);
    assert_accepted_motion_progress(closed.evidence().native_close_evidence().sample_frames());
    assert!(closed.evidence().successful_exit().status().success());
}

fn assert_scroll_geometry(evidence: &PlatformPulseScrollJourneyEvidence) {
    let lines = evidence.lines_per_notch();
    assert!(
        lines >= 1,
        "the platform wheel setting names at least one line per notch"
    );
    assert_eq!(evidence.notch_points(), f64::from(lines) * 20.0);
    // The content moved by exactly the platform's lines times the declared 20-pt
    // line extent, at the observed DPI; the oracle already matched the shifted
    // column, so what remains is that the matched shift is the declared one.
    let wheel = evidence.wheel_shift();
    assert!(wheel.compared_rows() > 0);
    let expected_shift_px = physical_px(evidence.notch_points(), evidence.dpi());
    assert!(
        (wheel.shift_px() - expected_shift_px).abs() <= SHIFT_TOLERANCE_PX,
        "wheel shift {} px is not one notch ({expected_shift_px} px) at {} DPI",
        wheel.shift_px(),
        evidence.dpi()
    );
    assert!(wheel.mismatched_rows() * 10 <= wheel.compared_rows());
    // The drag carried the list further than the notch did, and the thumb
    // followed the pointer down the track.
    assert!(evidence.drag_offset_points() > evidence.notch_points());
    let drag = evidence.drag_shift();
    assert!(drag.shift_px() > wheel.shift_px());
    assert!(evidence.drag_thumb().top_px() > evidence.wheel_thumb().top_px());
    assert!(evidence.wheel_thumb().length_px() >= 24);
    assert_eq!(drag.column_px(), wheel.column_px());
}

fn assert_accepted_motion_progress(samples: &[PlatformPulseNativeSampleFrameEvidence]) {
    let [width, height] = certified_physical_extent([1_536, 1_024]);
    let complete_client_pixels = u64::from(width) * u64::from(height);
    assert!(
        samples.len() >= 2,
        "scroll must accept multiple motion samples"
    );
    assert!(
        samples.windows(2).any(|pair| {
            pair[0].frame() == pair[1].frame()
                && pair[0].presentation_epoch() < pair[1].presentation_epoch()
        }),
        "scroll must progress on presentation-only samples"
    );
    assert!(
        samples.iter().all(|sample| {
            sample.logical_damage_regions() > 0
                && sample.rendered_pixels() > 0
                && sample.rendered_pixels() < complete_client_pixels
                && sample.queue_submissions() == 1
                && sample.presents() == 1
                && sample.presentation_epoch() == sample.presentation_attempt()
        }),
        "each accepted scroll sample must carry bounded presentation work"
    );
    let vertical_tops: Vec<_> = samples
        .iter()
        .flat_map(|sample| sample.sampled_chrome())
        .filter(|chrome| chrome.thumb() && !chrome.inline())
        .map(|chrome| chrome.bounds_milli()[1])
        .collect();
    assert!(
        vertical_tops.windows(2).any(|pair| pair[0] != pair[1]),
        "accepted motion must move vertical thumb geometry, not just increment counters"
    );
}

fn assert_hit_testing(evidence: &PlatformPulseScrollJourneyEvidence) {
    let [band0, band1, band2] = evidence.resting_hits();
    let [after_wheel, after_drag] = evidence.hits_after_motion();
    assert_ne!(band0, band1);
    assert_ne!(band1, band2);
    assert_eq!(
        after_wheel, band1,
        "one notch moved the list a full row, so the same screen point hits the next row"
    );
    assert_eq!(
        after_drag, band2,
        "the drag moved the list two rows, so the same screen point hits the row after"
    );
}
