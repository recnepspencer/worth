use worth_ui_platform_pulse::observation_contract::PlatformPulseQueryActionObservation;

use super::super::{
    PlatformPulseApplicationRuntime, PlatformPulsePendingQueryAction, PlatformPulseTerminalError,
};

pub(in crate::native_application) struct PlatformPulsePendingQueryRequest {
    request: worth_ui_platform_pulse::intent::PlatformPulseActionPortRequest,
    evidence_reference: worth_ui::facade::inspection::UiIntentEvidenceReference,
}
impl PlatformPulsePendingQueryRequest {
    pub(in crate::native_application) fn before_effect(&self) -> bool { self.request.query_denial_requested() }
    pub(in crate::native_application) fn cancellation_requested(&self) -> bool { self.request.cancellation_requested() }
}

mod settlement;

impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn poll_intent_action_port(&mut self) -> usize {
        let received_before = self
            .intent_action_owner
            .as_ref()
            .map_or(0, |owner| owner.census().received());
        self.poll_intent_action_port_requests();
        self.intent_action_owner.as_ref().map_or(0, |owner| {
            owner.census().received().saturating_sub(received_before)
        })
    }

    fn poll_intent_action_port_requests(&mut self) {
        loop {
            if self.pending_query_request.is_some() || self.pending_query_cancellation.is_some() || !self.query_lifecycle.as_ref().is_some_and(|query| query.accepts_command()) {
                return;
            }
            let request = self
                .intent_action_owner
                .as_ref()
                .and_then(|owner| owner.try_next());
            let Some(request) = request else {
                return;
            };
            let reference = request.reference();
            let Some(evidence_reference) =
                self.intent_evidence_index.reference_for_product(reference)
            else {
                let _ = request.fail_before_effect();
                self.fail_intent_settlement(
                    "product request omitted its owner-issued intent evidence reference",
                );
                return;
            };
            let action_input_revision = request.action_input_revision();
            let query_denial_requested = request.query_denial_requested();
            if request.cancellation_requested() {
                let report = PlatformPulseQueryActionObservation::cancelled_before_effect(
                    reference,
                    action_input_revision,
                );
                if !request.cancel_before_effect() {
                    self.fail_intent_settlement("cancelled product request lost its receiver");
                    return;
                }
                if !self.publish_query_action(report) {
                    return;
                }
                continue;
            }
            #[cfg(test)]
            if let Some(probe) = &self.query_probe { probe.borrow_mut().attempts += 1; }
            let query = self
                .query_lifecycle
                .as_mut()
                .expect("prepared Pulse retains its Query lifecycle");
            let status = "Deployed";
            let action = if query_denial_requested {
                query.execute_denied_action(
                    status,
                    reference.idempotency_session(),
                    reference.idempotency_lineage(),
                )
            } else {
                if !request.begin_effect() {
                    let _ = request.settle_indeterminate();
                    self.fail_intent_settlement(
                        "product request could not begin its declared effect phase",
                    );
                    return;
                }
                query.execute_current_action(
                    status,
                    reference.idempotency_session(),
                    reference.idempotency_lineage(),
                )
            };
            if !self.settle_query_action(PlatformPulsePendingQueryRequest { request, evidence_reference }, action) {
                return;
            }
            if self.terminal.is_stopped() {
                return;
            }
        }
    }

}
