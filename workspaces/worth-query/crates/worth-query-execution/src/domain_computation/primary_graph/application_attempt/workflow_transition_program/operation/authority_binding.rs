use super::*;
use crate::domain_computation::primary_graph::application_attempt::read_set::{
    MutationHandlerBindingProof, WorkflowOperationBindingProof,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn bind_mutation_handler_input<Binding>(
        mut self,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
    {
        self.read_set.mutation_handler_binding = Some(MutationHandlerBindingProof {
            binding: Binding::IDENTITY,
            input_identity: *identities.input_identity(),
        });
        self
    }

    #[doc(hidden)]
    pub fn bind_workflow_operation_authority(
        mut self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        authority: &WorkflowOperationAuthority,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let phase_request = match phase.request_for_source(self.read_set.lease.handle().relational_runtime_instance_id()) {
            Ok(request) => request,
            Err(_) => return Err(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "application-read-phase")),
        };
        let execution = &phase_request;

        if self.read_set.workflow_authority_binding.is_some()
            || self.read_set.workflow_birth_binding.is_some()
            || self.read_set.workflow_succession_binding.is_some()
            || self.read_set.admission.operation() != authority.operation
            || self.read_set.admission.runtime_authority().as_u64() != authority.runtime_authority
            || self.read_set.lease.product().product_branch()
                != authority.observation.product_branch()
        {
            return Err(mismatch("workflow operation authority"));
        }
        let authority_fact_start = self.read_set.facts.len();
        self.read_set
            .append_authority_facts(execution, authority.facts(), allocation_policy)?;
        self.read_set.workflow_authority_binding = Some(WorkflowOperationBindingProof {
            authority_fact_range: authority_fact_start..self.read_set.facts.len(),
            captured_input: authority.captured_input.clone(),
            binding: authority.binding.clone(),
            transition_identity: authority.transition_identity,
            input_identity: authority.input_identity,
            approval_authority: authority.approval_authority.clone(),
            workflow_execution_requirements: authority.workflow_execution_requirements.clone(),
            settlement_basis: authority.settlement_basis.clone(),
            workflow_layout: authority.workflow_layout.clone(),
        });
        self.read_set.workflow_deadline = authority.deadline;
        Ok(self)
    }

    pub(in crate::domain_computation::primary_graph) fn matches_workflow_authority_binding<
        Binding,
    >(
        &self,
        idempotency: &crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> bool
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
    {
        match &self.read_set.workflow_authority_binding {
            None => !Binding::REQUIRES_WORKFLOW_AUTHORITY,
            Some(proof) => {
                Binding::REQUIRES_WORKFLOW_AUTHORITY
                    && proof.captured_input.as_ref().is_none_or(|expected| {
                        expected.resource == self.read_set.admission.scope_entity_id()
                            && expected.source.map(|source| source.0)
                                == self
                                    .read_set
                                    .admission
                                    .source_expectation_kind()
                                    .map(|kind| kind.code())
                            && expected.preconditions.as_ref()
                                == self.read_set.admission.mutation_preconditions().identity()
                            && idempotency.matches_captured_workflow_contract(expected)
                    })
                    && proof.workflow_execution_requirements
                        == Binding::workflow_execution_requirements()
                    && proof.binding == Binding::IDENTITY
                    && self
                        .read_set
                        .mutation_handler_binding
                        .as_ref()
                        .is_some_and(|handler| {
                            handler.binding == Binding::IDENTITY
                                && handler.input_identity == proof.input_identity
                        })
                    && idempotency.matches_guarded_workflow_effect(&proof.transition_identity)
                    && idempotency.intent_identity() == &proof.input_identity
            }
        }
    }
}
