//! Select the installed read operands before handing off to the read implementation.
use super::*;
use crate::visibility::materialization::read_records::reader::context::VisibilityReadContext;

impl VisibilityReadContext<'_> {
    pub fn read_historical_selection(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        selected: &RelationalBranchObservation,
        restored: &RelationalBranchObservation,
    ) -> Result<RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        crate::execution::request_custody::check_request(execution).map_err(Denial::Execution)?;
        for source in [selected, restored] {
            if source.identity().runtime_instance_id() != self.runtime().runtime_instance_id() {
                return Err(Denial::ForeignOwner);
            }
        }
        let commit = restored
            .selected_root()
            .commit_id()
            .ok_or(Denial::NotGraphSelection { commit: None })?;
        let envelope = self
            .runtime()
            .history()
            .demand_commit_envelope(execution, commit)
            .map_err(Denial::Execution)?
            .ok_or(Denial::NotGraphSelection {
                commit: Some(commit),
            })?;
        let CanonicalCommitAuthorityKind::OwnedGraphSelection(graph) = &envelope.authority_kind
        else {
            return Err(Denial::NotGraphSelection {
                commit: Some(commit),
            });
        };
        // Canonicalization copies branch text and parent IDs, never records.
        // Fund both its temporary backing and this retained read capsule first.
        let descriptor = selected.descriptor();
        let parents = descriptor
            .reference()
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        let bytes = (std::mem::size_of::<RelationalHistoricalSelectionRead>() as u64)
            .checked_add((descriptor.branch_id().0.len() as u64).saturating_mul(4))
            .and_then(|n| n.checked_add((parents as u64).saturating_mul(32)))
            .and_then(|n| n.checked_add(1024))
            .ok_or(Denial::Execution(Cause::ChargedBytesOverflow))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Denial::Request)?;
        execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, bytes)
            })
            .map_err(Denial::Request)?
            .0
            .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        let canonical = descriptor
            .in_canonical_frame()
            .map_err(|_| Denial::SelectedBasisMismatch { commit })?;
        if graph.selected_basis() != &canonical
            || selected.descriptor().schema_commitment()
                != restored.descriptor().schema_commitment()
            || envelope.commit.version_id != restored.version_id()
            || restored.commit_id() != Some(commit)
        {
            return Err(Denial::SelectedBasisMismatch { commit });
        }
        let project = |source| {
            VisibilityProjectionView::new(
                self.runtime(),
                crate::visibility::snapshot_states::SnapshotStateBasis::Exact(
                    crate::history::retention::VisibilitySnapshotBasis::from_observation(source),
                ),
            )
        };
        Ok(RelationalHistoricalSelectionRead {
            routes: std::sync::OnceLock::new(),
            stages: vec![HistoricalSelectionStage {
                selected: project(selected),
                restored: project(restored),
                envelope,
                _memory: memory,
            }],
            _memory: None,
        })
    }
}
