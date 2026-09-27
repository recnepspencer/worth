use super::*;

/// One count per millisecond keeps the expected values readable.
const FREQUENCY: i64 = 1000;

pub(crate) fn host(events: &[(i64, HostKind)]) -> HostTrace {
    HostTrace {
        frequency: FREQUENCY,
        events: events
            .iter()
            .map(|&(counter, kind)| HostEvent { counter, kind })
            .collect(),
        peaks: Vec::new(),
        adapter: None,
    }
}

pub(crate) fn submitted(frame: u64, extent: [u32; 2]) -> HostKind {
    HostKind::Submitted { frame, extent }
}

/// Samples every 5 ms from 0 to 400, each 2 ms long, held from 10 until 300
/// with the cursor moving on every held sample. The client follows the host's
/// latest observed extent, starting at 900x700.
pub(crate) fn capture(trace: &HostTrace, reading: impl Fn(i64) -> Reading) -> CaptureLog {
    let samples = (0..=80)
        .map(|index| {
            let before = index * 5;
            let client = trace
                .events
                .iter()
                .rev()
                .find_map(|event| match event.kind {
                    HostKind::Observed(extent) if event.counter <= before => Some(extent),
                    _ => None,
                })
                .unwrap_or([900, 700]);
            let pressed = (10..300).contains(&before);
            Sample {
                before,
                after: before + 2,
                client,
                pressed,
                cursor: if pressed { [before as i32, 0] } else { [0, 0] },
                reading: reading(before),
            }
        })
        .collect();
    CaptureLog {
        frequency: FREQUENCY,
        refresh_hz: 60,
        dpi: 96,
        windows_build: "26200.1".to_owned(),
        samples,
        complete: true,
    }
}

fn drag_trace() -> HostTrace {
    host(&[
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (40, HostKind::Accepted(1)),
        (50, HostKind::Observed([850, 650])),
        (51, HostKind::Observed([800, 600])),
        (55, HostKind::Consumed([800, 600])),
        (60, submitted(2, [800, 600])),
        (70, HostKind::Accepted(2)),
        (100, HostKind::Observed([820, 620])),
        (101, HostKind::Consumed([820, 620])),
        (105, submitted(3, [820, 620])),
        (110, HostKind::Observed([900, 700])),
        (111, HostKind::Consumed([900, 700])),
        (118, HostKind::Accepted(3)),
        (120, submitted(4, [900, 700])),
        (130, HostKind::Accepted(4)),
    ])
}

fn drag_reading(before: i64) -> Reading {
    match before {
        ..35 => Reading::Unreadable,
        35..65 => Reading::Frame(1),
        65..100 => Reading::Frame(2),
        100..125 => Reading::Stretched([9, 8]),
        _ => Reading::Frame(4),
    }
}

#[test]
fn a_drag_reports_latency_gaps_and_settlement() {
    let trace = drag_trace();
    let analysis = analyze(&trace, &capture(&trace, drag_reading)).expect("the logs correlate");
    assert_eq!(analysis.press, 10);
    assert_eq!((analysis.drag_ms, analysis.moving_ms), (285.0, 285.0));
    assert_eq!(
        analysis.coverage,
        Coverage {
            smallest: [800, 600],
            largest: [900, 700],
            reached_small: true,
            reached_large: false,
            reversals: 1,
            narrowing: 0,
            widening: 0,
        }
    );
    assert_eq!(
        analysis.latencies,
        [
            Latency {
                extent: [900, 700],
                observed: 10.0,
                visible: 27.0
            },
            Latency {
                extent: [800, 600],
                observed: 41.0,
                visible: 57.0
            },
            Latency {
                extent: [900, 700],
                observed: 100.0,
                visible: 117.0
            },
        ]
    );
    assert_eq!(analysis.unseen, 1, "820x620 was accepted but never caught");
    assert!(analysis.never_presented.is_empty());
    assert_eq!((analysis.missing, analysis.consume_starts), (None, 0));
    assert_eq!(
        analysis.gaps,
        [
            Gap {
                start: 27.0,
                end: 57.0,
                active: Some(27.0)
            },
            Gap {
                start: 57.0,
                end: 117.0,
                active: Some(57.0)
            },
            Gap {
                start: 117.0,
                end: 392.0,
                active: None
            },
        ],
        "the cursor moved from the start of each gap the window observed a new extent in"
    );
    assert_eq!(analysis.final_extent, [900, 700]);
    assert_eq!(analysis.final_ms, Some(0.0), "settled before release");
    assert_eq!((analysis.unknown, analysis.unaccepted), (0, 0));
    assert_eq!((analysis.regressions, analysis.stale_submissions), (0, 0));
    assert_eq!(analysis.stretched_ms, 25.0);
    assert_eq!(analysis.unreadable_ms, 20.0);
    assert_eq!(
        analysis.mismatched_ms, 15.0,
        "900x700 still shown after the client shrank"
    );
    assert_eq!(analysis.sample_intervals, vec![5.0; 57]);
}

#[test]
fn consumed_extents_are_seen_unseen_or_never_presented() {
    let trace = host(&[
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (31, HostKind::Accepted(1)),
        (50, HostKind::Observed([850, 650])),
        (51, HostKind::Consumed([850, 650])),
        (60, HostKind::Observed([820, 620])),
        (61, HostKind::Consumed([820, 620])),
        (62, submitted(2, [820, 620])),
        (63, HostKind::Accepted(2)),
        (70, HostKind::Observed([800, 600])),
        (71, HostKind::Consumed([800, 600])),
        (75, submitted(3, [800, 600])),
        (76, HostKind::Accepted(3)),
    ]);
    let log = capture(&trace, |before| match before {
        ..35 => Reading::Unreadable,
        35..80 => Reading::Frame(1),
        _ => Reading::Frame(3),
    });
    let analysis = analyze(&trace, &log).expect("the logs correlate");
    assert_eq!(analysis.never_presented, [[850, 650]]);
    assert_eq!(analysis.unseen, 1);
    let latencies: Vec<([u32; 2], f64, f64)> = analysis
        .latencies
        .iter()
        .map(|latency| (latency.extent, latency.observed, latency.visible))
        .collect();
    assert_eq!(
        latencies,
        [([900, 700], 10.0, 27.0), ([800, 600], 60.0, 72.0)]
    );
}

#[test]
fn a_return_to_an_older_frame_is_counted() {
    let trace = host(&[
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (31, HostKind::Accepted(1)),
        (60, submitted(2, [900, 700])),
        (61, HostKind::Accepted(2)),
    ]);
    let log = capture(&trace, |before| match before {
        ..35 => Reading::Unreadable,
        35..60 | 90..120 => Reading::Frame(1),
        _ => Reading::Frame(2),
    });
    let analysis = analyze(&trace, &log).expect("the logs correlate");
    assert_eq!(analysis.regressions, 1);
}

#[test]
fn frames_the_host_never_named_or_accepted_are_counted() {
    let trace = host(&[
        (20, HostKind::Observed([800, 600])),
        (22, HostKind::Consumed([800, 600])),
        (30, submitted(1, [900, 700])),
        (60, submitted(2, [800, 600])),
    ]);
    let log = capture(&trace, |before| match before {
        ..30 => Reading::Frame(7),
        30..60 => Reading::Frame(1),
        _ => Reading::Frame(2),
    });
    let analysis = analyze(&trace, &log).expect("the logs correlate");
    assert_eq!(analysis.unknown, 6, "frame 7 was never submitted");
    assert_eq!(analysis.unaccepted, 2);
    assert_eq!(
        analysis.mismatched_ms, 30.0,
        "900x700 shown in an 800x600 client"
    );
    assert_eq!(analysis.missing, Some([800, 600]));
    assert_eq!(analysis.final_ms, None);
    assert_eq!(
        analysis.stale_submissions, 1,
        "900x700 drawn after 800x600 was consumed"
    );
}

#[test]
fn logs_on_different_clocks_or_without_a_press_do_not_correlate() {
    let trace = drag_trace();
    let mut log = capture(&trace, drag_reading);
    log.frequency = 10_000_000;
    assert!(analyze(&trace, &log).is_err());
    let mut log = capture(&trace, drag_reading);
    for sample in &mut log.samples {
        sample.pressed = false;
    }
    assert!(analyze(&trace, &log).is_err());
}

#[test]
fn a_frame_seen_after_the_window_moved_on_still_ends_its_latency() {
    let trace = host(&[
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (31, HostKind::Accepted(1)),
        (33, HostKind::Observed([800, 600])),
        (34, HostKind::Consumed([800, 600])),
        (60, submitted(2, [800, 600])),
        (61, HostKind::Accepted(2)),
    ]);
    let log = capture(&trace, |before| match before {
        ..35 => Reading::Unreadable,
        35..60 => Reading::Frame(1),
        _ => Reading::Frame(2),
    });
    let analysis = analyze(&trace, &log).expect("the logs correlate");
    let latencies: Vec<([u32; 2], f64)> = analysis
        .latencies
        .iter()
        .map(|latency| (latency.extent, latency.visible - latency.observed))
        .collect();
    assert_eq!(latencies, [([900, 700], 17.0), ([800, 600], 29.0)]);
    assert_eq!((analysis.unseen, analysis.never_presented.len()), (0, 0));
    assert_eq!(
        analysis.mismatched_ms, 25.0,
        "900x700 shown in an 800x600 client"
    );
}

#[test]
fn only_moving_stretches_count_toward_the_drag() {
    let mut log = capture(&host(&[]), |_| Reading::Unreadable);
    for sample in &mut log.samples {
        if (30..=290).contains(&sample.before) {
            sample.cursor = [25, 0];
        }
    }
    let clock = Clock {
        origin: 0,
        frequency: FREQUENCY,
    };
    assert_eq!(
        moving_ms(&log.samples, [2, 60], &clock),
        15.0,
        "the 270 ms stillness before the last movement is a pause, not movement"
    );
}
