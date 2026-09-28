use super::*;

#[test]
fn capture_lines_read_back_as_written() {
    let samples = [
        Sample {
            before: 10,
            after: 12,
            client: [1536, 1024],
            pressed: true,
            cursor: [1540, 900],
            reading: Reading::Frame(513),
        },
        Sample {
            before: 13,
            after: 15,
            client: [1500, 1000],
            pressed: false,
            cursor: [-3, 0],
            reading: Reading::Stretched([9, 8]),
        },
        Sample {
            before: 16,
            after: 18,
            client: [1500, 1000],
            pressed: false,
            cursor: [0, 0],
            reading: Reading::Unreadable,
        },
    ];
    let mut text = capture_header(10_000_000, 60, 96, "26200.1", Grip::BottomRight);
    for sample in &samples {
        text.push('\n');
        text.push_str(&capture_line(sample));
    }
    text.push_str("\nend 3\n");
    let log = parse_capture(&text).expect("the capture parses");
    assert_eq!(log.samples, samples);
    assert_eq!(
        (
            log.frequency,
            log.refresh_hz,
            log.dpi,
            log.windows_build.as_str(),
            log.grip
        ),
        (10_000_000, 60, 96, "26200.1", Grip::BottomRight)
    );
    assert!(log.complete);
}

#[test]
fn a_capture_without_its_closing_line_is_incomplete() {
    let text = format!(
        "{}\n1 2 800 600 1 5 5 unreadable\n",
        capture_header(1, 60, 96, "26200.1", Grip::Unknown)
    );
    let log = parse_capture(&text).expect("the capture parses");
    assert!(!log.complete);
    assert_eq!(
        log.grip,
        Grip::Unknown,
        "a header naming no grip knows none"
    );
}

#[test]
fn a_grip_this_tool_does_not_know_is_refused() {
    let header = capture_header(1, 60, 96, "26200.1", Grip::Unknown);
    assert!(parse_capture(&format!("{header} grip left\nend 0\n")).is_err());
}

#[test]
fn host_events_parse_by_kind() {
    let text = "worth-ui-resize-trace 1 frequency 10000000\n\
                5 observed 800 600\n6 consumed 800 600\n\
                7 submitted 42 800 600\n8 accepted 42\n\
                9 target 800 600\n10 text 42 1 2 3 4 5\n\
                11 adapter NVIDIA GeForce (driver 1)\n12 peak textures 3\n\
                13 minimum 1200 900\n";
    let trace = parse_host(text).expect("the trace parses");
    assert_eq!(trace.frequency, 10_000_000);
    let kinds: Vec<HostKind> = trace.events.iter().map(|event| event.kind).collect();
    assert_eq!(
        kinds,
        [
            HostKind::Observed([800, 600]),
            HostKind::Consumed([800, 600]),
            HostKind::Submitted {
                frame: 42,
                extent: [800, 600]
            },
            HostKind::Accepted(42),
            HostKind::Target([800, 600]),
            HostKind::Text {
                frame: 42,
                work: [1, 2, 3, 4, 5]
            },
        ]
    );
    assert_eq!(trace.peaks, [("textures".to_owned(), 3)]);
    assert_eq!(trace.adapter.as_deref(), Some("NVIDIA GeForce (driver 1)"));
    assert_eq!(trace.minimum, Some([1200, 900]));
    let bare = parse_host("worth-ui-resize-trace 1 frequency 1\n5 adapter NVIDIA GeForce\n")
        .expect("an adapter with no driver parses");
    assert_eq!(bare.adapter.as_deref(), Some("NVIDIA GeForce"));
    assert_eq!(bare.minimum, None);
    assert!(parse_host("5 observed 1 1").is_err());
    assert!(parse_host("worth-ui-resize-trace 1 frequency 1\n5 moved 1 1").is_err());
}

#[test]
fn a_minimum_a_scale_change_moved_holds_for_none_of_the_trace() {
    let held = "worth-ui-resize-trace 1 frequency 1\n5 minimum 800 600\n9 minimum 800 600\n";
    assert_eq!(
        parse_host(held).expect("the trace parses").minimum,
        Some([800, 600])
    );
    let moved = "worth-ui-resize-trace 1 frequency 1\n5 minimum 800 600\n9 minimum 1000 750\n";
    assert_eq!(parse_host(moved).expect("the trace parses").minimum, None);
}
