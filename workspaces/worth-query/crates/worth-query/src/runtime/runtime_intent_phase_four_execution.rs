use super::*;
use crate::intent_admission::{
    admit_authoritative_execution, admit_effect_execution, intent_runtime_facade_family,
    WorthQueryAdmittedIntentExecutionHandoff, WorthQueryAuthoritativeIntentExecutionBinding,
    WorthQueryAuthoritativeIntentExecutionHandoff, WorthQueryEffectTriggeredIntentExecutionBinding,
    WorthQueryEffectTriggeredIntentExecutionHandoff,
};

impl WorthQueryRuntime {
    pub(crate) fn prepare_authoritative_intent_execution_binding(
        &self,
        handoff: WorthQueryAuthoritativeIntentExecutionHandoff,
    ) -> WorthQueryAuthoritativeIntentExecutionBinding {
        WorthQueryAuthoritativeIntentExecutionBinding::from_handoff(handoff)
    }

    pub(crate) fn prepare_effect_intent_execution_binding(
        &self,
        handoff: WorthQueryEffectTriggeredIntentExecutionHandoff,
        pending_delivery: &WorthQueryEffectDelivery,
    ) -> WorthQueryEffectTriggeredIntentExecutionBinding {
        WorthQueryEffectTriggeredIntentExecutionBinding::from_handoff_and_delivery(
            handoff,
            pending_delivery,
        )
    }

    pub(crate) fn execute_authoritative_intent_execution_binding(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryAuthoritativeIntentExecutionBinding,
    ) -> Result<WorthQueryIntentReceipt, WorthQueryRuntimeError> {
        self.admit_facade_family(intent_runtime_facade_family(
            binding.declaration().source_lane(),
        ))?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Intent)?;
        let declaration = binding.declaration().clone();
        let execution = self
            .backend
            .execute_intent(request_execution, &declaration)?
            .admit_runtime_authority();
        admit_authoritative_execution(binding.handoff(), &execution).map_err(|violation| {
            let handoff = WorthQueryAdmittedIntentExecutionHandoff::from(binding.handoff().clone());
            let decision_trace_envelope =
                WorthQueryIntentDecisionTraceEnvelope::for_execution_violation(
                    &handoff, &execution, &violation,
                );
            let execution_provenance = authoritative_execution_provenance(&binding, &execution);
            self.intent_violation_error(
                &declaration,
                violation,
                Some(&execution),
                Some(decision_trace_envelope),
                Some(execution_provenance),
            )
        })?;
        let handoff = WorthQueryAdmittedIntentExecutionHandoff::from(binding.handoff().clone());
        let execution_provenance = authoritative_execution_provenance(&binding, &execution);
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution(&handoff, &execution);
        let write_receipt = self.route_admitted_intent_execution(
            request_execution,
            &declaration,
            &execution,
            &execution_provenance,
            &decision_trace_envelope,
        )?;
        Ok(WorthQueryIntentReceipt::from_authoritative_binding(
            &binding,
            &declaration,
            execution,
            &write_receipt,
        ))
    }

    pub(crate) fn execute_effect_intent_execution_binding(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: WorthQueryEffectTriggeredIntentExecutionBinding,
    ) -> Result<WorthQueryEffectIntentReceipt, WorthQueryRuntimeError> {
        self.admit_facade_family(intent_runtime_facade_family(
            binding.declaration().source_lane(),
        ))?;
        self.admit_facade_family(WorthQueryRuntimeFacadeFamily::Intent)?;
        let handoff = WorthQueryAdmittedIntentExecutionHandoff::from(binding.handoff().clone());
        let (pending_index, pending_delivery) = self
            .pending_effect_write_delivery_for_binding(&binding)
            .map_err(|error| {
                self.admitted_handoff_violation_error(
                    &handoff,
                    "pending-write-intent-binding",
                    error.to_string(),
                )
            })?;
        let declaration = binding.declaration().clone();
        let execution = self
            .backend
            .execute_intent(request_execution, &declaration)?
            .admit_runtime_authority();
        admit_effect_execution(binding.handoff(), &execution).map_err(|violation| {
            let decision_trace_envelope =
                WorthQueryIntentDecisionTraceEnvelope::for_execution_violation(
                    &handoff, &execution, &violation,
                );
            let execution_provenance = effect_execution_provenance(&binding, &execution);
            self.intent_violation_error(
                &declaration,
                violation,
                Some(&execution),
                Some(decision_trace_envelope),
                Some(execution_provenance),
            )
        })?;
        let execution_provenance = effect_execution_provenance(&binding, &execution);
        let decision_trace_envelope =
            WorthQueryIntentDecisionTraceEnvelope::for_admitted_execution(&handoff, &execution);
        let write_receipt = self.route_admitted_intent_execution(
            request_execution,
            &declaration,
            &execution,
            &execution_provenance,
            &decision_trace_envelope,
        )?;
        let intent_receipt = WorthQueryIntentReceipt::from_effect_binding(
            &binding,
            &declaration,
            execution,
            &write_receipt,
        );
        let effect_target = WorthQueryEffectTarget::from_name(pending_delivery.effect_name());
        self.remove_pending_effect_delivery(&effect_target, pending_index, &pending_delivery);
        Ok(WorthQueryEffectIntentReceipt::new(
            &pending_delivery,
            intent_receipt,
        ))
    }

    fn pending_effect_write_delivery_for_binding(
        &self,
        binding: &WorthQueryEffectTriggeredIntentExecutionBinding,
    ) -> Result<(usize, WorthQueryEffectDelivery), WorthQueryRuntimeError> {
        let effect_target = WorthQueryEffectTarget::from_name(binding.effect_name());
        let runtime = self.effects.get(&effect_target).ok_or_else(|| {
            WorthQueryRuntimeError::MissingEffect(binding.effect_name().to_string())
        })?;
        runtime
            .deliveries
            .iter()
            .enumerate()
            .find(|(_, delivery)| {
                delivery.family() == &WorthQueryEffectDeliveryFamily::PendingWriteIntent
                    && binding.matches_pending_delivery(delivery)
            })
            .map(|(index, delivery)| (index, delivery.clone()))
            .ok_or_else(|| {
                WorthQueryRuntimeError::MissingPendingWriteIntent(binding.effect_name().to_string())
            })
    }

    fn route_admitted_intent_execution(
        &mut self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: &WorthQueryIntentDeclaration,
        execution: &WorthQueryIntentExecution,
        provenance: &WorthQueryIntentExecutionProvenance,
        decision_trace: &WorthQueryIntentDecisionTraceEnvelope,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let summary = classify_receipt_mutation_summary(execution.mutation_receipt());
        self.route_authoritative_mutation_receipt(
            request_execution,
            WorthQueryAuthoritativeMutationRoutingInput::from_intent_execution(
                execution.mutation_receipt().clone(),
                summary,
                decision_trace.clone(),
                provenance.clone(),
            ),
        )
        .map_err(|error| {
            self.intent_execution_routing_error(
                declaration,
                execution,
                provenance.clone(),
                decision_trace.clone(),
                error,
            )
        })
    }
}

fn authoritative_execution_provenance(
    binding: &WorthQueryAuthoritativeIntentExecutionBinding,
    execution: &WorthQueryIntentExecution,
) -> WorthQueryIntentExecutionProvenance {
    let snapshot_identity = execution
        .mutation_receipt()
        .snapshot_identity
        .evidence_identity();
    WorthQueryIntentExecutionProvenance::for_shared_execution_typed_parts(
        binding.family(),
        binding.entrypoint(),
        binding.execution_seam(),
        binding.handoff().decision_digest(),
        binding.handoff().handoff_digest(),
        binding.binding_digest(),
        execution.outcome_digest(),
        &snapshot_identity,
    )
}

fn effect_execution_provenance(
    binding: &WorthQueryEffectTriggeredIntentExecutionBinding,
    execution: &WorthQueryIntentExecution,
) -> WorthQueryIntentExecutionProvenance {
    let snapshot_identity = execution
        .mutation_receipt()
        .snapshot_identity
        .evidence_identity();
    WorthQueryIntentExecutionProvenance::for_shared_execution_typed_parts(
        binding.family(),
        binding.entrypoint(),
        binding.execution_seam(),
        binding.handoff().decision_digest(),
        binding.handoff().handoff_digest(),
        binding.binding_digest(),
        execution.outcome_digest(),
        &snapshot_identity,
    )
}
