use winit::event::{ElementState, MouseButton};
use worth_ui_host_contract::UiHostPointerButtonTransition;

use super::super::{
    pointer, UiNativeInputObservationDisposition, UiNativeInputObservationEventFamily,
    UiNativeInputObservationState, UiNativeInputObservationStop, UiNativePointerPositionWitness,
};

pub(super) fn observe(
    state: &mut UiNativeInputObservationState,
    button_state: ElementState,
    button: MouseButton,
    pointer_witness: UiNativePointerPositionWitness,
) -> UiNativeInputObservationDisposition {
    if !state.admit_input(UiNativeInputObservationEventFamily::Pointer) {
        return state.rejection_disposition();
    }
    let Some(button) = pointer::button(button) else {
        return state.terminal_disposition(UiNativeInputObservationStop::Unsupported(
            UiNativeInputObservationEventFamily::Pointer,
        ));
    };
    let transition = match button_state {
        ElementState::Pressed => UiHostPointerButtonTransition::Pressed,
        ElementState::Released => UiHostPointerButtonTransition::Released,
    };
    let position = match event_position(state, pointer_witness) {
        Ok(position) => position,
        Err(stop) => return state.terminal_disposition(stop),
    };
    let disposition = state.emit_payloads([state.pointer.button(button, transition, position)]);
    if disposition == UiNativeInputObservationDisposition::Retained {
        state
            .pointer
            .set_pressed(button, button_state == ElementState::Pressed);
    }
    disposition
}

fn event_position(
    state: &UiNativeInputObservationState,
    witness: UiNativePointerPositionWitness,
) -> Result<worth_ui_host_contract::UiHostSurfacePosition, UiNativeInputObservationStop> {
    let UiNativePointerPositionWitness::EventTime(position) = witness else {
        return Err(UiNativeInputObservationStop::PointerPositionUnavailable);
    };
    let profile = state
        .profile
        .profile()
        .ok_or(UiNativeInputObservationStop::MissingEventProfile)?;
    pointer::logical_position(position, profile.scale_factor).map_err(|denial| match denial {
        pointer::UiNativePointerCoordinateDenial::NotFinite => {
            UiNativeInputObservationStop::CoordinateNotFinite
        }
        pointer::UiNativePointerCoordinateDenial::OutOfRange => {
            UiNativeInputObservationStop::CoordinateOutOfRange
        }
    })
}
