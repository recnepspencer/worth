//! Two `GetImage` requests, one framebuffer. Under bare Xvfb there is no
//! compositor, so the window read and the root read at the window's bounds
//! are two request paths into the same pixels. Their agreement proves the
//! request is right (the bound window, its current bounds, the qualified
//! layout); it is not the independent-source proof the Windows lane has, and
//! the profile's `client_area_observation` says so.
use crate::external_observation::NativeClientPixelCapture;
use crate::native_platform::NativePlatformFailure;

pub(super) fn require_request_agreement(
    window: &NativeClientPixelCapture,
    root: &NativeClientPixelCapture,
) -> Result<(), NativePlatformFailure> {
    if window.process_id() != root.process_id()
        || window.width() != root.width()
        || window.height() != root.height()
    {
        return Err(NativePlatformFailure::ClientCapture(format!(
            "window/root GetImage identity disagreement: window={:?}; root={:?}",
            signature(window),
            signature(root),
        )));
    }
    for (x, y) in control_points(window) {
        let from_window = pixel(window, x, y);
        let from_root = pixel(root, x, y);
        if from_window[3] != 255 || from_window != from_root {
            return Err(NativePlatformFailure::ClientCapture(format!(
                "window/root GetImage disagreement at ({x}, {y}): window={from_window:?}; root={from_root:?}; one framebuffer answered two requests differently, so the bound window or its bounds are stale"
            )));
        }
    }
    Ok(())
}

fn control_points(capture: &NativeClientPixelCapture) -> [(u32, u32); 3] {
    [
        (capture.width() / 4, capture.height() / 4),
        (capture.width() / 2, capture.height() / 2),
        (capture.width() * 3 / 4, capture.height() * 3 / 4),
    ]
}

fn pixel(capture: &NativeClientPixelCapture, x: u32, y: u32) -> [u8; 4] {
    let offset = ((y * capture.width() + x) * 4) as usize;
    capture.rgba()[offset..offset + 4]
        .try_into()
        .expect("control point is one RGBA texel")
}

fn signature(capture: &NativeClientPixelCapture) -> (u32, [u32; 2]) {
    (capture.process_id(), [capture.width(), capture.height()])
}

#[cfg(test)]
mod tests {
    use super::require_request_agreement;
    use crate::external_observation::NativeClientPixelCapture;

    fn solid(process_id: u32, rgba: [u8; 4]) -> NativeClientPixelCapture {
        NativeClientPixelCapture::new(process_id, 4, 4, rgba.repeat(16)).unwrap()
    }

    #[test]
    fn a_root_region_showing_other_pixels_is_a_typed_capture_failure() {
        let window = solid(7, [47, 129, 247, 255]);
        assert!(require_request_agreement(&window, &solid(7, [47, 129, 247, 255])).is_ok());
        assert!(matches!(
            require_request_agreement(&window, &solid(7, [0, 0, 0, 255])),
            Err(crate::native_platform::NativePlatformFailure::ClientCapture(_))
        ));
        assert!(matches!(
            require_request_agreement(&window, &solid(8, [47, 129, 247, 255])),
            Err(crate::native_platform::NativePlatformFailure::ClientCapture(_))
        ));
    }
}
