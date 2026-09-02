use worth_ui_host_contract::{UiHostObservationCanonicalCore, UiHostObservationReport};

use crate::runtime::interaction::{
    UiInteractionTransition, UiPointerPresenceAdmissionDenial, UiPointerPresenceTargetTransition,
};
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

use super::super::UiInteractionRuntimeState;
use super::{draft_phase, emission, gesture_phase, pointer_admission, presence_phase};

pub(super) struct UiInteractionReportOutcome {
    transitions: Vec<UiInteractionTransition>,
    ignored: bool,
    pointer_presence_transition: Option<UiPointerPresenceTargetTransition>,
    pointer_presence_denials: Vec<UiPointerPresenceAdmissionDenial>,
}

impl UiInteractionReportOutcome {
    pub(super) fn into_parts(
        self,
    ) -> (
        Vec<UiInteractionTransition>,
        bool,
        Option<UiPointerPresenceTargetTransition>,
        Vec<UiPointerPresenceAdmissionDenial>,
    ) {
        (
            self.transitions,
            self.ignored,
            self.pointer_presence_transition,
            self.pointer_presence_denials,
        )
    }
}

pub(super) fn process(
    state: &mut UiInteractionRuntimeState,
    core: UiHostObservationCanonicalCore,
    report: &UiHostObservationReport,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    generation: &WorthUiActiveApplicationGenerationIdentity,
) -> UiInteractionReportOutcome {
    let mut admission = pointer_admission::UiPointerAdmission::from_report(report);
    let pointer_presence_transition = if admission.denied() {
        None
    } else {
        match presence_phase::process(
            &mut state.pointer_presence,
            &admission,
            core,
            report,
            mounted,
            generation,
        ) {
            Ok(transition) => transition,
            Err(denial) => {
                admission.deny(denial);
                None
            }
        }
    };
    let pointer = gesture_phase::process(
        &mut state.pointer,
        admission.take_stop(),
        core,
        report,
        admission.kind(),
        mounted,
    );
    let draft = draft_phase::process(
        &mut state.draft,
        admission.denied(),
        core,
        report,
        mounted,
        generation,
    );
    let ignored = !admission.denied()
        && pointer_presence_transition.is_none()
        && pointer.is_empty()
        && draft.is_empty();
    let mut transitions = emission::emit_pointer(state, pointer, core, generation);
    transitions.extend(emission::emit_draft(state, draft));
    UiInteractionReportOutcome {
        transitions,
        ignored,
        pointer_presence_transition,
        pointer_presence_denials: admission.into_denials(),
    }
}
