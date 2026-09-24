use winit::dpi::PhysicalPosition;

use super::super::{
    pointer, UiNativeInputObservationDisposition, UiNativeInputObservationEventFamily,
    UiNativeInputObservationState, UiNativeInputObservationStop,
};

pub(super) fn observe(
    state: &mut UiNativeInputObservationState,
    physical_position: PhysicalPosition<f64>,
) -> UiNativeInputObservationDisposition {
    if !state.admit_input(UiNativeInputObservationEventFamily::Pointer) {
        return state.rejection_disposition();
    }
    let Some(profile) = state.profile.profile() else {
        return state.terminal_disposition(UiNativeInputObservationStop::MissingEventProfile);
    };
    let position = match pointer::logical_position(physical_position, profile.scale_factor) {
        Ok(position) => position,
        Err(pointer::UiNativePointerCoordinateDenial::NotFinite) => {
            return state.terminal_disposition(UiNativeInputObservationStop::CoordinateNotFinite);
        }
        Err(pointer::UiNativePointerCoordinateDenial::OutOfRange) => {
            return state.terminal_disposition(UiNativeInputObservationStop::CoordinateOutOfRange);
        }
    };
    state.emit_payloads([state.pointer.motion(position)])
}
