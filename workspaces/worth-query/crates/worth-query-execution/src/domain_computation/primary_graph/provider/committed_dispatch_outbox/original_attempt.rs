//! Actual outstanding operation custody lends its exact World/Native poststate.
use super::*;
impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn original_attempt_dispatch_outbox(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &crate::domain_computation::primary_graph::application_attempt::WorthQueryPendingApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let Some(binding) = original.dispatch_binding() else {
            return Ok(None);
        };
        let publication = original.original_provider().committed_product_publication();
        self.observe_held_original(
            execution,
            binding,
            original.original_commit(),
            original.original_runtime_instance_id(),
            publication,
        )
        .map(|owner| {
            Some(WorthQueryCommittedDispatchOutboxObservation::seal(
                owner,
                publication.clone(),
            ))
        })
    }

    pub(super) fn observe_held_original(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &super::super::WorthQueryCommittedDispatchOutboxBinding,
        expected_commit: &RelationalCommitReceipt,
        expected_runtime: u64,
        publication: &WorthQueryCommittedProductPublication,
    ) -> Result<WorthQueryCommittedDispatchOutboxOwnerObservation, Denial> {
        if publication.relational_commit() != expected_commit {
            return Err(Denial::CommitMismatch);
        }
        let basis = publication
            .retained_relational_basis()
            .ok_or(Denial::ExactCommitUnavailable)?;
        // The window is an optional optimization. An evicted entry cannot
        // strand the exact poststate still owned by this original operation.
        let cached = self
            .receipt_basis_retention
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .acquire(expected_commit.commit_id);
        self.graph.with_runtime(execution, |runtime| {
            if runtime.runtime_instance_id() != expected_runtime {
                return Err(Denial::ForeignRuntime);
            }
            let acquired = if cached.is_none() {
                Some(
                    runtime
                        .retain_component_basis(basis)
                        .map_err(|_| Denial::ExactCommitUnavailable)?,
                )
            } else {
                None
            };
            let lease = cached
                .as_ref()
                .map(|basis| basis.lease())
                .or(acquired.as_ref())
                .expect("a genuine original lends either its cached or newly admitted retention");
            let answer = CommittedOutboxRead {
                runtime,
                layout: self.graph.layout.provider_dispatch_outbox(),
                record: binding.record(),
                record_ref: binding.record_ref(),
                expected_commit,
                expected_runtime,
                retained_basis: lease,
            }
            .resolve(execution);
            if let Some(lease) = acquired {
                runtime
                    .release_component_basis(lease)
                    .expect("the scoped original read releases its same-owner Native loan");
            }
            answer
        })
    }
}
