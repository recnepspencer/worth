//! Recovery reads its exact original through the owning source, then checks every binding axis.
use super::*;
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn committed_dispatch_outbox_for_binding(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleBinding,
        observation: &worth_runtime_world::facade::RuntimeWorldObservationPort,
        inspection: &worth_runtime_world::facade::RuntimeWorldInspectionPort,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        let committed = binding.committed_dispatch_outbox().ok_or(Denial::Missing)?;
        if binding
            .committed_product_publication()
            .composite_commit()
            .owner_identity()
            .durable()
            .is_some()
        {
            let original = self.committed_dispatch_outbox_for_correlation(
                execution,
                committed.record().correlation(),
                observation,
                inspection,
            )?;
            if original.relational_runtime_instance_id() != binding.runtime_instance_id() {
                return Err(Denial::ForeignRuntime);
            }
            if original.commit_reference() != binding.commit_reference()
                || original.committed_product_publication()
                    != binding.committed_product_publication()
                || original.committed_product_publication().product_branch()
                    != binding.committed_product_publication().product_branch()
                || original
                    .committed_product_publication()
                    .product_incarnation()
                    != binding
                        .committed_product_publication()
                        .product_incarnation()
                || original
                    .committed_product_publication()
                    .product_generation()
                    != binding.committed_product_publication().product_generation()
            {
                return Err(Denial::CommitMismatch);
            }
            if original.record() != committed.record()
                || original.record_ref() != committed.record_ref()
            {
                return Err(Denial::RecordMismatch);
            }
            return Ok(original);
        }
        self.observe_held_original(
            execution,
            committed,
            binding.commit_reference(),
            binding.runtime_instance_id(),
            binding.committed_product_publication(),
        )
        .map(|owner| {
            WorthQueryCommittedDispatchOutboxObservation::seal(
                owner,
                binding.committed_product_publication().clone(),
            )
        })
    }
}
