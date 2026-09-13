use uiautomation::screenshots::Screenshot;
use uiautomation::types::Rect;

use crate::external_observation::{NativeClientAreaBounds, NativeClientPixelCapture};

use super::super::NativePlatformFailure;

pub(super) fn capture_client_area(
    client: NativeClientAreaBounds,
    process_id: u32,
) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
    let screenshot = Screenshot::capture_rect(Rect::new(
        client.left(),
        client.top(),
        client.right(),
        client.bottom(),
    ))
    .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))?
    .to_rgba();
    NativeClientPixelCapture::new(
        process_id,
        screenshot.width(),
        screenshot.height(),
        screenshot.pixels().to_vec(),
    )
    .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)
}
