use worth_ui_host_contract::{UiHostObservationPayload, UiHostPointerIdentity};

use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

use super::super::gesture::{UiPointerGestureOutcome, UiPointerGestureStopReason};
use super::super::{
    UiActivateInteraction, UiInteractionBatchReceipt, UiInteractionStop, UiInteractionTransition,
    UiPointerPresenceAdmissionDenial, UiSemanticInteraction,
};
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
            let mut pointer_presence_denied = false;
            let mut pointer_stop = None;
            let pointer_kind = match super::super::pointer_admission::admit(report) {
                Ok(kind) => kind,
                Err(denial) => {
                    record_pointer_denial(
                        &mut pointer_presence_denials,
                        &mut pointer_presence_denied,
                        &mut pointer_stop,
                        denial,
                    );
                    None
                }
            };
            let pointer_presence = if pointer_presence_denied {
                None
            } else {
                match (
                    self.pointer_presence.as_mut(),
                    pointer_kind,
                    report.payload(),
                ) {
                    (Some(owner), Some(kind), UiHostObservationPayload::PointerMotion { .. }) => {
                        match owner.process_pointer_report(core, report, kind, mounted, generation)
                        {
                            Ok(transition) => transition,
                            Err(denial) => {
                                record_pointer_denial(
                                    &mut pointer_presence_denials,
                                    &mut pointer_presence_denied,
                                    &mut pointer_stop,
                                    denial,
                                );
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
                            record_pointer_denial(
                                &mut pointer_presence_denials,
                                &mut pointer_presence_denied,
                                &mut pointer_stop,
                                denial,
                            );
                        }
                        None
                    }
                    _ => None,
                }
            };
            let pointer = if let Some((pointer, reason)) = pointer_stop {
                self.pointer
                    .stop_pointer_for_denial(pointer, report.sequence(), reason)
            } else {
                self.pointer
                    .process_report(core, report, pointer_kind, mounted)
            };
            let draft = if pointer_presence_denied {
                Vec::new()
            } else {
                self.draft.process_report(core, report, mounted, generation)
            };
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

fn record_pointer_denial(
    denials: &mut Vec<UiPointerPresenceAdmissionDenial>,
    denied: &mut bool,
    stop: &mut Option<(UiHostPointerIdentity, UiPointerGestureStopReason)>,
    denial: UiPointerPresenceAdmissionDenial,
) {
    *denied = true;
    if let Some(reason) = pointer_stop_reason(&denial) {
        *stop = Some((denial.pointer(), reason));
    }
    denials.push(denial);
}

fn pointer_stop_reason(
    denial: &UiPointerPresenceAdmissionDenial,
) -> Option<UiPointerGestureStopReason> {
    match denial {
        UiPointerPresenceAdmissionDenial::MissingDeviceKind { .. } => {
            Some(UiPointerGestureStopReason::MissingPointerDeviceKind)
        }
        UiPointerPresenceAdmissionDenial::CapacityExceeded { .. } => None,
        UiPointerPresenceAdmissionDenial::PointerKindChanged {
            prior, observed, ..
        } => Some(UiPointerGestureStopReason::PointerDeviceKindChanged {
            expected: *prior,
            observed: *observed,
        }),
    }
}
