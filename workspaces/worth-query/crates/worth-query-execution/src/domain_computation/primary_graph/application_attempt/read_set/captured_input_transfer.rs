//! Consume completed capture custody without rebinding its original admission.
use super::*;
use crate::domain_computation::authorization::WorthQueryInstalledAuthorizationRegistry;

impl<S, O, I, E>
    WorthQueryCompleteApplicationReadSet<S, O, I, E, WorthQueryProjectedApplicationMutation>
{
    /// The producer owner separately checks its exact typed input, edition and
    /// component before any numerical contact. This owner checks read authority
    /// and native support, then retains the old completed body as Source facts.
    pub(in crate::domain_computation::primary_graph) fn append_captured_inputs(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        captured: Self,
        registry: &Arc<WorthQueryInstalledAuthorizationRegistry>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let mismatch = || {
            denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                self.admission.operation(),
            )
        };
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        captured
            .admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let contract = self.admission.retain_graph_contract();
        if self.admission.runtime_authority() != captured.admission.runtime_authority()
            || self.admission.binding_identity() != captured.admission.binding_identity()
            || self.admission.operation_authority_identity_bytes()
                != captured.admission.operation_authority_identity_bytes()
            || self.admission.operation_definition_identity()
                != captured.admission.operation_definition_identity()
            || self.admission.scope_entity_id() != captured.admission.scope_entity_id()
            || self.admission.graph_work_principal_entity_id()
                != captured.admission.graph_work_principal_entity_id()
            || self.lease.product().observation() != captured.lease.product().observation()
            || !captured.installed_read_scopes.same_contract(&contract)
            || captured.computation_facts.is_some()
            || !captured.computation_fact_ordinals.is_empty()
            || captured.original_read.is_some()
        {
            return Err(mismatch());
        }
        let pin = self.lease.applied_projection().ok_or_else(mismatch)?;
        if !captured.captured_inputs_remain_equal(execution, pin)? {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::SourceChanged,
                self.admission.operation(),
            ));
        }
        let permission = super::super::SourceReadAuthorization::consume_operation(
            execution, *captured.admission, registry,
        ).map_err(|stop| WorthQueryApplicationAttemptDenial::output_observation(
            crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop(stop),
        ))?;
        // Existing chunks keep their original native observations. Neither keys,
        // admission IDs nor projection qualifications are rewritten for the write.
        let request = self.admission.publication_request();
        let permission = super::admit_completed_facts(
            1,
            [WorthQueryApplicationObservedFact::SourceReadAuthorization {
                observation: permission,
            }],
            policy,
            request,
            self.admission.operation(),
            || {
                self.admission
                    .validate_current_authority()
                    .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
            },
        )?;
        let captured_facts = captured
            .facts
            .append_completed(
                permission,
                super::super::retained_decision_facts::StorageControl::new(policy, Some(request)),
            )
            .map_err(|cause| cause.into_attempt_denial(self.admission.operation()))?;
        self.append_completed_fact_payload(captured_facts, policy)?;
        self.consumed_outputs.extend(captured.consumed_outputs);
        Ok(())
    }
}
