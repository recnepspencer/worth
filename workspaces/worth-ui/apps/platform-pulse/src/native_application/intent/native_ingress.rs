use worth_ui::facade::app::{
    WorthUiNativeApplicationShell, WorthUiNativeIntentPosture, WorthUiNativeIntentPostureKind,
    WorthUiNativeIntentStop, WorthUiNativeIntentTransition,
    WorthUiNativeManagedIntentPosturePublicationOutcome,
};
use worth_ui_platform_pulse::observation_contract::PlatformPulseIntentPostureObservation;

use super::super::PlatformPulseApplicationRuntime;

const MAX_PENDING_INTENT_POSTURES: usize = 64;

pub(in crate::native_application) struct PlatformPulsePreparedIntentPosture {
    posture: WorthUiNativeIntentPosture,
    pending: PlatformPulsePendingIntentPosture,
}

pub(in crate::native_application) struct PlatformPulsePendingIntentPosture {
    observation: PlatformPulseIntentPostureObservation,
    settlement: PlatformPulseIntentPostureSettlement,
}

pub(in crate::native_application) enum PlatformPulseIntentPostureSettlement {
    PublicationOnly,
    RetireExecution {
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
    },
}

pub(in crate::native_application) enum PlatformPulseIntentPosturePublicationDisposition {
    Published,
    Pending,
    Failed,
}

impl PlatformPulsePreparedIntentPosture {
    pub(in crate::native_application) fn new(
        posture: WorthUiNativeIntentPosture,
        observation: PlatformPulseIntentPostureObservation,
        settlement: PlatformPulseIntentPostureSettlement,
    ) -> Self {
        Self {
            posture,
            pending: PlatformPulsePendingIntentPosture {
                observation,
                settlement,
            },
        }
    }
}

impl PlatformPulseIntentPostureSettlement {
    pub(in crate::native_application) const fn retire_execution(
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
    ) -> Self {
        Self::RetireExecution {
            attempt,
            idempotency,
        }
    }
}

impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn admit_worth_native_intent_input(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        progress: worth_ui_native_platform::UiNativeApplicationObservationProgress,
    ) {
        let deadline = match self.intent_clock.new_attempt_deadline() {
            Ok(deadline) => deadline,
            Err(denial) => {
                self.fail_intent_clock(denial);
                return;
            }
        };
        let ingress = shell.admit_native_intent_progress_triplet(
            worth_ui_platform_pulse::intent::platform_pulse_action_definition(),
            worth_ui_platform_pulse::intent::platform_pulse_open_portal_definition(),
            worth_ui_platform_pulse::intent::platform_pulse_close_portal_definition(),
            progress,
            deadline,
        );
        self.settle_native_intent_ingress(shell, ingress);
    }

    fn settle_native_intent_ingress(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        ingress: worth_ui::facade::app::WorthUiNativeIntentIngress,
    ) {
        let dismissals = ingress.dismissals().to_vec();
        if ingress.duplicate_batches() > 0 {
            self.fail_intent_settlement(format!(
                "native interaction ingress duplicated {} batch(es)",
                ingress.duplicate_batches()
            ));
            return;
        }
        for transition in ingress.into_transitions() {
            let Some(prepared) = self.prepare_ingress_intent_posture(transition) else {
                if self.terminal_error.is_some() {
                    break;
                }
                continue;
            };
            if self.pending_intent_postures.len() == MAX_PENDING_INTENT_POSTURES {
                self.fail_intent_settlement(format!(
                    "native intent posture queue exceeded capacity {MAX_PENDING_INTENT_POSTURES}"
                ));
                break;
            }
            self.pending_intent_postures.push_back(prepared);
        }
        for dismissal in dismissals {
            if self.terminal_error.is_some() || !self.dismiss_open_portal(shell, dismissal) {
                break;
            }
        }
        if self.terminal_error.is_none() {
            self.advance_pending_intent_postures(shell);
        }
    }

    fn prepare_ingress_intent_posture(
        &mut self,
        transition: WorthUiNativeIntentTransition,
    ) -> Option<PlatformPulsePreparedIntentPosture> {
        let (posture, observation) = match transition {
            WorthUiNativeIntentTransition::AttemptPrepared(prepared) => {
                if let Err(denial) = self.intent_evidence_index.retain(prepared.dispatch()) {
                    self.fail_intent_settlement(format!(
                        "prepared intent evidence could not be retained: {denial:?}"
                    ));
                    return None;
                }
                let observation =
                    PlatformPulseIntentPostureObservation::admitted(prepared.dispatch());
                (prepared.into_posture(), observation)
            }
            WorthUiNativeIntentTransition::ConfirmationRequired(pending) => {
                let observation =
                    PlatformPulseIntentPostureObservation::confirmation_required(pending.pending());
                let (_, posture) = pending.into_parts();
                (posture, observation)
            }
            WorthUiNativeIntentTransition::Stopped(stopped) => {
                let (stop, posture) = stopped.into_parts();
                if posture.is_none() {
                    if let Some(observation) = unrouted_observation(&stop) {
                        if let Err(error) = self.publisher.intent_routing_stopped(observation) {
                            self.fail(
                                super::super::PlatformPulseTerminalError::ObservationPublication,
                                Err(error),
                            );
                        }
                        return None;
                    }
                }
                let Some(posture) = posture else {
                    self.fail_intent_settlement(format!(
                        "native intent stopped without a publishable posture: {}",
                        native_intent_stop_kind(&stop)
                    ));
                    return None;
                };
                let Some(observation) = stopped_posture_observation(posture.kind()) else {
                    self.fail_intent_settlement(
                        "native intent stopped with an invalid terminal posture",
                    );
                    return None;
                };
                (posture, observation)
            }
        };
        Some(PlatformPulsePreparedIntentPosture::new(
            posture,
            observation,
            PlatformPulseIntentPostureSettlement::PublicationOnly,
        ))
    }

    pub(in crate::native_application) fn advance_pending_intent_postures(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) {
        while self.terminal_error.is_none()
            && self.pending_managed_rebind.is_none()
            && self.pending_frame_presentation.is_none()
        {
            let Some(prepared) = self.pending_intent_postures.pop_front() else {
                return;
            };
            if !matches!(
                self.publish_native_intent_posture(shell, prepared),
                PlatformPulseIntentPosturePublicationDisposition::Published
            ) {
                return;
            }
        }
    }

    pub(in crate::native_application::intent) fn publish_native_intent_posture(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        prepared: PlatformPulsePreparedIntentPosture,
    ) -> PlatformPulseIntentPosturePublicationDisposition {
        self.presentation_tick = self.presentation_tick.saturating_add(1);
        let PlatformPulsePreparedIntentPosture { posture, pending } = prepared;
        let outcome =
            shell.begin_managed_native_intent_posture_publication(posture, self.presentation_tick);
        let receipt = match outcome {
            Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Published(receipt)) => receipt,
            Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Pending) => {
                self.pending_managed_rebind =
                    Some(super::super::PlatformPulsePendingManagedRebind::IntentPosture(pending));
                return PlatformPulseIntentPosturePublicationDisposition::Pending;
            }
            Ok(WorthUiNativeManagedIntentPosturePublicationOutcome::Stopped(stop)) => {
                self.fail_intent_posture_publication(
                    super::PlatformPulseIntentPosturePublicationDenial::Stopped(stop),
                );
                return PlatformPulseIntentPosturePublicationDisposition::Failed;
            }
            Err(denial) => {
                self.fail_intent_posture_publication(
                    super::PlatformPulseIntentPosturePublicationDenial::Managed(denial),
                );
                return PlatformPulseIntentPosturePublicationDisposition::Failed;
            }
        };
        if self.settle_intent_posture_publication(shell, pending, receipt) {
            PlatformPulseIntentPosturePublicationDisposition::Published
        } else {
            PlatformPulseIntentPosturePublicationDisposition::Failed
        }
    }

    pub(in crate::native_application) fn settle_intent_posture_publication(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        pending: PlatformPulsePendingIntentPosture,
        receipt: worth_ui::facade::rebind::UiRebindReceipt,
    ) -> bool {
        let PlatformPulsePendingIntentPosture {
            observation,
            settlement,
        } = pending;
        let refresh_product_story = !matches!(
            &observation,
            PlatformPulseIntentPostureObservation::Admitted { .. }
        );
        let Some(mounted) = receipt.mounted_publication() else {
            self.fail_intent_settlement("intent posture receipt omitted mounted publication");
            return false;
        };
        let command_transition = super::latest_command_transition(shell);
        if let Err(error) =
            self.publisher
                .intent_posture_published(observation, mounted, command_transition)
        {
            self.fail(
                super::super::PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return false;
        }
        if let Err(denial) = self.visual_identity.refresh_after_presentation_replacement(
            shell,
            self.presentation_tick,
            std::time::Instant::now(),
        ) {
            self.fail_visual_identity(denial);
            return false;
        }
        if let PlatformPulseIntentPostureSettlement::RetireExecution {
            attempt,
            idempotency,
        } = settlement
        {
            if self
                .intent_evidence_index
                .retire_execution(attempt, idempotency)
                .is_none()
            {
                self.fail_intent_settlement(
                    "terminal attempt omitted its retained intent evidence reference",
                );
                return false;
            }
        }
        if refresh_product_story {
            if let Some(denial) = self.pending_query_denial_story.take() {
                if !self.publish_query_denial_story(shell, denial) {
                    return false;
                }
            }
            if !self.refresh_product_story(shell) {
                return false;
            }
        }
        true
    }

    fn fail_intent_posture_publication(
        &mut self,
        denial: super::PlatformPulseIntentPosturePublicationDenial,
    ) {
        let observation = self.publisher.intent_preparation_failure();
        self.fail(
            super::super::PlatformPulseTerminalError::IntentPosturePublication(denial),
            observation,
        );
    }
}

fn native_intent_stop_kind(stop: &WorthUiNativeIntentStop) -> &'static str {
    match stop {
        WorthUiNativeIntentStop::Route(_) => "route",
        WorthUiNativeIntentStop::Payload(_) => "payload",
        WorthUiNativeIntentStop::Admission(_) => "admission",
        WorthUiNativeIntentStop::Confirmation(_) => "confirmation",
        WorthUiNativeIntentStop::Dispatch(_) => "dispatch",
        WorthUiNativeIntentStop::PostureIdentityExhausted => "posture-identity-exhausted",
        WorthUiNativeIntentStop::DefinitionNotSelected => "definition-not-selected",
    }
}

fn unrouted_observation(
    stop: &WorthUiNativeIntentStop,
) -> Option<
    worth_ui_platform_pulse::observation_contract::PlatformPulseIntentRoutingStoppedObservation,
> {
    let WorthUiNativeIntentStop::Route(
        worth_ui::facade::intent::UiIntentRouteResolutionStop::Unrouted {
            graph_node,
            interaction,
        },
    ) = stop
    else {
        return None;
    };
    Some(
        worth_ui_platform_pulse::observation_contract::
            PlatformPulseIntentRoutingStoppedObservation::unrouted(*graph_node, *interaction),
    )
}

fn stopped_posture_observation(
    kind: WorthUiNativeIntentPostureKind,
) -> Option<PlatformPulseIntentPostureObservation> {
    match kind {
        WorthUiNativeIntentPostureKind::Denied => {
            Some(PlatformPulseIntentPostureObservation::Denied)
        }
        WorthUiNativeIntentPostureKind::StaleConfirmation => {
            Some(PlatformPulseIntentPostureObservation::StaleConfirmation)
        }
        WorthUiNativeIntentPostureKind::Admitted
        | WorthUiNativeIntentPostureKind::ConfirmationRequired
        | WorthUiNativeIntentPostureKind::Completed
        | WorthUiNativeIntentPostureKind::Cancelled => None,
    }
}
