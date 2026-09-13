use worth_ui_platform_pulse::observation_contract::PlatformPulseQueryActionObservation;

use super::super::{
    PlatformPulseApplicationRuntime, PlatformPulsePendingQueryAction, PlatformPulseTerminalError,
};

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
            let query = self
                .query_lifecycle
                .as_mut()
                .expect("prepared Pulse retains its Query lifecycle");
            let status = "Deployed";
            let action = if query_denial_requested {
                query.execute_denied_action(status)
            } else {
                if !request.begin_effect() {
                    let _ = request.settle_indeterminate();
                    self.fail_intent_settlement(
                        "product request could not begin its declared effect phase",
                    );
                    return;
                }
                query.execute_current_action(status)
            };
            match action {
                Ok(crate::query_source::PlatformPulseQueryActionOutcome::Executed {
                    evidence,
                    observation,
                }) => self.complete_query_action(
                    request,
                    reference,
                    evidence_reference,
                    action_input_revision,
                    evidence,
                    observation,
                ),
                Ok(crate::query_source::PlatformPulseQueryActionOutcome::Denied {
                    denial,
                    active_query_source_revision,
                    submitted_query_source_revision,
                }) => {
                    let audience_denial = worth_ui_platform_pulse::observation_contract::
                        PlatformPulseQueryActionPreconditionDenial::from_projection(denial);
                    if self
                        .pending_query_denial_story
                        .replace(audience_denial)
                        .is_some()
                    {
                        let _ = request.fail_before_effect();
                        self.fail_intent_settlement(
                            "query-denial story already awaited its terminal posture",
                        );
                        return;
                    }
                    let report = PlatformPulseQueryActionObservation::denied(
                        reference,
                        action_input_revision,
                        denial,
                        active_query_source_revision,
                        submitted_query_source_revision,
                    );
                    if !request.reject_before_effect() {
                        self.pending_query_denial_story = None;
                        self.fail_intent_settlement("denied product request lost its receiver");
                        return;
                    }
                    if !self.publish_query_action(report) {
                        return;
                    }
                }
                Ok(crate::query_source::PlatformPulseQueryActionOutcome::Indeterminate {
                    detail,
                }) => {
                    let report = PlatformPulseQueryActionObservation::indeterminate(
                        reference,
                        action_input_revision,
                        detail,
                    );
                    if !request.settle_indeterminate() {
                        self.fail_intent_settlement(
                            "indeterminate product request lost its receiver",
                        );
                        return;
                    }
                    if !self.publish_query_action(report) {
                        return;
                    }
                }
                Err(denial) => {
                    let _ = request.fail_before_effect();
                    let observation = self.publisher.intent_preparation_failure();
                    self.fail(
                        PlatformPulseTerminalError::QueryLifecycle(denial),
                        observation,
                    );
                    return;
                }
            }
            if self.terminal_error.is_some() {
                return;
            }
        }
    }

    fn complete_query_action(
        &mut self,
        request: worth_ui_platform_pulse::intent::PlatformPulseActionPortRequest,
        reference: worth_ui_platform_pulse::intent::PlatformPulseActionAttemptReference,
        evidence_reference: worth_ui::facade::inspection::UiIntentEvidenceReference,
        action_input_revision: worth_ui_platform_pulse::intent::PlatformPulseActionInputRevision,
        evidence: worth_ui::facade::query_binding::WorthUiScalarProjectionActionEvidence,
        observation: worth_ui::facade::query_binding::UiProjectionObservation,
    ) {
        let report = PlatformPulseQueryActionObservation::executed(
            reference,
            action_input_revision,
            &evidence,
        );
        let projection = self.publisher.query_projection_issued(&observation);
        if !request.complete(observation) {
            self.fail_intent_settlement("completed product request lost its receiver");
            return;
        }
        let projection = match projection {
            Ok(projection) => projection,
            Err(error) => {
                self.fail(
                    PlatformPulseTerminalError::ObservationPublication,
                    Err(error),
                );
                return;
            }
        };
        self.pending_query_actions
            .push(PlatformPulsePendingQueryAction {
                reference,
                evidence_reference,
                projection,
            });
        self.publish_query_action(report);
    }

    fn publish_query_action(&mut self, observation: PlatformPulseQueryActionObservation) -> bool {
        match self.publisher.query_action(observation) {
            Ok(()) => true,
            Err(error) => {
                self.fail(
                    PlatformPulseTerminalError::ObservationPublication,
                    Err(error),
                );
                false
            }
        }
    }
}
