use super::super::UiInteractionRuntimeState;
use super::report::UiInteractionReportOutcome;
use crate::runtime::interaction::{
    UiInteractionBatchReceipt, UiInteractionTransition, UiPointerPresenceAdmissionDenial,
    UiPointerPresenceTargetTransition,
};

#[derive(Default)]
pub(super) struct UiInteractionBatchReceiptBuilder {
    transitions: Vec<UiInteractionTransition>,
    ignored_reports: usize,
    pointer_presence_transitions: Vec<UiPointerPresenceTargetTransition>,
    pointer_presence_denials: Vec<UiPointerPresenceAdmissionDenial>,
}

impl UiInteractionBatchReceiptBuilder {
    pub(super) fn record(&mut self, outcome: UiInteractionReportOutcome) {
        let (transitions, ignored, pointer_presence_transition, pointer_presence_denials) =
            outcome.into_parts();
        self.transitions.extend(transitions);
        if ignored {
            self.ignored_reports += 1;
        }
        if let Some(transition) = pointer_presence_transition {
            self.pointer_presence_transitions.push(transition);
        }
        self.pointer_presence_denials
            .extend(pointer_presence_denials);
    }

    pub(super) fn finish(
        self,
        batch: crate::facade::observation_report::UiValidatedHostObservationBatch,
        state: &UiInteractionRuntimeState,
    ) -> UiInteractionBatchReceipt {
        UiInteractionBatchReceipt {
            core: batch.canonical_core(),
            frame_relation: batch.frame_relation(),
            disposition: batch.disposition(),
            transitions: self.transitions.into_boxed_slice(),
            ignored_reports: self.ignored_reports,
            state: state.snapshot(),
            scroll_observations: Box::new([]),
            command_routes: Box::new([]),
            pointer_presence_transitions: self.pointer_presence_transitions.into_boxed_slice(),
            pointer_presence_denials: self.pointer_presence_denials.into_boxed_slice(),
        }
    }
}
