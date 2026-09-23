use uiautomation::screenshots::Screenshot;
use uiautomation::types::Rect;

use crate::external_observation::{NativeClientAreaBounds, NativeClientPixelCapture};

use super::super::NativePlatformFailure;
use super::{NativePlatformContract, WindowsCaptureExposure, WindowsNativePlatform};

impl WindowsNativePlatform {
    /// One synchronous observation under an already qualified exposure. This
    /// never re-exposes, flushes, waits for settlement, or changes focus.
    pub(crate) fn observe_exposed_gdi_region(
        &self,
        exposure: &WindowsCaptureExposure<'_>,
        region: [u32; 4],
    ) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
        let bound = exposure.bound;
        let observed = self.observe_bound_client_area(bound)?;
        if winsafe::HWND::GetForegroundWindow().as_ref() != Some(&bound.window) {
            return Err(NativePlatformFailure::ClientCapture(
                "GDI observation lost foreground".into(),
            ));
        }
        let region = client_region(observed.bounds(), region)
            .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
        let capture = capture_client_area(region, observed.process_id())?;
        self.observe_bound_client_area(bound)?;
        if winsafe::HWND::GetForegroundWindow().as_ref() != Some(&bound.window) {
            return Err(NativePlatformFailure::ClientCapture(
                "GDI observation foreground changed".into(),
            ));
        }
        Ok(capture)
    }
}

fn client_region(
    client: NativeClientAreaBounds,
    [x, y, width, height]: [u32; 4],
) -> Option<NativeClientAreaBounds> {
    let right = x.checked_add(width)?;
    let bottom = y.checked_add(height)?;
    if right > client.width() || bottom > client.height() {
        return None;
    }
    NativeClientAreaBounds::new(
        client.left().checked_add(i32::try_from(x).ok()?)?,
        client.top().checked_add(i32::try_from(y).ok()?)?,
        client.left().checked_add(i32::try_from(right).ok()?)?,
        client.top().checked_add(i32::try_from(bottom).ok()?)?,
    )
}

#[test]
fn gdi_region_is_client_relative_and_cannot_escape_its_bound_window() {
    let client = NativeClientAreaBounds::new(-100, 20, 300, 220).unwrap();
    assert_eq!(
        client_region(client, [10, 30, 100, 50]),
        NativeClientAreaBounds::new(-90, 50, 10, 100)
    );
    for region in [
        [0, 0, 0, 1],
        [390, 0, 20, 1],
        [0, 200, 1, 1],
        [u32::MAX, 0, 2, 1],
    ] {
        assert!(client_region(client, region).is_none());
    }
}

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
