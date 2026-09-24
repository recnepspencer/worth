use crate::facade::observation_report::{
    UiHostObservationBatch, UiHostObservationCanonicalCore, UiHostObservationFamily,
    UiHostObservationLoss, UiHostObservationReportDenial, UiHostObservationReportOutcome,
};
use crate::runtime::interaction::{
    UiHostInteractionIngressOutcome, UiInteractionLifecycleStopReason,
    UiInteractionObservationDenial, UiInteractionStateSnapshot, UiQuarantinedHostInteractionBatch,
};

use super::native_observation_settlement::UiNativeObservationIngressSettlement;
use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub(crate) fn drain_and_admit_host_observation_batches(
        &mut self,
        reachability: worth_ui_host_native::UiNativeInputReachability,
    ) -> UiNativeObservationIngressSettlement {
        let drain = match self.host_session.drain_observations() {
            Ok(drain) => drain,
            Err(denial) => {
                return UiNativeObservationIngressSettlement::DrainDenied(denial);
            }
        };
        let mut outcomes = Vec::new();
        let mut dismissals = Vec::new();
        for batch in drain.into_batches() {
            let (outcome, prepared) = self.admit_host_interaction_batch_with_dismissals(batch);
            outcomes.push(outcome);
            dismissals.extend(prepared);
        }
        // Reports retain their event-time epoch. Admit them while that epoch
        // is still current; a pending thumb press then waits for this physical
        // completion rather than being silently invalidated by it.
        if self.motion.is_installed() {
            self.complete_motion_sample_presentation();
        }
        UiNativeObservationIngressSettlement::with_dismissals(
            outcomes.into_boxed_slice(),
            dismissals.into_boxed_slice(),
            reachability,
        )
    }

    /// Validates raw host evidence and immediately moves admitted evidence into
    /// the interaction owner. Callers cannot supply pre-validated proxy input.
    pub fn admit_host_interaction_batch(
        &mut self,
        batch: UiHostObservationBatch,
    ) -> UiHostInteractionIngressOutcome {
        self.admit_host_interaction_batch_with_dismissals(batch).0
    }

    pub(super) fn admit_host_interaction_batch_with_dismissals(
        &mut self,
        batch: UiHostObservationBatch,
    ) -> (
        UiHostInteractionIngressOutcome,
        Vec<super::WorthUiAdmittedPortalDismissal>,
    ) {
        let mut dismissals = Vec::new();
        let previous_input = self.interaction.active_input_binding();
        let core = batch.canonical_core();
        let binding = core.binding();
        let outcome = match self.validate_host_observation_batch(batch) {
            UiHostObservationReportOutcome::Validated(batch) => {
                let portal_escape = self
                    .portal
                    .as_ref()
                    .is_some_and(|portal| portal.topmost_presentation().is_some())
                    .then(|| portal_escape_dismissal(batch.reports(), core.presentation()))
                    .flatten();
                let observation_tick = batch
                    .reports()
                    .iter()
                    .filter_map(|report| {
                        if let worth_ui_host_contract::UiHostObservationPayload::Tick { tick } =
                            report.report().payload()
                        {
                            Some(*tick)
                        } else {
                            None
                        }
                    })
                    .max();
                // Chrome answers first. A scrollbar is not a mounted node, so
                // a press on one would otherwise fall through to whatever node
                // is drawn beneath it; and while a thumb drag holds the
                // pointer, the moves and the release belong to the thumb.
                // Reports chrome does not claim reach ordinary routing
                // unchanged.
                let (scroll_chrome, chrome_claimed) = if self.scroll.is_installed() {
                    self.observe_scroll_chrome_pointer_reports(batch.reports(), core.presentation())
                        .into_parts()
                } else {
                    (Vec::new(), Vec::new())
                };
                let mut scroll_targeting_work = Default::default();
                let scroll_observations = if self.scroll.is_installed() {
                    batch
                        .reports()
                        .iter()
                        .filter_map(|report| {
                            // A host that stamps the batch with a Tick names the
                            // frame tick outright; otherwise a report observed on
                            // the host monotonic clock carries its own input tick,
                            // the clock every Motion frame is later sampled on.
                            let input_tick = observation_tick.or_else(|| {
                                match report.report().time_basis() {
                                    worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis(millis) => Some(millis),
                                    worth_ui_host_contract::UiHostObservationTimeBasis::HostWallClockMicros(_)
                                    | worth_ui_host_contract::UiHostObservationTimeBasis::PresentationRelativeTick(_) => None,
                                }
                            });
                            self.observe_scroll_payload(
                                report.report().payload(),
                                &mut scroll_targeting_work,
                                input_tick,
                            )
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let mut focus_publications = Vec::new();
                let command_routes = batch
                    .reports()
                    .iter()
                    .filter_map(|report| {
                        if let Some(focus) = self.focus.as_mut() {
                            focus.observe_host_payload(
                                report.report().payload(),
                                core.presentation(),
                            );
                        }
                        let focus_navigated = self.observe_focus_navigation_report(
                            report.report().payload(),
                            core.presentation(),
                            &mut focus_publications,
                        );
                        (!focus_navigated)
                            .then(|| {
                                self.observe_command_report(report.report(), core.presentation())
                            })
                            .flatten()
                    })
                    .collect();
                // A window that has stopped being the reader's leaves nothing
                // in flight. Every settle ends where its last accepted sample
                // put the content, and the wheel gesture gives its owner back,
                // so the first notch after the reader comes back is theirs to
                // aim rather than the tail of one they walked away from.
                if batch.reports().iter().any(|report| {
                    matches!(
                        report.report().payload(),
                        worth_ui_host_contract::UiHostObservationPayload::WindowFocus {
                            focused: false,
                            ..
                        }
                    )
                }) {
                    self.settle_scroll_after_attention_loss();
                }
                let generation = self.active_generation_identity();
                let mut receipt =
                    self.interaction
                        .ingest(batch, &self.mounted, &generation, &chrome_claimed);
                receipt.record_targeting_work(scroll_targeting_work);
                receipt.retain_scroll_observations(scroll_observations);
                receipt.retain_scroll_chrome_interactions(scroll_chrome);
                receipt.retain_command_routes(command_routes);
                receipt.retain_focus_publications(focus_publications);
                if let Some(dismissal) = portal_escape {
                    receipt.retain_service_dismissal(dismissal);
                }
                self.intent_evidence
                    .retain_transitions(receipt.transitions());
                // Seal service meaning before a Tick or completion advances the
                // accepted pixel epoch. Raw observations can then be retired.
                for transition in receipt.transitions() {
                    if let crate::runtime::interaction::UiInteractionTransition::DismissRequested(
                        input,
                    ) = transition
                    {
                        dismissals.push(self.prepare_portal_dismissal_interaction(*input));
                    }
                }
                if let Some(tick) = observation_tick.filter(|_| {
                    self.motion.is_installed() && self.mounted.has_active_motion_samples()
                }) {
                    let displayed = self
                        .mounted
                        .current_displayed_presentation(core.presentation());
                    if let Some(displayed) = displayed {
                        if let Ok(prepared) = self.prepare_motion_tick(tick, displayed) {
                            self.present_prepared_motion_tick(prepared, displayed);
                        }
                    }
                } else if self.awaits_scroll_settle_retry() {
                    // A settle deferred while a presentation was in flight is
                    // owed this frame even though no Motion tick asked for one.
                    // It settles on the surface its witness committed.
                    self.settle_owed_scroll_samples();
                }
                self.host_exchange
                    .retire_delivered_observation_batch(receipt.canonical_core());
                UiHostInteractionIngressOutcome::Applied(receipt)
            }
            UiHostObservationReportOutcome::Duplicate(duplicate) => {
                UiHostInteractionIngressOutcome::Duplicate(duplicate)
            }
            UiHostObservationReportOutcome::Quarantined(quarantined) => {
                let settlement = self.interaction.cancel_binding(
                    binding,
                    UiInteractionLifecycleStopReason::ObservationQuarantined,
                );
                UiHostInteractionIngressOutcome::Quarantined(
                    UiQuarantinedHostInteractionBatch::new(quarantined, settlement),
                )
            }
            UiHostObservationReportOutcome::Denied(denial) => {
                let settlement = if denial_invalidates_local_gesture(denial) {
                    self.interaction
                        .cancel_binding(binding, denial_stop_reason(denial, core))
                } else {
                    self.interaction.unchanged_settlement()
                };
                UiHostInteractionIngressOutcome::Denied(UiInteractionObservationDenial::new(
                    denial, settlement,
                ))
            }
        };
        self.clear_displaced_input_recipient(previous_input);
        (outcome, dismissals)
    }

    pub fn interaction_state(&self) -> UiInteractionStateSnapshot {
        self.interaction.snapshot()
    }

    pub(super) fn clear_displaced_input_recipient(
        &self,
        previous: Option<worth_ui_host_contract::UiHostInputRecipientBindingReceipt>,
    ) {
        if let Some(previous) = previous {
            if self.interaction.active_input_binding() != Some(previous) {
                let _ = self.host_session.clear_input_recipient(previous);
            }
        }
    }

    pub(super) fn cancel_all_interactions(&mut self, reason: UiInteractionLifecycleStopReason) {
        let previous_input = self.interaction.active_input_binding();
        self.interaction.cancel_all(reason);
        self.clear_displaced_input_recipient(previous_input);
    }
}

fn portal_escape_dismissal(
    reports: &[crate::facade::observation_report::UiValidatedHostObservationReport],
    presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
) -> Option<crate::runtime::interaction::UiDismissInteraction> {
    reports.iter().find_map(|validated| {
        let report = validated.report();
        matches!(
            report.payload(),
            worth_ui_host_contract::UiHostObservationPayload::Keyboard {
                logical_key: worth_ui_host_contract::UiHostKey::Escape,
                transition: worth_ui_host_contract::UiHostKeyTransition::Pressed { repeat: false },
                ..
            }
        )
        .then(|| {
            crate::runtime::interaction::UiDismissInteraction::escape(
                presentation,
                report.sequence(),
                report.time_basis(),
            )
        })
    })
}

fn denial_stop_reason(
    denial: UiHostObservationReportDenial,
    core: UiHostObservationCanonicalCore,
) -> UiInteractionLifecycleStopReason {
    match (denial, core.loss()) {
        (
            UiHostObservationReportDenial::LosslessOverflow(UiHostObservationFamily::PointerButton),
            UiHostObservationLoss::Overflow {
                family: UiHostObservationFamily::PointerButton,
                affected,
            },
        ) => UiInteractionLifecycleStopReason::ObservationLoss {
            family: UiHostObservationFamily::PointerButton,
            affected: Some(affected),
        },
        (UiHostObservationReportDenial::LosslessOverflow(family), _) => {
            UiInteractionLifecycleStopReason::ObservationLoss {
                family,
                affected: None,
            }
        }
        _ => UiInteractionLifecycleStopReason::ObservationInvalid,
    }
}

fn denial_invalidates_local_gesture(denial: UiHostObservationReportDenial) -> bool {
    matches!(
        denial,
        UiHostObservationReportDenial::SequenceGap
            | UiHostObservationReportDenial::SequenceReordered
            | UiHostObservationReportDenial::SequenceOverlap
            | UiHostObservationReportDenial::SequenceExhausted
            | UiHostObservationReportDenial::LosslessOverflow(_)
            | UiHostObservationReportDenial::UnknownFrame
            | UiHostObservationReportDenial::ExpiredFrame
            | UiHostObservationReportDenial::RejectedFrame
            | UiHostObservationReportDenial::NeverPresentedFrame
            | UiHostObservationReportDenial::BindingNotPresented
            | UiHostObservationReportDenial::PresentationEpochMismatch
            | UiHostObservationReportDenial::MountedInstanceNotPresented
            | UiHostObservationReportDenial::NodeReceiptMismatch
            | UiHostObservationReportDenial::FrameTransitionInFlight
    )
}
