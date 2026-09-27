use super::*;
use crate::analysis::Latency;
use crate::coverage::Coverage;
use crate::gaps::Gap;

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
