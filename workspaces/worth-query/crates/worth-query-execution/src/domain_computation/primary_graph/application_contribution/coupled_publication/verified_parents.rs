//! Verified parents the assembly retains, and the identities by which routes
//! carry the parent outputs their derivations consumed.
use super::*;
impl<S: ApplicationSchema> CoupledApplicationAssembly<'_, '_, '_, S> {
    /// A fresh identity for an output this assembly retains or derives.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn next_read(
        &self,
    ) -> crate::domain_computation::primary_graph::provider::ParentRead {
        let read = crate::domain_computation::primary_graph::provider::ParentRead(
            self.parent_reads.get(),
        );
        self.parent_reads.set(read.0 + 1);
        read
    }
    /// The route prepared next consumes these parent outputs.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn consume(
        &self,
        reads: Vec<crate::domain_computation::primary_graph::provider::ParentRead>,
    ) {
        *self.consuming.borrow_mut() = reads;
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn consuming(
        &self,
    ) -> Vec<crate::domain_computation::primary_graph::provider::ParentRead> {
        self.consuming.borrow().clone()
    }
    pub(in crate::domain_computation::primary_graph::application_contribution) fn retain_verified_parent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mut parent: super::super::producer::VerifiedAppliedParent,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
    ) -> Result<synchronous_outputs::AppliedOutputParent, CoupledSourceReadStop> {
        let read = self.next_read();
        parent.retained_as(read);
        let input = parent
            .publication()
            .map(|publication| candidate.current_output_input(execution, publication))
            .transpose()?;
        let applied = synchronous_outputs::AppliedOutputParent::new(
            parent.pin().clone(),
            parent.source(),
            input,
        )
        .identified(read);
        self.application
            .runtime
            .primary_provider
            .retain_candidate_read_support(
                execution,
                &self.root_session,
                parent.source(),
                self.applied_position(),
                crate::domain_computation::primary_graph::provider::CandidateReadSupport::Parent(
                    parent,
                ),
            )
            .map_err(execution::denial::candidate_support_denied)?;
        Ok(applied)
    }
}
