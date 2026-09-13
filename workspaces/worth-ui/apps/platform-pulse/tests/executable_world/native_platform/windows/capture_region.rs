use winsafe::HWND;
use xcap::Monitor;

use crate::external_observation::{NativeClientAreaBounds, NativeClientPixelCapture};

use super::NativePlatformFailure;

pub(super) struct XcapMonitorCaptureSource;

trait NativeMonitorCaptureSource {
    fn capture(&self, monitor: &Monitor) -> Result<xcap::image::RgbaImage, NativePlatformFailure>;
}

impl NativeMonitorCaptureSource for XcapMonitorCaptureSource {
    fn capture(&self, monitor: &Monitor) -> Result<xcap::image::RgbaImage, NativePlatformFailure> {
        monitor
            .capture_image()
            .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativeMonitorCaptureRegion {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl NativeMonitorCaptureRegion {
    pub(super) const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

pub(super) struct NativeCapturedClientPixels {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl NativeCapturedClientPixels {
    pub(super) fn into_observation(
        self,
        process_id: u32,
    ) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
        let bounds = NativeClientAreaBounds::new(0, 0, self.width as i32, self.height as i32)
            .expect("captured client pixels have nonzero extent");
        NativeClientPixelCapture::new(process_id, self.width, self.height, self.rgba).ok_or(
            NativePlatformFailure::InvalidClientCapture {
                image_width: self.width,
                image_height: self.height,
                outer: bounds,
                client: bounds,
            },
        )
    }
}

pub(super) fn capture_bound_client_area(
    monitor: &Monitor,
    region: NativeMonitorCaptureRegion,
    process_id: u32,
) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
    capture_bound_client_area_from(&XcapMonitorCaptureSource, monitor, region, process_id)
}

fn capture_bound_client_area_from(
    source: &dyn NativeMonitorCaptureSource,
    monitor: &Monitor,
    region: NativeMonitorCaptureRegion,
    process_id: u32,
) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
    let image = source.capture(monitor)?;
    crop_monitor_client(image, region)?.into_observation(process_id)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeMonitorPhysicalBounds {
    left: i32,
    top: i32,
    width: u32,
    height: u32,
}

pub(super) fn crop_monitor_client(
    screenshot: xcap::image::RgbaImage,
    region: NativeMonitorCaptureRegion,
) -> Result<NativeCapturedClientPixels, NativePlatformFailure> {
    let right = region
        .x
        .checked_add(region.width)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
    let bottom = region
        .y
        .checked_add(region.height)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
    if right > screenshot.width() || bottom > screenshot.height() {
        return Err(invalid_crop(
            screenshot.width(),
            screenshot.height(),
            region,
        ));
    }
    let source_width = screenshot.width() as usize;
    let row_bytes = region.width as usize * 4;
    let raw = screenshot.into_raw();
    let mut cropped = Vec::with_capacity(row_bytes * region.height as usize);
    for y in region.y as usize..bottom as usize {
        let start = (y * source_width + region.x as usize) * 4;
        cropped.extend_from_slice(&raw[start..start + row_bytes]);
    }
    Ok(NativeCapturedClientPixels {
        width: region.width,
        height: region.height,
        rgba: cropped,
    })
}

fn invalid_crop(
    width: u32,
    height: u32,
    region: NativeMonitorCaptureRegion,
) -> NativePlatformFailure {
    NativePlatformFailure::InvalidClientCapture {
        image_width: width,
        image_height: height,
        outer: NativeClientAreaBounds::new(0, 0, width as i32, height as i32)
            .expect("nonempty monitor capture"),
        client: NativeClientAreaBounds::new(
            region.x as i32,
            region.y as i32,
            (region.x + region.width) as i32,
            (region.y + region.height) as i32,
        )
        .expect("nonempty client capture region"),
    }
}

pub(super) fn client_bounds(
    window: &HWND,
) -> Result<NativeClientAreaBounds, NativePlatformFailure> {
    let client = window
        .GetClientRect()
        .and_then(|rect| window.ClientToScreenRc(rect))
        .map_err(|error| NativePlatformFailure::WindowEnumeration(error.to_string()))?;
    NativeClientAreaBounds::new(client.left, client.top, client.right, client.bottom)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)
}

pub(super) fn monitor_for_client(
    client: NativeClientAreaBounds,
) -> Result<Monitor, NativePlatformFailure> {
    let center_x = client
        .left()
        .checked_add_unsigned(client.width() / 2)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
    let center_y = client
        .top()
        .checked_add_unsigned(client.height() / 2)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)?;
    Monitor::from_point(center_x, center_y)
        .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))
}

pub(super) fn monitor_capture_region(
    monitor: &Monitor,
    client: NativeClientAreaBounds,
) -> Result<NativeMonitorCaptureRegion, NativePlatformFailure> {
    let bounds = NativeMonitorPhysicalBounds {
        left: monitor
            .x()
            .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))?,
        top: monitor
            .y()
            .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))?,
        width: monitor
            .width()
            .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))?,
        height: monitor
            .height()
            .map_err(|error| NativePlatformFailure::ClientCapture(error.to_string()))?,
    };
    monitor_capture_region_from_bounds(bounds, client)
}

fn monitor_capture_region_from_bounds(
    monitor: NativeMonitorPhysicalBounds,
    client: NativeClientAreaBounds,
) -> Result<NativeMonitorCaptureRegion, NativePlatformFailure> {
    let right = monitor
        .left
        .checked_add_unsigned(monitor.width)
        .ok_or(NativePlatformFailure::ClientOutsideCaptureMonitor)?;
    let bottom = monitor
        .top
        .checked_add_unsigned(monitor.height)
        .ok_or(NativePlatformFailure::ClientOutsideCaptureMonitor)?;
    if client.left() < monitor.left
        || client.top() < monitor.top
        || client.right() > right
        || client.bottom() > bottom
    {
        return Err(NativePlatformFailure::ClientOutsideCaptureMonitor);
    }
    Ok(NativeMonitorCaptureRegion {
        x: u32::try_from(client.left() - monitor.left)
            .map_err(|_| NativePlatformFailure::ClientOutsideCaptureMonitor)?,
        y: u32::try_from(client.top() - monitor.top)
            .map_err(|_| NativePlatformFailure::ClientOutsideCaptureMonitor)?,
        width: client.width(),
        height: client.height(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_dpi_client_extent_is_captured_without_resampling() {
        let client = NativeClientAreaBounds::new(87, 121, 327, 265).unwrap();
        assert_eq!(
            monitor_capture_region_from_bounds(
                NativeMonitorPhysicalBounds {
                    left: 0,
                    top: 0,
                    width: 3_840,
                    height: 2_160,
                },
                client,
            )
            .unwrap(),
            NativeMonitorCaptureRegion {
                x: 87,
                y: 121,
                width: 240,
                height: 144,
            }
        );
    }

    #[test]
    fn negative_monitor_origin_projects_to_monitor_local_physical_pixels() {
        let client = NativeClientAreaBounds::new(-1_700, 140, -1_460, 284).unwrap();
        assert_eq!(
            monitor_capture_region_from_bounds(
                NativeMonitorPhysicalBounds {
                    left: -1_920,
                    top: 0,
                    width: 1_920,
                    height: 1_080,
                },
                client,
            )
            .unwrap(),
            NativeMonitorCaptureRegion {
                x: 220,
                y: 140,
                width: 240,
                height: 144,
            }
        );
    }

    #[test]
    fn client_crossing_a_monitor_edge_is_rejected_without_partial_capture() {
        let client = NativeClientAreaBounds::new(1_800, 140, 2_040, 284).unwrap();
        assert!(matches!(
            monitor_capture_region_from_bounds(
                NativeMonitorPhysicalBounds {
                    left: 0,
                    top: 0,
                    width: 1_920,
                    height: 1_080,
                },
                client,
            ),
            Err(NativePlatformFailure::ClientOutsideCaptureMonitor)
        ));
    }

    #[test]
    fn os_capture_crop_preserves_spatially_distinct_source_pixels() {
        let source = xcap::image::RgbaImage::from_raw(
            3,
            2,
            vec![
                1, 2, 3, 255, 10, 11, 12, 255, 20, 21, 22, 255, 30, 31, 32, 255, 40, 41, 42, 255,
                50, 51, 52, 255,
            ],
        )
        .unwrap();
        let captured = crop_monitor_client(
            source,
            NativeMonitorCaptureRegion {
                x: 1,
                y: 0,
                width: 2,
                height: 2,
            },
        )
        .unwrap();
        assert_eq!(captured.width, 2);
        assert_eq!(captured.height, 2);
        assert_eq!(
            captured.rgba,
            vec![10, 11, 12, 255, 20, 21, 22, 255, 40, 41, 42, 255, 50, 51, 52, 255]
        );
    }
}
