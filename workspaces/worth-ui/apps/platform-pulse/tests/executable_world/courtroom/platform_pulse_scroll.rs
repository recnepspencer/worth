use crate::adjudication::physical_px;
use crate::product_process::PlatformPulseScrollJourneyEvidence;

use super::platform_pulse_cleanup::close_recovered_at_sequence;
use super::platform_pulse_dashboard::launch_dashboard_at_rest;

/// Physical pixels of slack for one snapped content edge.
const SHIFT_TOLERANCE_PX: i64 = 1;

#[test]
fn native_wheel_notch_and_thumb_drag_move_recent_activity_by_declared_geometry() {
    let ready = launch_dashboard_at_rest();
    let journey_started = ready.native_journey_started();
    let completed = ready.complete_scroll_journey().unwrap_or_else(|failure| {
        panic!("native wheel, thumb drag and post-move hit-testing must remain causal: {failure}")
    });
    assert_scroll_geometry(completed.evidence());
    assert_hit_testing(completed.evidence());
    let shutdown_sequence = completed.evidence().expected_shutdown_sequence();
    let closed = close_recovered_at_sequence(completed.into_ready(), shutdown_sequence);
    assert!(closed.evidence().successful_exit().status().success());
    let elapsed = journey_started.elapsed();
    assert!(
        elapsed <= std::time::Duration::from_secs(45),
        "the native scroll journey must finish within 45 seconds; elapsed={elapsed:?}"
    );
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
