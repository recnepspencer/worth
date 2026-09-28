use super::*;
use crate::analysis::Latency;
use crate::coverage::Coverage;
use crate::gaps::Gap;
use crate::logs::Grip;

#[test]
fn percentiles_use_the_nearest_rank() {
    let values: Vec<f64> = (1..=100).map(f64::from).collect();
    assert_eq!(percentile(&values, 0.95), Some(95.0));
    assert_eq!(percentile(&values, 0.99), Some(99.0));
    assert_eq!(percentile(&values, 1.0), Some(100.0));
    assert_eq!(percentile(&[7.0], 0.5), Some(7.0));
    assert_eq!(percentile(&[], 0.5), None);
    assert_eq!(percentile(&[1.0, f64::INFINITY], 0.95), Some(f64::INFINITY));
}

fn capture(complete: bool, refresh_hz: u32) -> CaptureLog {
    CaptureLog {
        frequency: 1000,
        refresh_hz,
        dpi: 96,
        windows_build: "26200.1".to_owned(),
        grip: Grip::Unknown,
        samples: Vec::new(),
        complete,
    }
}

/// A run that passes every grade, sampled every `interval` ms.
fn passing(interval: f64) -> Analysis {
    Analysis {
        drag_ms: 12_000.0,
        moving_ms: 11_000.0,
        coverage: Coverage {
            smallest: [800, 600],
            largest: [1536, 1024],
            reached_small: true,
            reached_large: true,
            reversals: 3,
            narrowing: 1,
            widening: 1,
        },
        latencies: vec![Latency {
            extent: [800, 600],
            observed: 0.0,
            visible: 20.0,
        }],
        gaps: vec![Gap {
            start: 0.0,
            end: 17.0,
            active: Some(17.0),
        }],
        final_extent: [800, 600],
        final_ms: Some(10.0),
        sample_intervals: vec![interval; 10],
        ..Analysis::default()
    }
}

#[test]
fn the_report_lists_only_the_stages_traced() {
    let mut analysis = passing(16.0);
    analysis.work.stage_ms[8] = vec![4.0, 2.0, 6.0];
    analysis.work.swapchains = vec![[1024, 768]];
    let report = render(&analysis, &capture(true, 60)).0;
    assert!(report.contains("  draw: 3: 4.0 ms, 6.0 ms, 6.0 ms, 12.0 ms\n"));
    assert!(report.contains("swapchain configurations during the drag: 1 [[1024, 768]]\n"));
    assert!(!report.contains("  frame: "));
}

#[test]
fn the_report_summarizes_presentation_work_per_frame() {
    let mut analysis = passing(16.0);
    analysis.work.presentation = (1..=20)
        .map(|frame| {
            let mut counts = [0; crate::work::PRESENTATION_WORK.len()];
            counts[0] = frame;
            counts
        })
        .collect();
    let report = render(&analysis, &capture(true, 60)).0;
    assert!(report.contains("presentation work over 20 submitted frames"));
    assert!(report.contains("  demand glyphs: 10, 19, 20, 210\n"));
    assert!(report.contains("  vertex glyphs: 0, 0, 0, 0\n"));
    assert!(report.contains("  allocations: not counted (build Pulse with"));

    analysis.work.allocations = vec![5, 1, 3];
    let report = render(&analysis, &capture(true, 60)).0;
    assert!(report.contains("  allocations: 3, 5, 5, 9\n"));
}

#[test]
fn a_run_meeting_every_threshold_passes() {
    assert_eq!(render(&passing(16.0), &capture(true, 60)).1, Verdict::Pass);
}

#[test]
fn only_sighting_grades_turn_inconclusive_on_a_coarse_capture() {
    let slow = Analysis {
        final_ms: Some(400.0),
        ..passing(40.0)
    };
    assert_eq!(render(&slow, &capture(true, 60)).1, Verdict::Inconclusive);
    let slow = Analysis {
        final_ms: Some(400.0),
        ..passing(16.0)
    };
    assert_eq!(render(&slow, &capture(true, 60)).1, Verdict::Fail);
    assert_eq!(
        render(&passing(16.0), &capture(false, 60)).1,
        Verdict::Inconclusive
    );

    let short = Analysis {
        moving_ms: 4_000.0,
        ..passing(40.0)
    };
    assert_eq!(render(&short, &capture(true, 60)).1, Verdict::Fail);
    let unknown = Analysis {
        unknown: 1,
        ..passing(40.0)
    };
    assert_eq!(render(&unknown, &capture(false, 60)).1, Verdict::Fail);
}

#[test]
fn a_display_other_than_60_hz_fails() {
    let (text, verdict) = render(&passing(8.0), &capture(true, 120));
    assert_eq!(verdict, Verdict::Fail);
    assert!(text.contains("FAIL  display refresh: 120 Hz"));
}

#[test]
fn a_breakpoint_crossed_one_way_fails() {
    let mut analysis = passing(16.0);
    analysis.coverage.narrowing = 0;
    assert_eq!(render(&analysis, &capture(true, 60)).1, Verdict::Fail);
}

#[test]
fn extents_never_presented_count_as_latency_misses() {
    let mut analysis = passing(16.0);
    analysis.never_presented = vec![[900, 700]];
    let (text, verdict) = render(&analysis, &capture(true, 60));
    assert_eq!(verdict, Verdict::Fail, "one miss in two is above p95");
    assert!(text.contains("FAIL  p95 size event to first matching accepted frame: never"));

    analysis.latencies = vec![analysis.latencies[0]; 20];
    assert_eq!(
        render(&analysis, &capture(true, 60)).1,
        Verdict::Pass,
        "one miss in 21 is within p95"
    );
}
