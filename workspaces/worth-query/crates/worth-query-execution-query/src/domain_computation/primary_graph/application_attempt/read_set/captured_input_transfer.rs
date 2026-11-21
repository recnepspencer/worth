//! Consume completed capture custody without rebinding its original admission.
use super::*;
use crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry;
use crate::domain_computation::primary_graph::SourceReadAuthorizationRuntimeExt;
use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
use worth_query_execution_attempt::facade::ports::CompleteReadSetProjectedInternalPort as _;
use worth_query_execution_attempt::facade::ports::{
    CompleteReadSetInternalPort as _, EffectProgramInternalPort as _,
};
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

pub trait ReadSetCapturedInputTransferExt: Sized {
    fn append_captured_inputs(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        captured: Self,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>;
}

impl<S, O, I, E> ReadSetCapturedInputTransferExt
    for WorthQueryCompleteApplicationReadSet<S, O, I, E, WorthQueryProjectedApplicationMutation>
{
    /// The producer owner separately checks its exact typed input, edition and
    /// component before any numerical contact. This owner checks read authority
    /// and native support, then retains the old completed body as Source facts.
    fn append_captured_inputs(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        captured: Self,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let mismatch = || {
            denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                self.admission().operation(),
            )
        };
        self.admission()
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        captured
            .admission()
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let contract = self.admission().retain_graph_contract();
        if self.admission().runtime_authority() != captured.admission().runtime_authority()
            || self.admission().binding_identity() != captured.admission().binding_identity()
            || self.admission().operation_authority_identity_bytes()
                != captured.admission().operation_authority_identity_bytes()
            || self.admission().operation_definition_identity()
                != captured.admission().operation_definition_identity()
            || self.admission().scope_entity_id() != captured.admission().scope_entity_id()
            || self.admission().graph_work_principal_entity_id()
                != captured.admission().graph_work_principal_entity_id()
            || self.lease().product().observation() != captured.lease().product().observation()
            || !captured.installed_read_scopes().same_contract(&contract)
            || captured.computation_facts().is_some()
            || !captured.computation_fact_ordinals().is_empty()
            || captured.original_read().is_some()
        {
            return Err(mismatch());
        }
        let pin = self.lease().applied_projection().ok_or_else(mismatch)?;
        if !captured.captured_inputs_remain_equal(execution, pin)? {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::SourceChanged,
                self.admission().operation(),
            ));
        }
        let captured_parts = captured.into_parts();
        let permission = super::super::SourceReadAuthorization::consume_operation(
            execution, *captured_parts.admission, registry,
        ).map_err(|stop| WorthQueryApplicationAttemptDenial::output_observation(
            crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop(stop),
        ))?;
        // Existing chunks keep their original native observations. Neither keys,
        // admission IDs nor projection qualifications are rewritten for the write.
        let request = self.admission().publication_request();
        let permission = super::admit_completed_facts(
            1,
            [WorthQueryApplicationObservedFact::SourceReadAuthorization {
                observation: permission,
            }],
            policy,
            request,
            self.admission().operation(),
            || {
                self.admission()
                    .validate_current_authority()
                    .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
            },
        )?;
        let captured_facts = captured_parts
            .facts
            .append_completed(
                permission,
                super::super::retained_decision_facts::StorageControl::new(policy, Some(request)),
            )
            .map_err(|cause| cause.into_attempt_denial(self.admission().operation()))?;
        // A normal input cutoff replaces ONLY the producer-source suffix.
        // Capture data remains part of the handler input prefix across reuse.
        let boundary = self
            .admission()
            .completed_input_prefix()
            .ok_or_else(mismatch)?;
        let prefix = self
            .facts()
            .checked_slice(0..boundary)
            .ok_or_else(mismatch)?;
        let suffix = self
            .facts()
            .checked_slice(boundary..self.facts().len())
            .ok_or_else(mismatch)?;
        let count = captured_facts.len();
        boundary.checked_add(count).ok_or_else(mismatch)?;
        let control =
            super::super::retained_decision_facts::StorageControl::new(policy, Some(request));
        let joined = prefix
            .append_completed(captured_facts, control)
            .and_then(|facts| facts.append_completed(suffix, control))
            .map_err(|cause| cause.into_attempt_denial(self.admission().operation()))?;
        let routes = super::super::CompletedFactRoutes::from_facts(joined.clone(), control)
            .map_err(|cause| cause.into_attempt_denial(self.admission().operation()))?;
        self.admission()
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        if !self.admission_mut().insert_input_facts(boundary, count) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                self.admission().operation(),
            ));
        }
        self.replace_facts_and_routes(joined, routes);
        self.consumed_outputs_mut()
            .extend(captured_parts.consumed_outputs);
        Ok(())
    }
}
