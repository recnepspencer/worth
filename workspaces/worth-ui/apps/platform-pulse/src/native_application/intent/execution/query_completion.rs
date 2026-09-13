use worth_ui::facade::app::WorthUiNativeApplicationShell;
use worth_ui_platform_pulse::observation_contract::PlatformPulseIntentPostureObservation;

use super::super::super::{
    PlatformPulseApplicationRuntime, PlatformPulsePendingQueryAction, PlatformPulseTerminalError,
};

impl PlatformPulseApplicationRuntime {
    pub(super) fn finish_action_query_publication(
        &mut self,
        shell: &mut WorthUiNativeApplicationShell,
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
        receipt: worth_ui::facade::rebind::UiRebindReceipt,
    ) -> bool {
        let Some(pending) = self.take_completed_query_action(attempt, idempotency) else {
            return false;
        };
        let Some(trace) = self.resolve_completed_trace(shell, &pending, attempt, idempotency)
        else {
            return false;
        };
        let Some(mounted) = receipt.mounted_publication() else {
            self.fail_intent_settlement("intent consequence receipt omitted mounted publication");
            return false;
        };
        if !self.publish_query_projection_evidence(&pending.projection, mounted) {
            return false;
        }
        let posture = PlatformPulseIntentPostureObservation::completed(attempt, idempotency);
        let command_transition = super::super::latest_command_transition(shell);
        if let Err(error) =
            self.publisher
                .intent_posture_published(posture, mounted, command_transition)
        {
            self.fail(
                PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return false;
        }
        if !self.publish_completed_trace(trace, &pending.projection, mounted) {
            return false;
        }
        if let Err(denial) = self.refresh_query_visual_identity(shell, &pending.projection) {
            self.fail_visual_identity(denial);
            return false;
        }
        self.admit_query_predecessor(receipt)
    }

    fn take_completed_query_action(
        &mut self,
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
    ) -> Option<PlatformPulsePendingQueryAction> {
        let index = self
            .pending_query_actions
            .iter()
            .position(|pending| pending.reference.matches_execution(attempt, idempotency));
        let Some(index) = index else {
            self.fail_intent_settlement("completed attempt has no product-issued Query evidence");
            return None;
        };
        Some(self.pending_query_actions.remove(index))
    }

    fn resolve_completed_trace(
        &mut self,
        shell: &WorthUiNativeApplicationShell,
        pending: &PlatformPulsePendingQueryAction,
        attempt: worth_ui::facade::intent::UiIntentExecutionAttemptIdentity,
        idempotency: worth_ui::facade::intent::UiIntentExecutionIdempotencyIdentity,
    ) -> Option<worth_ui::facade::inspection::UiIntentCausalTraceEvidence> {
        let trace = match shell.lookup_intent_causal_trace(pending.evidence_reference) {
            worth_ui::facade::inspection::UiIntentEvidenceLookup::Found(trace)
                if trace.is_complete_through_product_outcome() =>
            {
                trace
            }
            worth_ui::facade::inspection::UiIntentEvidenceLookup::Found(_) => {
                self.fail_intent_settlement(
                    "completed product action resolved an incomplete intent causal trace",
                );
                return None;
            }
            posture => {
                self.fail_intent_settlement(format!(
                    "completed product action lost its intent causal trace: {posture:?}"
                ));
                return None;
            }
        };
        if self
            .intent_evidence_index
            .retire_execution(attempt, idempotency)
            != Some(trace.reference())
        {
            self.fail_intent_settlement(
                "completed product action substituted its retained intent evidence reference",
            );
            return None;
        }
        Some(trace)
    }

    fn publish_completed_trace(
        &mut self,
        trace: worth_ui::facade::inspection::UiIntentCausalTraceEvidence,
        projection: &worth_ui_platform_pulse::observation_contract::PlatformPulseQueryProjectionEvidence,
        mounted: &worth_ui::facade::app::UiMountedFramePublicationReceipt,
    ) -> bool {
        let causal_trace = match worth_ui_platform_pulse::observation_contract::
            PlatformPulseIntentCausalTraceObservation::from_completed_publication(
                trace, projection, mounted,
            ) {
            Ok(trace) => trace,
            Err(denial) => {
                self.fail_intent_settlement(format!(
                    "completed intent causal trace projection stopped: {denial:?}"
                ));
                return false;
            }
        };
        if let Err(error) = self.publisher.intent_causal_trace(causal_trace) {
            self.fail(
                PlatformPulseTerminalError::ObservationPublication,
                Err(error),
            );
            return false;
        }
        true
    }

    fn admit_query_predecessor(
        &mut self,
        receipt: worth_ui::facade::rebind::UiRebindReceipt,
    ) -> bool {
        let observation = match receipt.release_scalar_projection_observation() {
            Ok(observation) => observation,
            Err(_) => {
                self.fail_intent_settlement(
                    "intent consequence receipt omitted scalar Query predecessor",
                );
                return false;
            }
        };
        let admission = self
            .query_lifecycle
            .as_mut()
            .expect("prepared Pulse retains its Query lifecycle")
            .admit_publication(observation);
        if let Err(denial) = admission {
            self.fail(
                PlatformPulseTerminalError::QueryLifecycle(denial),
                self.publisher.intent_preparation_failure(),
            );
            return false;
        }
        true
    }
}
