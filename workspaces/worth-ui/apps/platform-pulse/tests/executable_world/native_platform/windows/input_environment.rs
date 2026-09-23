use std::fmt;
use std::time::Duration;

use uiautomation::inputs::Mouse;
use uiautomation::types::Point;
use winsafe::{co, HDESK, HWND};

const FOREGROUND_SETTLEMENT_ATTEMPTS: usize = 20;
const FOREGROUND_SETTLEMENT_INTERVAL: Duration = Duration::from_millis(5);

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WindowsInputEnvironmentDenial {
    InputDesktopUnavailable(String),
    ForegroundTargetMismatch {
        target_pid: u32,
        foreground_pid: u32,
        target_window: usize,
        foreground_window: usize,
        activation_accepted: bool,
    },
    CursorActuationDenied {
        requested: (i32, i32),
        error: String,
    },
    CursorActuationNotObserved {
        requested: (i32, i32),
        observed: (i32, i32),
    },
    /// The cursor would not stay on the control: something outside the harness
    /// holds this desktop's one physical pointer.
    CursorHeldAgainstHarness {
        requested: (i32, i32),
        observed: (i32, i32),
        attempts: u32,
    },
    PointerTargetMismatch {
        target_window: usize,
        hit_window: usize,
    },
    KeyboardFocusUnavailable {
        target_window: usize,
    },
}

impl fmt::Display for WindowsInputEnvironmentDenial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputDesktopUnavailable(error) => {
                write!(formatter, "input desktop is unavailable: {error}")
            }
            Self::ForegroundTargetMismatch {
                target_pid,
                foreground_pid,
                target_window,
                foreground_window,
                activation_accepted,
            } => write!(
                formatter,
                "foreground target mismatch: target_pid={target_pid}; foreground_pid={foreground_pid}; target_window={target_window:#x}; foreground_window={foreground_window:#x}; set_foreground_window={activation_accepted}"
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
            Self::CursorHeldAgainstHarness {
                requested,
                observed,
                attempts,
            } => write!(
                formatter,
                "the cursor would not stay on ({}, {}) across {attempts} attempts; it sat at ({}, {}). Another hand or window holds this desktop's pointer",
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

pub(super) fn qualify_keyboard_world(
    window: &HWND,
    target_pid: u32,
) -> Result<(), WindowsInputEnvironmentDenial> {
    qualify_input_desktop()?;
    qualify_foreground_target(window, target_pid)
}

pub(super) fn qualify_pointer_world(
    window: &HWND,
    target_pid: u32,
    requested: (i32, i32),
) -> Result<(), WindowsInputEnvironmentDenial> {
    qualify_input_desktop()?;
    qualify_foreground_target(window, target_pid)?;
    actuate_and_observe_cursor(requested)
}

pub(super) fn actuate_and_observe_cursor(
    requested: (i32, i32),
) -> Result<(), WindowsInputEnvironmentDenial> {
    Mouse::set_cursor_pos(&Point::new(requested.0, requested.1)).map_err(|error| {
        WindowsInputEnvironmentDenial::CursorActuationDenied {
            requested,
            error: error.to_string(),
        }
    })?;
    let observed = Mouse::get_cursor_pos().map_err(|error| {
        WindowsInputEnvironmentDenial::CursorActuationDenied {
            requested,
            error: error.to_string(),
        }
    })?;
    let observed = (observed.get_x(), observed.get_y());
    if observed == requested {
        Ok(())
    } else {
        Err(WindowsInputEnvironmentDenial::CursorActuationNotObserved {
            requested,
            observed,
        })
    }
}

fn qualify_input_desktop() -> Result<(), WindowsInputEnvironmentDenial> {
    // Independently opened HDESK values are handles, not stable identity
    // tokens. The foreground-window check below is the identity proof: a
    // foreground target belongs to the active input desktop.
    let _input_desktop = HDESK::OpenInputDesktop(
        None,
        false,
        co::DESKTOP_RIGHTS::READOBJECTS | co::DESKTOP_RIGHTS::ENUMERATE,
    )
    .map_err(|error| WindowsInputEnvironmentDenial::InputDesktopUnavailable(error.to_string()))?;
    Ok(())
}

fn qualify_foreground_target(
    window: &HWND,
    target_pid: u32,
) -> Result<(), WindowsInputEnvironmentDenial> {
    let foreground = HWND::GetForegroundWindow();
    let foreground_pid = foreground
        .as_ref()
        .map_or(0, |handle| handle.GetWindowThreadProcessId().1);
    if foreground.as_ref() == Some(window) && foreground_pid == target_pid {
        return Ok(());
    }
    let activation_accepted = window.SetForegroundWindow();
    for attempt in 0..FOREGROUND_SETTLEMENT_ATTEMPTS {
        let foreground = HWND::GetForegroundWindow();
        let foreground_pid = foreground
            .as_ref()
            .map_or(0, |handle| handle.GetWindowThreadProcessId().1);
        if foreground.as_ref() == Some(window) && foreground_pid == target_pid {
            return Ok(());
        }
        if attempt + 1 < FOREGROUND_SETTLEMENT_ATTEMPTS {
            std::thread::sleep(FOREGROUND_SETTLEMENT_INTERVAL);
        }
    }
    let foreground = HWND::GetForegroundWindow();
    let foreground_pid = foreground
        .as_ref()
        .map_or(0, |handle| handle.GetWindowThreadProcessId().1);
    Err(WindowsInputEnvironmentDenial::ForegroundTargetMismatch {
        target_pid,
        foreground_pid,
        target_window: window.ptr() as usize,
        foreground_window: foreground.map_or(0, |handle| handle.ptr() as usize),
        activation_accepted,
    })
}

#[cfg(test)]
mod tests {
    use super::WindowsInputEnvironmentDenial;

    #[test]
    fn denial_display_preserves_observed_foreground_identity() {
        let denial = WindowsInputEnvironmentDenial::ForegroundTargetMismatch {
            target_pid: 11,
            foreground_pid: 22,
            target_window: 0x33,
            foreground_window: 0x44,
            activation_accepted: false,
        };
        assert_eq!(
            denial.to_string(),
            "foreground target mismatch: target_pid=11; foreground_pid=22; target_window=0x33; foreground_window=0x44; set_foreground_window=false"
        );
    }
}
