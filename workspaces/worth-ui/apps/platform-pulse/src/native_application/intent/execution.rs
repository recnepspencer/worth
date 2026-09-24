use super::super::{PlatformPulseApplicationRuntime, PlatformPulseTerminalError};
use super::{
    PlatformPulseIntentExecutionProgress, PlatformPulseIntentPosturePublicationDisposition,
    PlatformPulseIntentPostureSettlement, PlatformPulsePreparedIntentPosture,
};
use worth_ui::facade::app::{
    WorthUiNativeApplicationShell, WorthUiNativeIntentTerminalPostureOutcome,
    WorthUiNativeManagedIntentConsequencePublicationOutcome,
};
use worth_ui::facade::intent::{
    UiIntentExecutionAdvanceOutcome, UiIntentExecutionTransition,
    UiIntentExecutionTransitionPosture,
};
use worth_ui_platform_pulse::observation_contract::PlatformPulseIntentPostureObservation;
mod consequence_publication;
mod query_completion;
use consequence_publication::consequence_kind;
pub(in crate::native_application) use consequence_publication::PlatformPulsePendingIntentConsequence;

enum PlatformPulseIntentTransitionContinuation {
    ContinueToConsequence,
    Finished(bool),
}

const MAX_PENDING_INTENT_EXECUTION_TRANSITIONS: usize = 64;

impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn advance_intent_execution(
        &mut self,
    ) -> PlatformPulseIntentExecutionProgress {
        let Some(mut shell) = self.shell.take() else {
            return PlatformPulseIntentExecutionProgress::Idle;
        };
        let mut locally_progressed = self
            .pending_intent_execution_transitions
            .iter()
            .any(|transition| is_local_progress(transition.posture()));
        let pending_before = self.pending_intent_execution_transitions.len();
        self.drain_pending_intent_execution_transitions(&mut shell);
        let mut progress =
            pending_before.saturating_sub(self.pending_intent_execution_transitions.len());
        if self.terminal.is_stopped() || self.pending_managed_rebind.is_some() {
            self.shell = Some(shell);
            return PlatformPulseIntentExecutionProgress::from_transitions(
                progress,
                locally_progressed,
            );
        }
        let reading = match self.intent_clock.read() {
            Ok(reading) => reading,
            Err(denial) => {
                self.fail_intent_clock(denial);
                self.shell = Some(shell);
                return PlatformPulseIntentExecutionProgress::from_transitions(
                    progress,
                    locally_progressed,
                );
            }
        };

        match shell.advance_native_intent_executions(reading) {
            UiIntentExecutionAdvanceOutcome::Stopped(stop) => {
                self.fail(
                    PlatformPulseTerminalError::IntentExecution(format!(
                        "execution clock stopped: {stop:?}"
                    )),
                    self.publisher.intent_preparation_failure(),
                );
            }
            UiIntentExecutionAdvanceOutcome::Advanced(report) => {
                if !self.publish_intent_executor_starts(&report) {
                    self.shell = Some(shell);
                    return PlatformPulseIntentExecutionProgress::from_transitions(
                        progress,
                        locally_progressed,
                    );
                }
                let transitions = report.into_transitions();

                progress = progress.saturating_add(transitions.len());
                locally_progressed |= transitions
                    .iter()
                    .any(|transition| is_local_progress(transition.posture()));
                if self.pending_intent_execution_transitions.len() + transitions.len()
                    > MAX_PENDING_INTENT_EXECUTION_TRANSITIONS
                {
                    self.fail_intent_settlement(format!(
                        "intent execution transition queue exceeded capacity {MAX_PENDING_INTENT_EXECUTION_TRANSITIONS}"
                    ));
                } else {
                    self.pending_intent_execution_transitions
                        .extend(transitions.into_vec());
                    self.drain_pending_intent_execution_transitions(&mut shell);
                }
            }
        }
        self.shell = Some(shell);
        PlatformPulseIntentExecutionProgress::from_transitions(progress, locally_progressed)
    }

    fn drain_pending_intent_execution_transitions(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
    ) {
        while self.terminal.is_running() && self.pending_managed_rebind.is_none() {
            let Some(transition) = self.pending_intent_execution_transitions.pop_front() else {
                return;
            };
            if !self.finish_intent_transition(shell, transition) {
                return;
            }
        }
    }

    fn publish_intent_executor_starts(
        &mut self,
        report: &worth_ui::facade::intent::UiIntentExecutionAdvanceReport,
    ) -> bool {
        for observation in report.transitions().iter().filter_map(|transition| {
            worth_ui_platform_pulse::observation_contract::
                PlatformPulseIntentExecutorStartedObservation::from_transition(report, transition)
        }) {
            if let Err(error) = self.publisher.intent_executor_started(observation) {
                self.fail(
                    PlatformPulseTerminalError::ObservationPublication,
                    Err(error),
                );
                return false;
            }
        }
        true
    }

    fn finish_intent_transition(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        transition: UiIntentExecutionTransition,
    ) -> bool {
        let attempt = transition.attempt();
        let idempotency = transition.idempotency();
        match self.finish_terminal_intent_posture(shell, &transition, attempt, idempotency) {
            PlatformPulseIntentTransitionContinuation::Finished(finished) => return finished,
            PlatformPulseIntentTransitionContinuation::ContinueToConsequence => {}
        }
        if !matches!(
            transition.posture(),
            UiIntentExecutionTransitionPosture::Completed { .. }
        ) {
            return true;
        }
        let Some(kind) = consequence_kind(transition.posture()) else {
            self.fail_intent_settlement("completed intent carried an unknown outcome schema");
            return false;
        };
        let Some(consequence) = transition.into_consequence() else {
            self.fail_intent_settlement("completed transition omitted its consequence handle");
            return false;
        };
        let Some(target) = self
            .intent_evidence_index
            .target_for_execution(attempt, idempotency)
        else {
            self.fail_intent_settlement("completed intent omitted its retained feedback target");
            return false;
        };
        if let Err(denial) = self.product_story.prepare_completed_feedback(shell, target) {
            self.fail_intent_settlement(format!("completed feedback admission failed: {denial:?}"));
            return false;
        }
        self.presentation_tick = self.presentation_tick.saturating_add(1);

        let outcome = shell.begin_managed_native_intent_consequence_publication(
            consequence,
            self.presentation_tick,
        );

        match outcome {
            Ok(WorthUiNativeManagedIntentConsequencePublicationOutcome::Published(receipt)) => self
                .finish_intent_consequence_publication(shell, attempt, idempotency, kind, receipt),
            Ok(WorthUiNativeManagedIntentConsequencePublicationOutcome::Pending) => {
                self.pending_managed_rebind = Some(
                    super::super::PlatformPulsePendingManagedRebind::IntentConsequence(
                        PlatformPulsePendingIntentConsequence {
                            attempt,
                            idempotency,
                            kind,
                        },
                    ),
                );
                false
            }
            Ok(WorthUiNativeManagedIntentConsequencePublicationOutcome::NoConsequences(_)) => {
                self.fail_intent_settlement(
                    "completed Pulse action produced no declared consequences",
                );
                false
            }
            Ok(WorthUiNativeManagedIntentConsequencePublicationOutcome::Stopped(stop)) => {
                self.fail_intent_settlement(format!(
                    "intent consequence publication stopped: {stop:?}"
                ));
                false
            }
            Err(denial) => {
                self.fail_intent_settlement(format!(
                    "intent consequence managed admission stopped: {denial:?}"
                ));
                false
            }
        }
    }

    fn finish_terminal_intent_posture(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        transition: &UiIntentExecutionTransition,
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
    ) -> PlatformPulseIntentTransitionContinuation {
        match shell.prepare_native_intent_terminal_posture(transition) {
            WorthUiNativeIntentTerminalPostureOutcome::Prepared(posture) => {
                let observation = match transition.posture() {
                    UiIntentExecutionTransitionPosture::CancelledBeforeEffect { .. } => {
                        PlatformPulseIntentPostureObservation::cancelled(attempt, idempotency)
                    }
                    UiIntentExecutionTransitionPosture::RejectedBeforeEffect { .. }
                    | UiIntentExecutionTransitionPosture::FailedBeforeEffect { .. }
                    | UiIntentExecutionTransitionPosture::TimedOutBeforeEffect { .. } => {
                        PlatformPulseIntentPostureObservation::Denied
                    }
                    _ => {
                        self.fail_intent_settlement(
                            "terminal execution posture disagreed with its transition",
                        );
                        return PlatformPulseIntentTransitionContinuation::Finished(false);
                    }
                };
                let prepared = PlatformPulsePreparedIntentPosture::new(
                    posture,
                    observation,
                    PlatformPulseIntentPostureSettlement::retire_execution(attempt, idempotency),
                );
                let settled = matches!(
                    self.publish_native_intent_posture(shell, prepared),
                    PlatformPulseIntentPosturePublicationDisposition::Published
                );
                PlatformPulseIntentTransitionContinuation::Finished(settled)
            }
            WorthUiNativeIntentTerminalPostureOutcome::MissingExecutionBasis => {
                self.fail_intent_settlement("terminal execution omitted its mounted target basis");
                PlatformPulseIntentTransitionContinuation::Finished(false)
            }
            WorthUiNativeIntentTerminalPostureOutcome::PostureIdentityExhausted => {
                self.fail_intent_settlement("terminal execution posture identity exhausted");
                PlatformPulseIntentTransitionContinuation::Finished(false)
            }
            WorthUiNativeIntentTerminalPostureOutcome::RecoveryRetained
            | WorthUiNativeIntentTerminalPostureOutcome::NotTerminal => {
                PlatformPulseIntentTransitionContinuation::ContinueToConsequence
            }
        }
    }

    pub(in crate::native_application) fn settle_pending_intent_consequence(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        pending: PlatformPulsePendingIntentConsequence,
        receipt: worth_ui::facade::intent::UiIntentConsequencePublicationReceipt,
    ) {
        self.finish_intent_consequence_publication(
            shell,
            pending.attempt,
            pending.idempotency,
            pending.kind,
            receipt,
        );
    }
}

const fn is_local_progress(posture: UiIntentExecutionTransitionPosture) -> bool {
    !matches!(
        posture,
        UiIntentExecutionTransitionPosture::PendingBeforeEffect
            | UiIntentExecutionTransitionPosture::PendingEffectMayHaveBegun
    )
}
