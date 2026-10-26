//! Bounded branch metadata selection precedes every aspect-history body demand.
use super::HistoryAccess;
use crate::execution::RelationalExecutionDenialCause as Denial;
use crate::history::data::{BranchId, CanonicalCommitEnvelope};
use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::sync::Arc;

pub(super) const ASPECT_HISTORY_COMMIT_CEILING: NonZeroUsize = NonZeroUsize::new(4096).unwrap();

impl HistoryAccess<'_> {
    pub(super) fn demand_branch_history_regions(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &BranchId,
        partitions: impl IntoIterator<Item = crate::identity::data::PartitionId>,
    ) -> Result<(), Denial> {
        crate::execution::check_request(execution)?;
        if let Some(cell) = self.runtime.history.branch_cell(branch)
            && let Some(root) = cell.root()
        {
            root.demand_regions_full(execution, &self.runtime.services.symbols, partitions)?;
        }
        Ok(())
    }

    pub(super) fn demand_branch_commit_envelopes(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &BranchId,
        ceiling: NonZeroUsize,
    ) -> Result<Vec<Arc<CanonicalCommitEnvelope>>, Denial> {
        crate::execution::check_request(execution)?;
        let Some(head) = self.branch_head(branch) else {
            return Ok(Vec::new());
        };
        let mut scheduled = BTreeSet::from([head.commit_id]);
        let mut pending = vec![head.commit_id];
        let mut selected = Vec::new();
        while let Some(commit) = pending.pop() {
            crate::execution::check_request(execution)?;
            let receipt = self
                .immutable_commit_receipt(commit)
                .ok_or(Denial::EquivalenceContractUnavailable)?;
            for parent in &receipt.parents {
                if !scheduled.contains(parent) {
                    if scheduled.len() >= ceiling.get() {
                        return Err(Denial::WorkExhausted);
                    }
                    scheduled.insert(*parent);
                    pending
                        .try_reserve(1)
                        .map_err(|_| Denial::AllocationFailed)?;
                    pending.push(*parent);
                }
            }
            if &receipt.branch_id == branch {
                selected
                    .try_reserve(1)
                    .map_err(|_| Denial::AllocationFailed)?;
                selected.push(receipt);
            }
        }
        selected.sort_by_key(|receipt| (receipt.version_id.0, receipt.commit_id.0));
        let mut envelopes = Vec::new();
        envelopes
            .try_reserve(selected.len())
            .map_err(|_| Denial::AllocationFailed)?;
        for receipt in selected {
            let envelope = self
                .demand_commit_envelope(execution, receipt.commit_id)?
                .ok_or(Denial::EquivalenceContractUnavailable)?;
            envelopes.push(envelope);
        }
        crate::execution::check_request(execution)?;
        Ok(envelopes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::support::*;

    #[test]
    fn branch_history_ceiling_refuses_complete_warm_history_without_partial_results() {
        let host = crate::test_execution_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host);
        let runtime = runtime_with_test_schema();
        let entity = create_entity(execution, &runtime, "bounded-history-first");
        for value in ["bounded-history-second", "bounded-history-third"] {
            update_entity_and_release_snapshot(execution, &runtime, entity, value);
        }
        let frontier = runtime.history().latest_patch_stream_position();
        let branch = BranchId("main".into());
        assert_eq!(
            runtime
                .history()
                .demand_branch_commit_envelopes(execution, &branch, NonZeroUsize::new(1).unwrap())
                .unwrap_err(),
            Denial::WorkExhausted
        );
        let complete = runtime
            .history()
            .demand_branch_commit_envelopes(execution, &branch, NonZeroUsize::new(8).unwrap())
            .unwrap();
        assert!(complete.len() > 1 && complete.len() < 8);
        assert_eq!(
            complete.last().unwrap().commit.commit_id,
            runtime.history().branch_head(&branch).unwrap().commit_id
        );
        assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    }
}
