use crate::external_observation::NativeClientPixelCapture;

use super::NativePlatformFailure;

pub(super) fn require_matching_capture_sources(
    monitor: &NativeClientPixelCapture,
    window: &NativeClientPixelCapture,
) -> Result<(), NativePlatformFailure> {
    if monitor.process_id() != window.process_id()
        || monitor.width() != window.width()
        || monitor.height() != window.height()
    {
        return Err(capture_mismatch(monitor, window));
    }
    for (x, y) in capture_control_points(monitor) {
        let observed = capture_pixel(monitor, x, y);
        let source = capture_pixel(window, x, y);
        if source[3] != 255 || observed != source {
            return Err(capture_mismatch(monitor, window));
        }
    }
    Ok(())
}

pub(super) fn require_matching_composited_sources(
    monitor: &NativeClientPixelCapture,
    gdi: &NativeClientPixelCapture,
) -> Result<(), NativePlatformFailure> {
    let same_identity = monitor.process_id() == gdi.process_id()
        && monitor.width() == gdi.width()
        && monitor.height() == gdi.height();
    let same_rgb = capture_control_points(monitor)
        .into_iter()
        .all(|(x, y)| capture_pixel(monitor, x, y)[..3] == capture_pixel(gdi, x, y)[..3]);
    if same_identity && same_rgb {
        Ok(())
    } else {
        Err(capture_mismatch(monitor, gdi))
    }
}

fn capture_mismatch(
    monitor: &NativeClientPixelCapture,
    window: &NativeClientPixelCapture,
) -> NativePlatformFailure {
    NativePlatformFailure::ClientCapture(format!(
        "independent capture mismatch: monitor={:?}; window={:?}",
        capture_signature(monitor),
        capture_signature(window),
    ))
}

fn capture_control_points(capture: &NativeClientPixelCapture) -> [(u32, u32); 3] {
    [
        (capture.width() / 4, capture.height() / 4),
        (capture.width() / 2, capture.height() / 2),
        (capture.width() * 3 / 4, capture.height() * 3 / 4),
    ]
}

fn capture_pixel(capture: &NativeClientPixelCapture, x: u32, y: u32) -> [u8; 4] {
    let index = ((y * capture.width() + x) * 4) as usize;
    capture.rgba()[index..index + 4].try_into().unwrap()
}

fn capture_signature(capture: &NativeClientPixelCapture) -> ([u32; 2], [[u8; 4]; 3]) {
    let points = [
        (0, 0),
        (capture.width() / 2, capture.height() / 2),
        (capture.width() - 1, capture.height() - 1),
    ];
    let pixels = points.map(|(x, y)| {
        let index = ((y * capture.width() + x) * 4) as usize;
        capture.rgba()[index..index + 4].try_into().unwrap()
    });
    ([capture.width(), capture.height()], pixels)
}

pub(super) fn independent_window_capture_rejects_monitor_pixel_substitution() {
    let monitor = NativeClientPixelCapture::new(
        7,
        4,
        1,
        vec![
            1, 2, 3, 4, 47, 129, 247, 255, 47, 129, 247, 255, 47, 129, 247, 255,
        ],
    )
    .unwrap();
    let exact = NativeClientPixelCapture::new(7, 4, 1, monitor.rgba().to_vec()).unwrap();
    assert!(require_matching_capture_sources(&monitor, &exact).is_ok());
    let gdi = NativeClientPixelCapture::new(
        7,
        4,
        1,
        vec![
            9, 9, 9, 0, 47, 129, 247, 0, 47, 129, 247, 0, 47, 129, 247, 0,
        ],
    )
    .unwrap();
    assert!(require_matching_composited_sources(&monitor, &gdi).is_ok());
    let substituted = NativeClientPixelCapture::new(
        7,
        4,
        1,
        vec![
            1, 2, 3, 4, 47, 129, 247, 255, 1, 2, 3, 255, 47, 129, 247, 255,
        ],
    )
    .unwrap();
    assert!(matches!(
        require_matching_capture_sources(&substituted, &exact),
        Err(NativePlatformFailure::ClientCapture(_))
    ));
    assert!(matches!(
        require_matching_composited_sources(&substituted, &gdi),
        Err(NativePlatformFailure::ClientCapture(_))
    ));
}
