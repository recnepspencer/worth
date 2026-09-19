use xcap::Window;

use crate::external_observation::{NativeClientAreaBounds, NativeClientPixelCapture};

use super::super::NativePlatformFailure;
use super::capture_region::{crop_monitor_client, NativeMonitorCaptureRegion};

pub(super) fn exact_window(
    process_id: u32,
    window_id: u32,
) -> Result<Window, NativePlatformFailure> {
    let mut matches = Window::all()
        .map_err(capture_failure)?
        .into_iter()
        .filter(|window| {
            window.pid().ok() == Some(process_id) && window.id().ok() == Some(window_id)
        });
    let window = matches.next().ok_or_else(|| {
        NativePlatformFailure::ClientCapture("bound HWND is not capturable".into())
    })?;
    if matches.next().is_some() {
        return Err(NativePlatformFailure::ClientCapture(
            "bound HWND resolves to multiple capture windows".into(),
        ));
    }
    Ok(window)
}

pub(super) fn capture_client_area(
    window: &Window,
    client: NativeClientAreaBounds,
    process_id: u32,
) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
    let image = window
        .capture_image()
        .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))?;
    let region = if image.width() == client.width() && image.height() == client.height() {
        NativeMonitorCaptureRegion::new(0, 0, image.width(), image.height())
    } else {
        let left = window
            .x()
            .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))?;
        let top = window
            .y()
            .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))?;
        NativeMonitorCaptureRegion::new(
            u32::try_from(client.left() - left)
                .map_err(|_| NativePlatformFailure::InvalidCaptureWindowBounds)?,
            u32::try_from(client.top() - top)
                .map_err(|_| NativePlatformFailure::InvalidCaptureWindowBounds)?,
            client.width(),
            client.height(),
        )
    };
    crop_monitor_client(image, region)?.into_observation(process_id)
}

fn capture_failure(error: xcap::XCapError) -> NativePlatformFailure {
    NativePlatformFailure::ClientCapture(error.to_string())
}
