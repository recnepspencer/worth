use super::*;
use crate::intent_admission::{
    WorthQueryAuthoritativeIntentExecutionHandoff, WorthQueryEffectTriggeredIntentExecutionHandoff,
    WorthQueryIntentDecisionTraceEnvelope, WorthQueryIntentViolationDecision,
    WorthQueryRawIntentAdmissionRequest, WorthQueryRuntimeEffectWriteIntentAuthoring,
    WorthQueryRuntimeIntentAuthoring,
};

impl WorthQueryRuntime {
    pub fn intent(
        &mut self,
        declaration: WorthQueryIntentDeclaration,
    ) -> WorthQueryRuntimeIntentAuthoring<'_> {
        WorthQueryRuntimeIntentAuthoring::new(self, declaration)
    }

    pub fn next_effect_write_intent<T>(
        &mut self,
        effect: &WorthQueryEffectHandle<T>,
        strategy_version: impl Into<String>,
        input_contract: impl Into<String>,
    ) -> WorthQueryRuntimeEffectWriteIntentAuthoring<'_> {
        WorthQueryRuntimeEffectWriteIntentAuthoring::new(
            self,
            effect,
            strategy_version.into(),
            input_contract.into(),
        )
    }

    pub fn execute_intent(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentReceipt, WorthQueryRuntimeError> {
        let handoff = self.admit_authoritative_intent_for_execution(declaration)?;
        let binding = self.prepare_authoritative_intent_execution_binding(handoff);
        self.execute_authoritative_intent_execution_binding(execution, binding)
    }

    pub fn execute_next_effect_write_intent<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        effect: &WorthQueryEffectHandle<T>,
        strategy_version: impl Into<String>,
        input_contract: impl Into<String>,
    ) -> Result<WorthQueryEffectIntentReceipt, WorthQueryRuntimeError> {
        let (pending_delivery, handoff) = self.admit_next_effect_write_intent_for_execution(
            effect.name(),
            strategy_version.into(),
            input_contract.into(),
        )?;
        let binding = self.prepare_effect_intent_execution_binding(handoff, &pending_delivery);
        self.execute_effect_intent_execution_binding(execution, binding)
    }

    pub fn admit_authoritative_intent_for_execution(
        &self,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryAuthoritativeIntentExecutionHandoff, WorthQueryRuntimeError> {
        let review = self.review_authoritative_runtime_intent(declaration)?;
        self.resolve_reviewed_admitted_authoritative_intent_handoff(review)
    }

    pub fn admit_next_effect_write_intent_for_execution(
        &self,
        effect_name: &str,
        strategy_version: impl Into<String>,
        input_contract: impl Into<String>,
    ) -> Result<
        (
            WorthQueryEffectDelivery,
            WorthQueryEffectTriggeredIntentExecutionHandoff,
        ),
        WorthQueryRuntimeError,
    > {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Effect)?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Intent)?;
        let effect_target = WorthQueryEffectTarget::from_name(effect_name);
        let pending_delivery = self.pending_effect_write_delivery(&effect_target)?.1;
        let request = self.effect_runtime_intent_request(
            &pending_delivery,
            strategy_version.into(),
            input_contract.into(),
        )?;
        let review =
            crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData::from_request(
                request,
            );
        let handoff = self.resolve_reviewed_admitted_effect_intent_handoff(review)?;
        Ok((pending_delivery, handoff))
    }

    pub(crate) fn review_authoritative_runtime_intent(
        &self,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<
        crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
        WorthQueryRuntimeError,
    > {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Intent)?;
        let request = self.authoritative_runtime_intent_request(declaration)?;
        Ok(
            crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData::from_request(
                request,
            ),
        )
    }

    pub(crate) fn review_next_effect_write_runtime_intent(
        &self,
        effect_name: &str,
        strategy_version: String,
        input_contract: String,
    ) -> Result<
        (
            WorthQueryEffectDelivery,
            crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData,
        ),
        WorthQueryRuntimeError,
    > {
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Effect)?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Intent)?;
        let effect_target = WorthQueryEffectTarget::from_name(effect_name);
        let (_, pending_delivery) = self.pending_effect_write_delivery(&effect_target)?;
        let request = self.effect_runtime_intent_request(
            &pending_delivery,
            strategy_version,
            input_contract,
        )?;
        Ok((
            pending_delivery,
            crate::intent_admission::dx::WorthQueryRuntimeIntentAdmissionReviewData::from_request(
                request,
            ),
        ))
    }

    fn authoritative_runtime_intent_request(
        &self,
        declaration: WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryRawIntentAdmissionRequest, WorthQueryRuntimeError> {
        let declaration_for_error = declaration.clone();
        WorthQueryRawIntentAdmissionRequest::authoritative_runtime_entrypoint(declaration).map_err(
            |violation| {
                self.intent_violation_error(&declaration_for_error, violation, None, None, None)
            },
        )
    }

    fn effect_runtime_intent_request(
        &self,
        delivery: &WorthQueryEffectDelivery,
        strategy_version: String,
        input_contract: String,
    ) -> Result<WorthQueryRawIntentAdmissionRequest, WorthQueryRuntimeError> {
        let declaration = WorthQueryIntentDeclaration::strategy_commit(
            format!("effect:{}", delivery.effect_name()),
            delivery.target().to_string(),
            strategy_version,
            input_contract,
            WorthQueryIntentInput::from_effect_payload(delivery.payload()),
        )
        .with_source_lane(WorthQueryIntentSourceLane::EffectTriggered)
        .with_effect_trigger(delivery.write_adjacent_trigger().clone());
        let declaration_for_error = declaration.clone();
        WorthQueryRawIntentAdmissionRequest::effect_runtime_entrypoint(declaration).map_err(
            |violation| {
                self.intent_violation_error(&declaration_for_error, violation, None, None, None)
            },
        )
    }

    pub(crate) fn intent_violation_error(
        &self,
        declaration: &WorthQueryIntentDeclaration,
        violation: WorthQueryIntentViolationDecision,
        execution: Option<&WorthQueryIntentExecution>,
        decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
        execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
    ) -> WorthQueryRuntimeError {
        let denial =
            intent::WorthQueryIntentAdmissionDenial::new(violation.stage(), violation.message());
        let evidence = WorthQueryIntentDenialEvidence::new_with_trace(
            declaration,
            &denial,
            execution,
            execution_provenance,
            decision_trace_envelope,
        );
        WorthQueryRuntimeError::IntentCommitDenied {
            intent_name: declaration.name().to_string(),
            stage: violation.stage(),
            message: violation.message().to_string(),
            evidence,
        }
    }

    pub(crate) fn intent_execution_routing_error(
        &self,
        declaration: &WorthQueryIntentDeclaration,
        execution: &WorthQueryIntentExecution,
        execution_provenance: WorthQueryIntentExecutionProvenance,
        decision_trace_envelope: WorthQueryIntentDecisionTraceEnvelope,
        source: WorthQueryRuntimeError,
    ) -> WorthQueryRuntimeError {
        let stage = "post-execution-routing";
        let message = source.to_string();
        let evidence = intent::WorthQueryIntentExecutionFailureEvidence::new(
            declaration,
            stage,
            message.clone(),
            execution,
            execution_provenance,
            decision_trace_envelope,
        );
        WorthQueryRuntimeError::IntentExecutionRoutingFailed {
            intent_name: declaration.name().to_string(),
            stage,
            message,
            evidence,
            source: Box::new(source),
        }
    }

    pub(crate) fn admitted_handoff_violation_error(
        &self,
        handoff: &WorthQueryAdmittedIntentExecutionHandoff,
        stage: &'static str,
        message: impl Into<String>,
    ) -> WorthQueryRuntimeError {
        let violation = WorthQueryIntentViolationDecision::new(
            handoff.family(),
            handoff.entrypoint(),
            stage,
            message,
            handoff.request_digest(),
            handoff.eligibility_digest(),
        );
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_handoff_violation(handoff, &violation);
        self.intent_violation_error(
            handoff.declaration(),
            violation,
            None,
            Some(decision_trace_envelope),
            None,
        )
    }

    pub(in crate::runtime) fn pending_effect_write_delivery(
        &self,
        effect_target: &WorthQueryEffectTarget,
    ) -> Result<(usize, WorthQueryEffectDelivery), WorthQueryRuntimeError> {
        let runtime = self.effects.get(effect_target).ok_or_else(|| {
            WorthQueryRuntimeError::MissingEffect(effect_target.as_str().to_string())
        })?;
        runtime
            .deliveries
            .iter()
            .enumerate()
            .find(|(_, delivery)| {
                delivery.family() == &WorthQueryEffectDeliveryFamily::PendingWriteIntent
            })
            .map(|(index, delivery)| (index, delivery.clone()))
            .ok_or_else(|| {
                WorthQueryRuntimeError::MissingPendingWriteIntent(
                    effect_target.as_str().to_string(),
                )
            })
    }

    pub(in crate::runtime) fn remove_pending_effect_delivery(
        &mut self,
        effect_target: &WorthQueryEffectTarget,
        pending_index: usize,
        pending_delivery: &WorthQueryEffectDelivery,
    ) {
        if let Some(runtime) = self.effects.get_mut(effect_target) {
            if runtime
                .deliveries
                .get(pending_index)
                .is_some_and(|delivery| {
                    delivery.family() == &WorthQueryEffectDeliveryFamily::PendingWriteIntent
                        && delivery.effect_name() == pending_delivery.effect_name()
                        && delivery
                            .commit_identity()
                            .is_same_current_identity_as(pending_delivery.commit_identity())
                })
            {
                runtime.deliveries.remove(pending_index);
            } else if let Some(index) = runtime.deliveries.iter().position(|delivery| {
                delivery.family() == &WorthQueryEffectDeliveryFamily::PendingWriteIntent
                    && delivery.effect_name() == pending_delivery.effect_name()
                    && delivery
                        .commit_identity()
                        .is_same_current_identity_as(pending_delivery.commit_identity())
            }) {
                runtime.deliveries.remove(index);
            }
        }
    }
}
