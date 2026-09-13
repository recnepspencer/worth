use winsafe::{self as win, co, HWND};

use crate::external_observation::NativeClientAreaBounds;
use crate::native_platform::NativePlatformFailure;

use super::capture_region::client_bounds;

pub(super) struct ProcessWindowCandidate {
    pub(super) window: HWND,
    pub(super) bounds: NativeClientAreaBounds,
}

pub(super) fn enumerate(
    process_id: u32,
) -> Result<Vec<ProcessWindowCandidate>, NativePlatformFailure> {
    let mut candidates = Vec::new();
    let enumeration = win::EnumWindows(|window| {
        let (_, owner_process_id) = window.GetWindowThreadProcessId();
        if owner_process_id == process_id && window.IsWindowVisible() && !window.IsIconic() {
            if let Ok(client) = client_bounds(&window) {
                candidates.push(ProcessWindowCandidate {
                    window,
                    bounds: client,
                });
            }
        }
        true
    });
    require_diagnosable_result(enumeration)?;
    Ok(candidates)
}

fn require_diagnosable_result(result: win::SysResult<()>) -> Result<(), NativePlatformFailure> {
    match result {
        Ok(()) | Err(co::ERROR::SUCCESS) => Ok(()),
        Err(error) => Err(NativePlatformFailure::WindowEnumeration(error.to_string())),
    }
}

#[test]
fn zero_error_is_an_empty_or_complete_observation_not_a_failure() {
    assert!(require_diagnosable_result(Err(co::ERROR::SUCCESS)).is_ok());
    assert!(matches!(
        require_diagnosable_result(Err(co::ERROR::ACCESS_DENIED)),
        Err(NativePlatformFailure::WindowEnumeration(_))
    ));
}
