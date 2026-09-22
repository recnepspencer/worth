//! Which top-level X windows belong to the product process. Without a window
//! manager (bare Xvfb) the root's children are the clients' top-levels
//! themselves: no reparenting frames to see through.
use x11rb::protocol::xproto::{ConnectionExt as _, MapState, Window};

use crate::external_observation::NativeClientAreaBounds;
use crate::native_platform::NativePlatformFailure;

use super::connection::{enumeration_failure, X11Observation};

pub(super) struct ProcessWindowCandidate {
    pub(super) window: Window,
    pub(super) bounds: NativeClientAreaBounds,
}

/// Viewable, non-override-redirect top-levels whose `_NET_WM_PID` is the
/// product's. A window that vanishes mid-enumeration is skipped, not failed:
/// enumeration is a snapshot and the binding loop re-observes for stability.
pub(super) fn enumerate(
    x11: &X11Observation,
    process_id: u32,
) -> Result<Vec<ProcessWindowCandidate>, NativePlatformFailure> {
    let connection = x11.connection();
    let tree = connection
        .query_tree(x11.root())
        .map_err(enumeration_failure)?
        .reply()
        .map_err(enumeration_failure)?;
    let mut candidates = Vec::new();
    for window in tree.children {
        let owner = x11.window_process_id(window)?;
        let Ok(attributes) = connection
            .get_window_attributes(window)
            .map_err(enumeration_failure)?
            .reply()
        else {
            continue;
        };
        if !is_process_top_level(
            owner,
            process_id,
            attributes.map_state,
            attributes.override_redirect,
        ) {
            continue;
        }
        if let Some(bounds) = client_bounds(x11, window)? {
            candidates.push(ProcessWindowCandidate { window, bounds });
        }
    }
    Ok(candidates)
}

/// The one rule that names a product window. `_NET_WM_PID` is the only
/// ownership fact X exposes and winit sets it on every top-level
/// (`x11/window.rs`); a window without the property has no owner the
/// observer can prove and is never a match, whatever else it looks like.
fn is_process_top_level(
    owner: Option<u32>,
    process_id: u32,
    map_state: MapState,
    override_redirect: bool,
) -> bool {
    owner == Some(process_id) && map_state == MapState::VIEWABLE && !override_redirect
}

/// Whether the server still knows the window at all; the distinction between
/// "gone" and "changed owner" is what `BoundWindowMissing` carries.
pub(super) fn window_exists(
    x11: &X11Observation,
    window: Window,
) -> Result<bool, NativePlatformFailure> {
    match x11
        .connection()
        .get_window_attributes(window)
        .map_err(enumeration_failure)?
        .reply()
    {
        Ok(_) => Ok(true),
        Err(x11rb::errors::ReplyError::X11Error(error))
            if error.error_kind == x11rb::protocol::ErrorKind::Window =>
        {
            Ok(false)
        }
        Err(error) => Err(enumeration_failure(error)),
    }
}

/// The window's client area in root coordinates. Under bare Xvfb the client
/// area is the whole window; the geometry is translated to the root so the
/// bounds compare with pointer positions and root captures directly.
pub(super) fn client_bounds(
    x11: &X11Observation,
    window: Window,
) -> Result<Option<NativeClientAreaBounds>, NativePlatformFailure> {
    let connection = x11.connection();
    let Ok(geometry) = connection
        .get_geometry(window)
        .map_err(enumeration_failure)?
        .reply()
    else {
        return Ok(None);
    };
    let Ok(origin) = connection
        .translate_coordinates(window, x11.root(), 0, 0)
        .map_err(enumeration_failure)?
        .reply()
    else {
        return Ok(None);
    };
    let left = i32::from(origin.dst_x);
    let top = i32::from(origin.dst_y);
    Ok(NativeClientAreaBounds::new(
        left,
        top,
        left + i32::from(geometry.width),
        top + i32::from(geometry.height),
    ))
}

#[cfg(test)]
mod tests {
    use x11rb::protocol::xproto::MapState;

    use super::is_process_top_level;

    #[test]
    fn a_window_without_net_wm_pid_never_matches_even_when_viewable() {
        assert!(!is_process_top_level(None, 4242, MapState::VIEWABLE, false));
        assert!(is_process_top_level(
            Some(4242),
            4242,
            MapState::VIEWABLE,
            false
        ));
    }

    #[test]
    fn another_process_an_unmapped_window_or_an_override_redirect_cover_is_not_the_product() {
        assert!(!is_process_top_level(
            Some(1),
            4242,
            MapState::VIEWABLE,
            false
        ));
        assert!(!is_process_top_level(
            Some(4242),
            4242,
            MapState::UNMAPPED,
            false
        ));
        assert!(!is_process_top_level(
            Some(4242),
            4242,
            MapState::UNVIEWABLE,
            false
        ));
        assert!(!is_process_top_level(
            Some(4242),
            4242,
            MapState::VIEWABLE,
            true
        ));
    }
}
