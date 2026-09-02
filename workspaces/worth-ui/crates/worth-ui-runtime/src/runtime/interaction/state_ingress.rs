use worth_ui_host_contract::UiHostObservationPayload;

use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

use super::super::gesture::UiPointerGestureOutcome;
use super::super::{
    UiActivateInteraction, UiInteractionBatchReceipt, UiInteractionStop, UiInteractionTransition,
    UiSemanticInteraction,
};
use super::super::pointer_presence::UiPrimaryPointerKind;

impl super::UiInteractionRuntimeState {
    pub(crate) fn ingest(
        &mut self,
        batch: crate::facade::observation_report::UiValidatedHostObservationBatch,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        generation: &WorthUiActiveApplicationGenerationIdentity,
    ) -> UiInteractionBatchReceipt {
        let core = batch.canonical_core();
        let mut transitions = Vec::new();
        let mut ignored_reports = 0;
        let mut pointer_presence_transitions = Vec::new();
        let mut pointer_presence_denials = Vec::new();
        for validated in batch.reports() {
            let report = validated.report();
            let pointer_kind = report
                .effective_pointer_device_kind()
                .map(UiPrimaryPointerKind::from_host);
            let mut pointer_presence_denied = false;
            let pointer_presence = match (
                self.pointer_presence.as_mut(),
                pointer_kind,
                report.payload(),
            ) {
                (Some(owner), Some(kind), UiHostObservationPayload::PointerMotion { .. }) => {
                    match owner.process_pointer_report(core, report, kind, mounted, generation) {
                        Ok(transition) => transition,
                        Err(denial) => {
                            pointer_presence_denied = true;
                            pointer_presence_denials.push(denial);
                            None
                        }
                    }
                }
                (
                    Some(owner),
                    Some(kind),
                    UiHostObservationPayload::PointerButton { pointer, .. },
                ) => {
                    if let Err(denial) = owner.admit_pointer_kind(*pointer, kind) {
                        pointer_presence_denied = true;
                        pointer_presence_denials.push(denial);
                    }
                    None
                }
                _ => None,
            };
            let pointer = self.pointer.process_report(
                core,
                report,
                pointer_kind.unwrap_or(UiPrimaryPointerKind::Mouse),
                mounted,
            );
            let draft = self.draft.process_report(core, report, mounted, generation);
            if !pointer_presence_denied
                && pointer_presence.is_none()
                && pointer.is_empty()
                && draft.is_empty()
            {
                ignored_reports += 1;
            }
            pointer_presence_transitions.extend(pointer_presence);
            for outcome in pointer {
                match outcome {
                    UiPointerGestureOutcome::Pressed(press) => {
                        let dismissal = super::super::UiDismissInteraction::outside_press(
                            core.presentation(),
                            press.sequence(),
                            press.time_basis(),
                            press.position(),
                        );
                        transitions.push(UiInteractionTransition::PointerPressed(press));
                        transitions.push(UiInteractionTransition::DismissRequested(dismissal));
                    }
                    UiPointerGestureOutcome::Completed(gesture) => {
                        if gesture.pointer_device_kind()
                            == worth_ui_host_contract::UiHostPointerDeviceKind::Touch
                        {
                            continue;
                        }
                        self.record_semantic();
                        transitions.push(UiInteractionTransition::Semantic(
                            UiSemanticInteraction::Activate(UiActivateInteraction::from_pointer(
                                gesture,
                                generation.clone(),
                            )),
                        ));
                    }
                    UiPointerGestureOutcome::Stopped(stop) => {
                        transitions.push(UiInteractionTransition::Stopped(
                            UiInteractionStop::PointerGesture(stop),
                        ));
                    }
                }
            }
            transitions.extend(draft.into_iter().map(|outcome| match outcome {
                super::super::draft::UiDraftProcessingOutcome::Mutation(receipt) => {
                    UiInteractionTransition::DraftMutation(receipt)
                }
                super::super::draft::UiDraftProcessingOutcome::DismissRequested(interaction) => {
                    UiInteractionTransition::DismissRequested(interaction)
                }
                super::super::draft::UiDraftProcessingOutcome::Semantic(interaction) => {
                    self.record_semantic();
                    UiInteractionTransition::Semantic(interaction)
                }
                super::super::draft::UiDraftProcessingOutcome::Stopped(stop) => {
                    UiInteractionTransition::Stopped(UiInteractionStop::LocalInput(stop))
                }
            }));
        }
        UiInteractionBatchReceipt {
            core,
            frame_relation: batch.frame_relation(),
            disposition: batch.disposition(),
            transitions: transitions.into_boxed_slice(),
            ignored_reports,
            state: self.snapshot(),
            scroll_observations: Box::new([]),
            command_routes: Box::new([]),
            pointer_presence_transitions: pointer_presence_transitions.into_boxed_slice(),
            pointer_presence_denials: pointer_presence_denials.into_boxed_slice(),
        }
    }
}
