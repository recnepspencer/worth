//! The bound client's pixels as the X server holds them. `GetImage` on the
//! window and on the root at the window's bounds read the same framebuffer
//! through two request paths; under Xvfb there is no compositor and so no
//! second image source. The consistency check between them therefore
//! proves request agreement (right window, right bounds, right layout), not
//! source independence, and the profile declares it that way.
use x11rb::protocol::xproto::{ConnectionExt as _, ImageFormat, Window};

use crate::external_observation::{NativeClientAreaBounds, NativeClientPixelCapture};
use crate::native_platform::NativePlatformFailure;

use super::connection::{capture_failure, Drawable, X11Observation};

/// The whole window: under bare Xvfb the client area is the window.
pub(super) fn capture_window(
    x11: &X11Observation,
    window: Window,
    bounds: NativeClientAreaBounds,
    process_id: u32,
) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
    capture(x11, window, 0, 0, bounds, process_id)
}

/// The same bounds read through the root: what the screen shows there.
pub(super) fn capture_root_at(
    x11: &X11Observation,
    bounds: NativeClientAreaBounds,
    process_id: u32,
) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
    let left = i16::try_from(bounds.left())
        .map_err(|_| NativePlatformFailure::InvalidCaptureWindowBounds)?;
    let top = i16::try_from(bounds.top())
        .map_err(|_| NativePlatformFailure::InvalidCaptureWindowBounds)?;
    capture(x11, x11.root(), left, top, bounds, process_id)
}

/// Raw RGBA of the window for settlement sampling; no observation identity.
pub(super) fn window_rgba(
    x11: &X11Observation,
    window: Window,
    bounds: NativeClientAreaBounds,
) -> Result<Vec<u8>, NativePlatformFailure> {
    let (width, height) = extent(bounds)?;
    let image = get_image(x11, window, 0, 0, width, height)?;
    x11.pixel_layout()
        .to_rgba(&image.data, bounds.width(), bounds.height())
}

fn capture(
    x11: &X11Observation,
    drawable: Drawable,
    x: i16,
    y: i16,
    bounds: NativeClientAreaBounds,
    process_id: u32,
) -> Result<NativeClientPixelCapture, NativePlatformFailure> {
    let (width, height) = extent(bounds)?;
    let image = get_image(x11, drawable, x, y, width, height)?;
    if image.depth != x11.screen().root_depth {
        return Err(NativePlatformFailure::ClientCapture(format!(
            "GetImage answered depth {} for a depth {} root",
            image.depth,
            x11.screen().root_depth
        )));
    }
    let rgba = x11
        .pixel_layout()
        .to_rgba(&image.data, bounds.width(), bounds.height())?;
    NativeClientPixelCapture::new(process_id, bounds.width(), bounds.height(), rgba)
        .ok_or(NativePlatformFailure::InvalidCaptureWindowBounds)
}

fn get_image(
    x11: &X11Observation,
    drawable: Drawable,
    x: i16,
    y: i16,
    width: u16,
    height: u16,
) -> Result<x11rb::protocol::xproto::GetImageReply, NativePlatformFailure> {
    x11.connection()
        .get_image(ImageFormat::Z_PIXMAP, drawable, x, y, width, height, !0)
        .map_err(capture_failure)?
        .reply()
        .map_err(capture_failure)
}

fn extent(bounds: NativeClientAreaBounds) -> Result<(u16, u16), NativePlatformFailure> {
    Ok((
        u16::try_from(bounds.width())
            .map_err(|_| NativePlatformFailure::InvalidCaptureWindowBounds)?,
        u16::try_from(bounds.height())
            .map_err(|_| NativePlatformFailure::InvalidCaptureWindowBounds)?,
    ))
}

/// The root is the one capture surface X offers; a client area that extends
/// past it cannot be read back whole.
pub(super) fn require_within_screen(
    x11: &X11Observation,
    bounds: NativeClientAreaBounds,
) -> Result<(), NativePlatformFailure> {
    let screen = x11.screen();
    let within = bounds.left() >= 0
        && bounds.top() >= 0
        && bounds.right() <= i32::from(screen.width_in_pixels)
        && bounds.bottom() <= i32::from(screen.height_in_pixels);
    if within {
        Ok(())
    } else {
        Err(NativePlatformFailure::ClientOutsideCaptureMonitor)
    }
}
