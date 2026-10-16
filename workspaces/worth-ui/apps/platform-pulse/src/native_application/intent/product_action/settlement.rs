//! One settlement owner consumes the original Intent request only on a final Query outcome.
use super::*;
impl PlatformPulseApplicationRuntime {
    pub(in crate::native_application) fn cancel_retained_query_request(&mut self) -> bool {
        let cancelled = self
            .query_lifecycle
            .as_mut()
            .expect("original Query owner retained")
            .cancel_pending_mutation();
        if cancelled.is_err() {
            return false;
        }
        let original = self
            .pending_query_request
            .take()
            .expect("original before-effect request retained");
        let reference = original.request.reference();
        let report = PlatformPulseQueryActionObservation::cancelled_before_effect(
            reference,
            original.request.action_input_revision(),
        );
        if !original.request.cancel_before_effect() {
            self.fail_intent_settlement(
                "Query no-commit cancellation lost its before-effect receiver",
            );
            return false;
        }
        self.pending_query_cancellation = Some(reference);
        self.publish_query_action(report)
    }

    pub(in crate::native_application) fn settle_query_action(
        &mut self,
        context: PlatformPulsePendingQueryRequest,
        action: Result<
            crate::query_source::PlatformPulseQueryActionOutcome,
            crate::query_source::PlatformPulseQueryLifecycleDenial,
        >,
    ) -> bool {
        let PlatformPulsePendingQueryRequest {
            request,
            evidence_reference,
        } = context;
        let reference = request.reference();
        let action_input_revision = request.action_input_revision();
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
                    return false;
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
                    return false;
                }
                if !self.publish_query_action(report) {
                    return false;
                }
            }
            Ok(crate::query_source::PlatformPulseQueryActionOutcome::Pending) => {
                self.pending_query_request = Some(PlatformPulsePendingQueryRequest {
                    request,
                    evidence_reference,
                });
                #[cfg(test)]
                if let Some(probe) = self.query_probe.clone() {
                    assert!(self
                        .query_lifecycle
                        .as_ref()
                        .unwrap()
                        .authentication_cancelled_for_test());
                    {
                        let mut recording = probe.borrow_mut();
                        recording.cancelled_stops += 1;
                        recording.stopped_request = Some(reference);
                        recording.pending_census = self
                            .intent_action_owner
                            .as_ref()
                            .map(|owner| owner.census());
                    }
                    let refused = self.shutdown_product().is_none()
                        && self.shell.is_some()
                        && self.query_lifecycle.is_some();
                    probe.borrow_mut().close_refused_before_presentation = refused;
                    worth_ui_native_platform::UiNativeApplicationRuntime::external_close_requested(
                        self,
                    );
                    probe.borrow_mut().close_waited_for_presentation =
                        !worth_ui_native_platform::UiNativeApplicationRuntime::external_close_ready(
                            self,
                        );
                }
                return false;
            }
            Ok(crate::query_source::PlatformPulseQueryActionOutcome::Indeterminate(cause)) => {
                let report = PlatformPulseQueryActionObservation::indeterminate(
                    reference,
                    action_input_revision,
                    cause.to_string(),
                );
                if !request.settle_indeterminate() {
                    self.fail_intent_settlement("indeterminate product request lost its receiver");
                    return false;
                }
                if !self.publish_query_action(report) {
                    return false;
                }
            }
            Err(denial) => {
                let _ = request.fail_before_effect();
                let observation = self.publisher.intent_preparation_failure();
                self.fail(
                    PlatformPulseTerminalError::QueryLifecycle(denial),
                    observation,
                );
                return false;
            }
        }
        true
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
        #[cfg(test)]
        if let Some(probe) = &self.query_probe
            && let Ok(evidence) = &projection
        {
            let worth_ui::facade::query_binding::UiProjectionObservation::ApplicationScalar(scalar) =
                &observation
            else {
                panic!("genuine Query consequence required");
            };
            let mut recording = probe.borrow_mut();
            recording.issued.push(evidence.clone());
            recording
                .basis_versions
                .push(scalar.fact().query_receipt().inspect().basis().version());
            recording.revisions.push(scalar.fact().value().revision);
        }
        if !request.complete(observation) {
            self.fail_intent_settlement("completed product request lost its receiver");
            return;
        }
        #[cfg(test)]
        if let Some(probe) = &self.query_probe {
            let mut recording = probe.borrow_mut();
            recording.completed_request = Some(reference);
            recording.completed_census = self
                .intent_action_owner
                .as_ref()
                .map(|owner| owner.census());
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

    pub(super) fn publish_query_action(
        &mut self,
        observation: PlatformPulseQueryActionObservation,
    ) -> bool {
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
