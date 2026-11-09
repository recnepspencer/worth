mod admission;
mod affected_indexes;
mod basis;
mod basis_currency;
mod branch_seed;
mod fork_selection;
pub(crate) use candidate_publication::PreparedCandidateIndexPublication;
pub(crate) use fork_selection::PreparedForkIndexSelection;
mod change_routing;
mod changes;
mod entry_edits;
mod entry_refresh;
use entry_refresh::{empty_entries, update_entries};
mod applied_field;
mod field;
mod field_delta;
pub(crate) use applied_field::AppliedFieldImage;
mod field_changes;
mod field_keys;
mod field_routing;
mod generation_reuse;
mod join;
mod ordering;
mod persistent_allocation;
mod preparation;
mod publication;
mod refresh_inputs;
mod refresh_preparation;
mod selected_field_request;
use publication::PreparedRefresh;
mod candidate_publication;
#[cfg(test)]
pub(crate) mod charge_rendezvous;
mod cold_projection;
mod maintained_selection;
mod owner;
mod pipeline_preparation;
mod reads;
mod record_metadata;
mod recovery;
mod request_payment;
mod root_view;
mod schema_transition;
#[cfg(test)]
pub(crate) use request_payment::last_allocation_bytes;
mod work;

use super::{IndexAuthority, IndexGenerationPublicationBasis};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::*;
use crate::snapshots::data::SnapshotHandle;
use work::MaintenanceWork;

impl IndexAuthority<'_> {
    /// Refresh one exact published basis. The optional before snapshot enables
    /// patch-local maintenance only when the canonical pre-commit root matches.
    /// Missing predecessor generations refuse regardless of the cold budget. All work
    /// is prepared before any generation is published; denial publishes none.
    pub fn refresh_for_basis(
        &self,
        request: DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        before: Option<&SnapshotHandle>,
        budget: DerivedIndexMaintenanceBudget,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let (prepared, counts) = request_payment::run(execution, budget, |work| {
            let prepared = self.prepare_basis_refresh(&request, basis, before, work, execution)?;
            prepared.publish_admitted(self.runtime, work)
        })?;
        Ok(DerivedIndexMaintenanceOutcome {
            generations: prepared.value,
            work: counts,
        })
    }

    fn prepare_basis_refresh(
        &self,
        request: &DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        before: Option<&SnapshotHandle>,
        work: &mut MaintenanceWork,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<PreparedRefresh, DerivedIndexMaintenanceDenialKind> {
        work.charge(request.index_ids.len())?;
        let branch = basis.identity().branch_id();
        work.prepare(
            (branch.0.len() as u64)
                .checked_add(2)
                .ok_or(DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded)?,
            branch.0.len() as u64,
        )?;
        let observation = basis.observation();
        let after = self
            .runtime
            .read_truth()
            .project_observation(&observation)
            .map_err(DerivedIndexMaintenanceDenialKind::Basis)?;
        let before = before
            .map(|snapshot| {
                if snapshot.branch_id() != &request.branch_id {
                    return Err(DerivedIndexMaintenanceDenialKind::BeforeRootMismatch);
                }
                self.runtime
                    .read_truth()
                    .project_snapshot(snapshot)
                    .ok_or(DerivedIndexMaintenanceDenialKind::SnapshotUnavailable)
            })
            .transpose()?;
        work.prepare(branch.0.len().min(request.branch_id.0.len()) as u64 + 1, 0)?;
        basis::validate(request, &observation, before.as_ref(), &after)?;
        let after = root_view::IndexRootView::from_projection(&after)?;
        let before = before
            .as_ref()
            .map(root_view::IndexRootView::from_projection)
            .transpose()?;
        self.maintenance()
            .prepare_refresh(execution, request, before.as_ref(), &after, work)
    }
}

#[cfg(test)]
mod persistent_allocation_tests;
