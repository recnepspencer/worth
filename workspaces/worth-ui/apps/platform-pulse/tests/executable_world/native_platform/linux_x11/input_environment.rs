//! What must be true of the X input world before the observer has effects,
//! and the cursor actuation both pointer paths share. Every variant names an
//! X11 fact; the Win32 desktop and foreground concepts have no referent here
//! and are not copied.
use std::fmt;

use x11rb::protocol::xproto::{ConnectionExt as _, InputFocus, Window};
use x11rb::protocol::xtest::ConnectionExt as _;

use super::connection::X11Observation;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum LinuxX11InputEnvironmentDenial {
    DisplayUnavailable(String),
    FocusTargetMismatch {
        target_window: u32,
        focus_window: u32,
    },
    CursorActuationDenied {
        requested: (i32, i32),
        error: String,
    },
    CursorActuationNotObserved {
        requested: (i32, i32),
        observed: (i32, i32),
    },
    PointerTargetMismatch {
        target_window: u32,
        hit_window: u32,
    },
    KeyboardFocusUnavailable {
        target_window: u32,
    },
}

impl fmt::Display for LinuxX11InputEnvironmentDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DisplayUnavailable(error) => {
                write!(formatter, "X display is unavailable: {error}")
            }
            Self::FocusTargetMismatch {
                target_window,
                focus_window,
            } => write!(
                formatter,
                "input focus target mismatch: target_window={target_window:#x}; focus_window={focus_window:#x}"
            ),
            Self::CursorActuationDenied { requested, error } => write!(
                formatter,
                "cursor actuation denied at ({}, {}): {error}",
                requested.0, requested.1
            ),
            Self::CursorActuationNotObserved {
                requested,
                observed,
            } => write!(
                formatter,
                "cursor actuation was not observed: requested=({}, {}); observed=({}, {})",
                requested.0, requested.1, observed.0, observed.1
            ),
            Self::PointerTargetMismatch {
                target_window,
                hit_window,
            } => write!(
                formatter,
                "pointer target mismatch before effects: target_window={target_window:#x}; hit_window={hit_window:#x}"
            ),
            Self::KeyboardFocusUnavailable { target_window } => write!(
                formatter,
                "keyboard focus was unavailable before effects: target_window={target_window:#x}"
            ),
        }
    }
}

/// XTEST `MotionNotify` with absolute root coordinates, then `QueryPointer`
/// must report exactly that position. The server's answer is the proof; the
/// request's acceptance is not.
pub(super) fn actuate_and_observe_cursor(
    x11: &X11Observation,
    requested: (i32, i32),
) -> Result<(), LinuxX11InputEnvironmentDenial> {
    let denied = |error: &dyn fmt::Display| LinuxX11InputEnvironmentDenial::CursorActuationDenied {
        requested,
        error: error.to_string(),
    };
    let (root_x, root_y) = (
        i16::try_from(requested.0).map_err(|error| denied(&error))?,
        i16::try_from(requested.1).map_err(|error| denied(&error))?,
    );
    let connection = x11.connection();
    connection
        .xtest_fake_input(
            MOTION_NOTIFY,
            0,
            x11rb::CURRENT_TIME,
            x11.root(),
            root_x,
            root_y,
            0,
        )
        .map_err(|error| denied(&error))?
        .check()
        .map_err(|error| denied(&error))?;
    let observed = observe_cursor(x11).map_err(|error| denied(&error))?;
    if observed == requested {
        Ok(())
    } else {
        Err(LinuxX11InputEnvironmentDenial::CursorActuationNotObserved {
            requested,
            observed,
        })
    }
}

pub(super) fn observe_cursor(x11: &X11Observation) -> Result<(i32, i32), String> {
    let pointer = x11
        .connection()
        .query_pointer(x11.root())
        .map_err(|error| error.to_string())?
        .reply()
        .map_err(|error| error.to_string())?;
    Ok((i32::from(pointer.root_x), i32::from(pointer.root_y)))
}

/// Direct the server's input focus at the bound window and require the
/// server to report it back. `InputFocus::PARENT` is what winit itself uses,
/// so focus reverts the same way the product expects if the window goes.
pub(super) fn qualify_keyboard_focus(
    x11: &X11Observation,
    target_window: Window,
) -> Result<(), LinuxX11InputEnvironmentDenial> {
    let connection = x11.connection();
    connection
        .set_input_focus(InputFocus::PARENT, target_window, x11rb::CURRENT_TIME)
        .map_err(x11rb::errors::ReplyError::from)
        .and_then(|cookie| cookie.check())
        .map_err(|_| LinuxX11InputEnvironmentDenial::KeyboardFocusUnavailable { target_window })?;
    let focus_window =
        focused_window(x11).map_err(LinuxX11InputEnvironmentDenial::DisplayUnavailable)?;
    if focus_window == target_window {
        Ok(())
    } else {
        Err(LinuxX11InputEnvironmentDenial::FocusTargetMismatch {
            target_window,
            focus_window,
        })
    }
}

pub(super) fn focused_window(x11: &X11Observation) -> Result<Window, String> {
    x11.connection()
        .get_input_focus()
        .map_err(|error| error.to_string())?
        .reply()
        .map(|reply| reply.focus)
        .map_err(|error| error.to_string())
}

// Core protocol event codes XTEST `FakeInput` takes as `type_`.
pub(super) const KEY_PRESS: u8 = 2;
pub(super) const KEY_RELEASE: u8 = 3;
pub(super) const BUTTON_PRESS: u8 = 4;
pub(super) const BUTTON_RELEASE: u8 = 5;
pub(super) const MOTION_NOTIFY: u8 = 6;

#[cfg(test)]
mod tests {
    use super::LinuxX11InputEnvironmentDenial;

    #[test]
    fn denial_display_preserves_observed_focus_identity() {
        let denial = LinuxX11InputEnvironmentDenial::FocusTargetMismatch {
            target_window: 0x33,
            focus_window: 0x44,
        };
        assert_eq!(
            denial.to_string(),
            "input focus target mismatch: target_window=0x33; focus_window=0x44"
        );
    }
}
