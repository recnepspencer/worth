use worth_ui_host_contract::{UiHostObservationCanonicalCore, UiHostObservationReport};

use crate::runtime::interaction::{
    UiInteractionTransition, UiPointerPresenceAdmissionDenial, UiPointerPresenceTargetTransition,
};
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

use super::super::UiInteractionRuntimeState;
use super::{draft_phase, emission, gesture_phase, pointer_admission, presence_phase};

pub(super) struct UiInteractionReportOutcome {
    pub(super) targeting_work: crate::mounting::UiHitTestSpatialWork,
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

/// The outcome of a report this owner did not route because another lane had
/// already answered for it. It counts as ignored, exactly as a report this
/// owner itself declines does.
pub(super) fn claimed_elsewhere() -> UiInteractionReportOutcome {
    UiInteractionReportOutcome {
        targeting_work: Default::default(),
        transitions: Vec::new(),
        ignored: true,
        pointer_presence_transition: None,
        pointer_presence_denials: Vec::new(),
    }
}

pub(super) fn process(
    state: &mut UiInteractionRuntimeState,
    core: UiHostObservationCanonicalCore,
    report: &UiHostObservationReport,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    generation: &WorthUiActiveApplicationGenerationIdentity,
) -> UiInteractionReportOutcome {
    let mut targeting_work = Default::default();
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
            &mut targeting_work,
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
        &mut targeting_work,
    );
    let draft = draft_phase::process(
        &mut state.draft,
        admission.denied(),
        core,
        report,
        mounted,
        generation,
    );
    let mut transitions = emission::emit_pointer(state, pointer, generation);
    // Outside presses are Portal input even when shielding admits no hit
    // target. They must not manufacture a Pressed or Activate for that target.
    if !admission.denied() {
        transitions.extend(emission::outside_press(core, report));
    }
    let ignored = !admission.denied()
        && pointer_presence_transition.is_none()
        && transitions.is_empty()
        && draft.is_empty();
    transitions.extend(emission::emit_draft(state, draft));
    UiInteractionReportOutcome {
        targeting_work,
        transitions,
        ignored,
        pointer_presence_transition,
        pointer_presence_denials: admission.into_denials(),
    }
}
