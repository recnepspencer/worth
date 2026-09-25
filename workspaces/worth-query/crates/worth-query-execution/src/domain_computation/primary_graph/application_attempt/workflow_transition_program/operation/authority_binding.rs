use super::*;
use crate::domain_computation::primary_graph::application_attempt::read_set::{
    MutationHandlerBindingProof, WorkflowOperationBindingProof,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn bind_mutation_handler_input<Binding>(
        mut self,
        input: &Binding::Input,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
    {
        self.read_set.mutation_handler_binding = Some(MutationHandlerBindingProof {
            binding: Binding::IDENTITY,
            input_identity: Binding::input_identity(input),
        });
        self
    }

    #[doc(hidden)]
    pub fn bind_workflow_operation_authority(
        mut self,
        authority: &WorkflowOperationAuthority,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        if self.read_set.workflow_authority_binding.is_some()
            || self.read_set.admission.operation() != authority.operation
            || self.read_set.admission.runtime_authority().as_u64() != authority.runtime_authority
            || self.read_set.lease.product().product_branch()
                != authority.observation.product_branch()
        {
            return Err(mismatch("workflow operation authority"));
        }
        let current = self.read_set.lease.handle().with_runtime(|runtime| {
            authority
                .facts()
                .iter()
                .all(|fact| fact.remains_equal_in(runtime, self.read_set.lease.snapshot()))
        });
        if !current {
            return Err(mismatch("workflow operation authority"));
        }
        let mut locators = std::collections::BTreeMap::new();
        for (index, fact) in self.read_set.facts.iter().enumerate() {
            let locator = fact.locator_identity();
            if locators.insert(locator, index).is_some() {
                return Err(mismatch("workflow operation authority"));
            }
        }
        for fact in authority.facts() {
            let locator = fact.locator_identity();
            match locators.entry(locator) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(self.read_set.facts.len());
                    self.read_set.facts.push(fact.clone());
                }
                std::collections::btree_map::Entry::Occupied(entry)
                    if self.read_set.facts[*entry.get()] != *fact =>
                {
                    return Err(mismatch("workflow operation authority"));
                }
                std::collections::btree_map::Entry::Occupied(_) => {}
            }
        }
        if self.read_set.facts.len()
            > self
                .read_set
                .admission
                .allowed_graph_contract()
                .decision_fact_budget()
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                "workflow operation authority",
            ));
        }
        self.read_set.workflow_authority_binding = Some(WorkflowOperationBindingProof {
            binding: authority.binding.clone(),
            transition_identity: authority.transition_identity,
            input_identity: authority.input_identity,
            approval_authority: authority.approval_authority.clone(),
            settlement_basis: authority.settlement_basis.clone(),
            workflow_layout: authority.workflow_layout.clone(),
        });
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
