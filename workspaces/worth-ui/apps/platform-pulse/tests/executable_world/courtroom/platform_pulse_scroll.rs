use crate::adjudication::physical_px;
use crate::product_process::PlatformPulseScrollJourneyEvidence;

use super::platform_pulse_cleanup::close_recovered_at_sequence;
use super::platform_pulse_dashboard::launch_dashboard_at_rest;

/// Physical pixels of slack for one snapped content edge.
const SHIFT_TOLERANCE_PX: i64 = 1;
/// The whole journey: geometry, hit testing and the timed notches after them.
const JOURNEY_BUDGET: std::time::Duration = std::time::Duration::from_secs(150);
/// The milestone's p95 budget from input to the first pixel that moves.
const FIRST_CHANGE_BUDGET: std::time::Duration = std::time::Duration::from_millis(50);
/// No notch may stall past this, however good the percentiles look.
const FIRST_CHANGE_STALL: std::time::Duration = std::time::Duration::from_millis(100);
/// The milestone's p95 and p99 bounds on the gap between accepted visible frames.
const FRAME_GAP_BUDGET: std::time::Duration = std::time::Duration::from_millis(25);
const FRAME_GAP_TAIL: std::time::Duration = std::time::Duration::from_millis(50);
/// No gap may pass this while motion or input requires progress.
const FRAME_GAP_STALL: std::time::Duration = std::time::Duration::from_millis(100);
/// The declared 120 ms wheel settle plus two 60 Hz display intervals.
const SETTLEMENT_BOUND: std::time::Duration = std::time::Duration::from_millis(154);

#[test]
fn native_wheel_notch_and_thumb_drag_move_recent_activity_by_declared_geometry() {
    let ready = launch_dashboard_at_rest();
    let journey_started = ready.native_journey_started();
    let completed = ready.complete_scroll_journey().unwrap_or_else(|failure| {
        panic!("native wheel, thumb drag and post-move hit-testing must remain causal: {failure}")
    });
    assert_scroll_geometry(completed.evidence());
    assert_hit_testing(completed.evidence());
    assert_wheel_latency(completed.evidence());
    let timing = completed.evidence().latency().clone();
    let shutdown_sequence = completed.evidence().expected_shutdown_sequence();
    let closed = close_recovered_at_sequence(completed.into_ready(), shutdown_sequence);
    timing.assert_accepted_samples(closed.evidence().native_close_evidence());
    assert!(closed.evidence().successful_exit().status().success());
    let elapsed = journey_started.elapsed();
    assert!(
        elapsed <= JOURNEY_BUDGET,
        "the native scroll journey must finish within {JOURNEY_BUDGET:?}; elapsed={elapsed:?}"
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

/// The milestone's timing criteria, on the intervals actually measured.
fn assert_wheel_latency(evidence: &PlatformPulseScrollJourneyEvidence) {
    let latency = evidence.latency();
    let report = latency.report();
    println!("{report}");
    assert_eq!(latency.trace_spans().len(), 3);
    assert!(latency
        .trace_spans()
        .iter()
        .all(|span| *span >= std::time::Duration::from_secs(10)));
    // Input time is conservatively the start of the actual SendInput bracket.
    // DXGI supplies desktop timestamps: acquisition/copy lag is reported but
    // cannot quantize visible intervals as the old synchronous GDI probe did.
    let uncertainty = latency.delivery_uncertainty();
    assert!(
        uncertainty < FIRST_CHANGE_BUDGET,
        "input delivery bracket {uncertainty:?} exceeds the first-change budget -- {report}"
    );
    let p95 = latency.first_change_percentile(950);
    assert!(
        p95 <= FIRST_CHANGE_BUDGET,
        "p95 input to first visible change is {p95:?}, over the {FIRST_CHANGE_BUDGET:?} budget -- {report}"
    );
    assert!(
        worst(latency.first_change()) <= FIRST_CHANGE_STALL,
        "a wheel notch took {:?} to reach the screen, past the {FIRST_CHANGE_STALL:?} stall bound -- {report}",
        worst(latency.first_change())
    );
    assert!(
        latency.frame_gap_percentile(950) <= FRAME_GAP_BUDGET,
        "p95 accepted visible-frame gap is {:?}, over the {FRAME_GAP_BUDGET:?} budget -- {report}",
        latency.frame_gap_percentile(950)
    );
    assert!(
        latency.frame_gap_percentile(990) <= FRAME_GAP_TAIL,
        "p99 accepted visible-frame gap is {:?}, over the {FRAME_GAP_TAIL:?} budget -- {report}",
        latency.frame_gap_percentile(990)
    );
    assert!(
        worst(latency.frame_gaps()) <= FRAME_GAP_STALL,
        "a frame gap of {:?} passed the {FRAME_GAP_STALL:?} bound while motion required progress -- {report}",
        worst(latency.frame_gaps())
    );
    assert!(
        worst(latency.settlement()) <= SETTLEMENT_BOUND,
        "the list settled {:?} after the notch, past the {SETTLEMENT_BOUND:?} bound -- {report}",
        worst(latency.settlement())
    );
}

fn worst(samples: &[std::time::Duration]) -> std::time::Duration {
    samples.iter().copied().max().unwrap_or_default()
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
