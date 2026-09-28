use crate::analysis::analyze;
use crate::analysis::tests::{capture, host, submitted};
use crate::gaps::Gap;
use crate::logs::{Grip, HostKind};
use crate::stamp::Reading;

#[test]
fn an_idle_gap_has_no_active_part() {
    let trace = host(&[
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (31, HostKind::Accepted(1)),
        (100, HostKind::Observed([900, 700])),
        (200, submitted(2, [900, 700])),
        (201, HostKind::Accepted(2)),
        (250, HostKind::Observed([900, 700])),
    ]);
    let log = capture(&trace, |before| match before {
        ..35 => Reading::Unreadable,
        35..200 => Reading::Frame(1),
        _ => Reading::Frame(2),
    });
    let analysis = analyze(&trace, &log).expect("the logs correlate");
    assert_eq!(
        analysis.gaps,
        [
            Gap {
                start: 27.0,
                end: 192.0,
                active: None
            },
            Gap {
                start: 192.0,
                end: 392.0,
                active: None
            },
        ],
        "cursor movement the window answered with no new extent is not demand"
    );
}

#[test]
fn an_extent_still_unshown_when_a_gap_starts_makes_it_active() {
    let trace = host(&[
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (31, HostKind::Accepted(1)),
        (33, HostKind::Observed([800, 600])),
        (34, HostKind::Consumed([800, 600])),
        (200, submitted(2, [800, 600])),
        (201, HostKind::Accepted(2)),
    ]);
    let mut log = capture(&trace, |before| match before {
        ..35 => Reading::Unreadable,
        35..200 => Reading::Frame(1),
        _ => Reading::Frame(2),
    });
    for sample in &mut log.samples {
        sample.cursor = [0, 0];
    }
    let analysis = analyze(&trace, &log).expect("the logs correlate");
    assert_eq!(
        analysis.gaps[0],
        Gap {
            start: 27.0,
            end: 192.0,
            active: Some(165.0)
        }
    );
}

#[test]
fn a_stalled_host_owes_frames_from_the_first_cursor_movement() {
    let trace = host(&[
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (31, HostKind::Accepted(1)),
        (250, HostKind::Observed([800, 600])),
        (251, HostKind::Consumed([800, 600])),
        (255, submitted(2, [800, 600])),
        (256, HostKind::Accepted(2)),
    ]);
    let log = capture(&trace, |before| match before {
        ..35 => Reading::Unreadable,
        35..255 => Reading::Frame(1),
        _ => Reading::Frame(2),
    });
    let analysis = analyze(&trace, &log).expect("the logs correlate");
    assert_eq!(
        analysis.gaps[0],
        Gap {
            start: 27.0,
            end: 247.0,
            active: Some(217.0)
        },
        "not only the 7 ms after the host reported the extent"
    );
}

#[test]
fn the_gap_open_at_release_ends_at_the_next_frame_or_the_capture() {
    let events = [
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (31, HostKind::Accepted(1)),
        (100, HostKind::Observed([800, 600])),
        (101, HostKind::Consumed([800, 600])),
        (350, submitted(2, [800, 600])),
        (351, HostKind::Accepted(2)),
    ];
    let reading = |before| match before {
        ..35 => Reading::Unreadable,
        35..350 => Reading::Frame(1),
        _ => Reading::Frame(2),
    };
    let trace = host(&events);
    let analysis = analyze(&trace, &capture(&trace, reading)).expect("the logs correlate");
    assert_eq!(
        analysis.gaps,
        [Gap {
            start: 27.0,
            end: 342.0,
            active: Some(312.0)
        }]
    );

    let trace = host(&events[..6]);
    let analysis = analyze(&trace, &capture(&trace, reading)).expect("the logs correlate");
    assert_eq!(
        analysis.gaps,
        [Gap {
            start: 27.0,
            end: 392.0,
            active: Some(362.0)
        }]
    );
    assert_eq!(analysis.missing, Some([800, 600]));
}

#[test]
fn movement_held_past_the_minimum_extent_owes_nothing_until_it_comes_back() {
    let mut trace = host(&[
        (20, HostKind::Observed([900, 700])),
        (22, HostKind::Consumed([900, 700])),
        (30, submitted(1, [900, 700])),
        (31, HostKind::Accepted(1)),
        (32, HostKind::Observed([880, 700])),
        (33, HostKind::Consumed([880, 700])),
        (40, submitted(2, [880, 700])),
        (41, HostKind::Accepted(2)),
        (202, HostKind::Observed([890, 700])),
        (203, HostKind::Consumed([890, 700])),
        (250, submitted(3, [890, 700])),
        (251, HostKind::Accepted(3)),
    ]);
    let mut log = capture(&trace, |before| match before {
        ..35 => Reading::Unreadable,
        35..45 => Reading::Frame(1),
        45..250 => Reading::Frame(2),
        _ => Reading::Frame(3),
    });
    for sample in log.samples.iter_mut().filter(|sample| sample.pressed) {
        let inward = if sample.before < 200 {
            sample.before - 10
        } else {
            10
        };
        sample.cursor = [-(inward as i32), 0];
    }
    let pinned = |analysis: crate::analysis::Analysis| analysis.gaps[1];

    trace.minimum = Some([880, 700]);
    log.grip = Grip::BottomRight;
    assert_eq!(
        pinned(analyze(&trace, &log).expect("the logs correlate")),
        Gap {
            start: 37.0,
            end: 242.0,
            active: Some(52.0)
        },
        "the corner held at 880 wide moved again only when the cursor came back"
    );

    trace.minimum = None;
    assert_eq!(
        pinned(analyze(&trace, &log).expect("the logs correlate")).active,
        Some(202.0),
        "without a traced minimum, every held movement is demand"
    );

    trace.minimum = Some([880, 700]);
    log.grip = Grip::Unknown;
    assert_eq!(
        pinned(analyze(&trace, &log).expect("the logs correlate")).active,
        Some(202.0),
        "a drag of an unknown edge may be moving outward, so every held movement is demand"
    );
}
