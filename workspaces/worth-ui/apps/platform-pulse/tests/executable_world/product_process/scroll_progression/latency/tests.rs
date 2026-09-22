use super::*;
use accepted_samples::{join, AcceptedPose};

fn visible(timestamp: i64, top: u32) -> VisibleScrollFrame {
    VisibleScrollFrame {
        qpc_100ns: timestamp,
        thumb_top_px: top,
        thumb_length_px: 30,
    }
}
fn accepted(timestamp: i64, attempt: u64, top: i64) -> AcceptedPose {
    AcceptedPose {
        frame: 9,
        attempt,
        qpc_100ns: timestamp,
        top,
        length: 30,
    }
}

#[test]
fn scroll_timing_join_never_reuses_a_capture_for_multiple_accepted_identities() {
    let result = join(
        &[accepted(100, 1, 10), accepted(200, 2, 10)],
        &[visible(110, 10)],
    )
    .unwrap();
    assert_eq!(result.observed, [(0, 0)]);
    assert_eq!(result.unobserved, [1]);
    let exact = join(
        &[accepted(100, 1, 10), accepted(200, 2, 20)],
        &[visible(110, 10), visible(210, 20)],
    )
    .unwrap();
    assert_eq!(exact.observed, [(0, 0), (1, 1)]);
    assert!(exact.unobserved.is_empty());
}

#[test]
fn scroll_timing_join_refuses_out_of_order_and_stale_repeated_poses() {
    assert!(join(
        &[accepted(100, 1, 10)],
        &[visible(210, 10), visible(110, 10)]
    )
    .is_err());
    assert!(join(&[accepted(200, 1, 10), accepted(100, 2, 10)], &[]).is_err());
    assert!(join(&[accepted(100, 1, 10), accepted(200, 1, 10)], &[]).is_err());
    let stale = join(&[accepted(2_000_000, 1, 10)], &[visible(500_000, 10)]).unwrap();
    assert!(stale.observed.is_empty());
    assert_eq!(stale.unobserved, [0]);
}

#[test]
fn scroll_timing_settlement_requires_the_expected_target_and_quiet() {
    let offset = 180.0;
    let dpi = 144;
    let geometry = RecentActivityScrollGeometry;
    let frame = VisibleScrollFrame {
        qpc_100ns: 10,
        thumb_top_px: physical_px(geometry.thumb_top_points(offset), dpi) as u32,
        thumb_length_px: physical_px(geometry.thumb_length_points(), dpi) as u32,
    };
    assert!(trace::settled(
        &frame,
        offset,
        dpi,
        Duration::from_millis(150)
    ));
    assert!(!trace::settled(
        &frame,
        offset,
        dpi,
        Duration::from_millis(149)
    ));
    assert!(!trace::settled(
        &frame,
        offset + 60.0,
        dpi,
        Duration::from_secs(1)
    ));
}

#[test]
fn scroll_timing_content_motion_counts_while_thumb_is_pixel_quantized() {
    let frame = |value| ObservedScrollFrame {
        visible: visible(10, 20),
        pixels: NativeClientPixelCapture::new(42, 4, 2, vec![value; 4 * 4 * 2]).unwrap(),
    };
    assert!(!visible_change::changed(&frame(0), &frame(0)).unwrap());
    assert!(visible_change::changed(&frame(30), &frame(0)).unwrap());
    assert!(!visible_change::changed(&frame(4), &frame(0)).unwrap());
}
