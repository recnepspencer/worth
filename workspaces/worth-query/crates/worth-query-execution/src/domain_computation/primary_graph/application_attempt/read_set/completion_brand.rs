//! Typed completion shell. No input, seal or operation identity is reconstructed.
use super::complete_read_set_core::PreparedReadCompletion;
use super::mutation_read_attempt_core::MutationReadAttemptCore;
use super::*;
use worth_execution::ExecutionAllocationPolicy;

impl<'request, 'authority, Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<'request, 'authority, Schema, Operation, Input, Scope, Phase>
{
    pub fn complete(
        mut self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompleteApplicationReadSet<Schema, Operation, Input, Scope, Phase>,
        WorthQueryApplicationAttemptDenial,
    > {
        self.check_completion_live()?;
        let native = self.native_completion.take();
        let core = self.completion_core();
        let prepared = match native {
            Some(refresh) => core.complete_native(refresh, allocation_policy)?,
            None => core.complete_full(allocation_policy)?,
        };
        let (completed, lease) = match prepared {
            PreparedReadCompletion::Full(completed) => (completed, self.lease),
            PreparedReadCompletion::Native(completed) => completed.finish(
                self.admission.read_completion_loan(),
                self.lease,
                allocation_policy,
            )?,
        };
        let admission_memory = self
            .execution
            .reserve_retained_memory(std::mem::size_of_val(&self.admission) as u64)
            .map_err(|cause| {
                super::super::retained_decision_facts::StoreDenial::ExecutionRequest(cause)
                    .into_attempt_denial(self.admission.operation())
            })?;
        Ok(WorthQueryCompleteApplicationReadSet {
            admission: Box::new(self.admission),
            _admission_memory: admission_memory,
            lease,
            application_projection: completed.read.application_projection,
            original_read: completed.read.original_read,
            installed_read_scopes: completed.read.installed_read_scopes,
            facts: completed.read.facts,
            content: completed.read.content,
            effect_routes: completed.effect_routes,
            postcondition_carry: completed.read.postcondition_carry,
            consumed_outputs: self.consumed_outputs,
            computation_facts: completed.read.computation_facts,
            computation_fact_ordinals: completed.read.computation_fact_ordinals,
            workflow_authority_binding: None,
            workflow_birth_binding: None,
            workflow_succession_binding: None,
            mutation_handler_binding: None,
            local_action_settlement: None,
            body_external_emission: None,
            signed_creation: None,
            workflow_deadline: None,
            new_commit_refusal: None,
            _phase: PhantomData,
        })
    }

    pub(super) fn check_completion_live(&self) -> Result<(), WorthQueryApplicationAttemptDenial> {
        super::mutation_read_attempt_core::check_completion_live(
            &self.execution,
            &self.value_decode,
            self.admission.reentry_parts().0,
        )
    }

    fn completion_core(&mut self) -> MutationReadAttemptCore<'_, 'request, 'authority> {
        MutationReadAttemptCore {
            execution: &self.execution,
            value_decode: &self.value_decode,
            admission: self.admission.read_completion_loan(),
            lease: &self.lease,
            expected_facts: &mut self.expected_facts,
            computation_reads: &mut self.computation_reads,
            installed_read_scopes: &mut self.installed_read_scopes,
            facts: &mut self.facts,
            source_facts: &mut self.source_facts,
        }
    }
}
