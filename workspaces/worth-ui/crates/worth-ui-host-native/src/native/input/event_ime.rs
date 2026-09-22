use super::{
    event_outcome::input_affine_batch_fits, text_ime as ime, UiNativeInputObservationDisposition,
    UiNativeInputObservationEventFamily, UiNativeInputObservationState,
    UiNativeInputObservationStop,
};
use winit::event::{Ime, WindowEvent};

impl UiNativeInputObservationState {
    pub(crate) fn ime_composition_posture(&self) -> ime::UiNativeImeCompositionPosture {
        if self.ime_composition_active {
            ime::UiNativeImeCompositionPosture::Composing
        } else {
            ime::UiNativeImeCompositionPosture::Idle
        }
    }
}

pub(super) fn observe(
    state: &mut UiNativeInputObservationState,
    event: &WindowEvent,
) -> Option<UiNativeInputObservationDisposition> {
    let WindowEvent::Ime(event) = event else {
        return None;
    };
    Some(observe_ime(state, event))
}

fn observe_ime(
    state: &mut UiNativeInputObservationState,
    event: &Ime,
) -> UiNativeInputObservationDisposition {
    if !state.admit_input(UiNativeInputObservationEventFamily::Ime) {
        return state.rejection_disposition();
    }
    let next_revision = state.next_revision;
    let composition_active = state.ime_composition_active;
    match event {
        Ime::Enabled => state.ime_enabled = true,
        Ime::Disabled => state.ime_enabled = false,
        Ime::Preedit(..) | Ime::Commit(..) => {}
    }
    let translated = match ime::translate(
        event,
        &mut state.next_revision,
        &mut state.ime_composition_active,
    ) {
        Ok(translated) => translated,
        Err(ime::UiNativeImeDenial::RevisionExhausted) => {
            return state.terminal_disposition(UiNativeInputObservationStop::TextRevisionExhausted);
        }
        Err(ime::UiNativeImeDenial::RangeNotScalarBoundary) => {
            return state
                .denial_disposition(UiNativeInputObservationStop::ImeRangeNotScalarBoundary);
        }
        Err(ime::UiNativeImeDenial::Preedit(denial)) => {
            return state.terminal_disposition(UiNativeInputObservationStop::ImePreedit(denial));
        }
    };
    if let Some(payload) = translated {
        if !input_affine_batch_fits(std::slice::from_ref(&payload)) {
            state.next_revision = next_revision;
            state.ime_composition_active = composition_active;
            return state.denial_disposition(UiNativeInputObservationStop::OverCapacityText);
        }
        let disposition = state.emit_payloads([payload]);
        if disposition != UiNativeInputObservationDisposition::Retained {
            state.next_revision = next_revision;
            state.ime_composition_active = composition_active;
        }
        disposition
    } else {
        UiNativeInputObservationDisposition::Ignored
    }
}
