use super::*;
use crate::external_observation::{NativeClientAreaBounds, NativeClientPixelCapture};

#[test]
fn graphics_capture_crop_keeps_client_offsets_and_refuses_partial_or_unbounded_strips() {
    let outer = NativeClientAreaBounds::new(-1920, 40, -920, 840).unwrap();
    let client = NativeClientAreaBounds::new(-1912, 72, -928, 832).unwrap();
    let crop = CaptureCrop::from_bounds(outer, client, [11, 17, 200, 100]).unwrap();
    assert_eq!(crop.frame_extent, [1000, 800]);
    assert_eq!(crop.edges, [19, 49, 219, 149]);
    assert!(CaptureCrop::from_bounds(outer, client, [0, 0, 0, 100]).is_none());
    assert!(CaptureCrop::from_bounds(outer, client, [900, 0, 100, 100]).is_none());
    assert!(CaptureCrop::from_bounds(outer, client, [0, 0, 984, 760]).is_none());
    assert!(CaptureCrop::from_bounds(outer, client, [u32::MAX, 0, 1, 1]).is_none());
}

#[test]
fn graphics_capture_timestamp_retains_qpc_epoch_and_rejects_duplicates_or_future_frames() {
    assert_eq!(
        clock::convert_counter(7_500_000, 3_000_000),
        Some(25_000_000)
    );
    assert_eq!(clock::convert_counter(i64::MAX, 10_000_000), Some(i64::MAX));
    assert_eq!(clock::convert_counter(1, 0), None);
    assert!(clock::validate_timestamp(Some(20), 21, 30));
    assert!(!clock::validate_timestamp(Some(20), 20, 30));
    assert!(!clock::validate_timestamp(Some(20), 19, 30));
    assert!(!clock::validate_timestamp(None, 31, 30));
}

#[test]
fn graphics_capture_removes_only_stride_padding_without_changing_rgba() {
    let pixels =
        worker::copy_rgba_rows(&[1, 2, 3, 4, 99, 99, 5, 6, 7, 8, 99, 99], 6, 1, 2).unwrap();
    assert_eq!(pixels, [1, 2, 3, 4, 5, 6, 7, 8]);
    assert!(worker::copy_rgba_rows(&[1, 2, 3], 4, 1, 1).is_err());
    assert!(worker::copy_rgba_rows(&[1, 2, 3, 4], 3, 1, 1).is_err());
}

#[test]
fn graphics_capture_backpressure_is_a_denial_not_a_silent_dropped_frame() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let frame = || NativeTimedClientPixelCapture {
        captured_qpc_100ns: 1,
        acquired_qpc_100ns: 2,
        copied_qpc_100ns: 3,
        pixels: NativeClientPixelCapture::new(42, 1, 1, vec![1, 2, 3, 255]).unwrap(),
    };
    worker::publish(&sender, frame()).unwrap();
    assert!(worker::publish(&sender, frame())
        .unwrap_err()
        .to_string()
        .contains("backlog"));
    assert_eq!(receiver.recv().unwrap().pixels.rgba(), [1, 2, 3, 255]);
    drop(receiver);
    assert!(worker::publish(&sender, frame()).is_err());
}
