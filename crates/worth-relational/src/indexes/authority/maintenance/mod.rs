mod admission;
mod basis;
mod branch_seed;
pub(super) mod candidate;
mod change_routing;
mod changes;
mod entry_edits;
mod entry_refresh;
use entry_refresh::{empty_entries, update_entries};
mod field;
mod field_keys;
mod join;
mod ordering;
mod persistent_allocation;
pub(crate) use persistent_allocation::{persistent_map_path_bytes, row_copy_bounds};
mod preparation;
mod publication;
mod refresh_preparation;
mod retained_projection;
mod selected_field_request;
use publication::PreparedRefresh;
#[cfg(test)]
pub(crate) mod charge_rendezvous;
mod reads;
mod record_metadata;
mod request_payment;
#[cfg(test)]
pub(crate) use request_payment::last_allocation_bytes;
mod work;

use super::{IndexAuthority, IndexGenerationPublicationBasis};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::indexes::data::*;
use crate::runtime::VisibilityProjectionView;
use crate::snapshots::data::SnapshotHandle;
use work::MaintenanceWork;

impl IndexAuthority<'_> {
    /// Explicit bounded reconstruction for a committed version that is no
    /// longer a live branch head. Retention selects its actual immutable root;
    /// its admitted receipt and authoring facts must match the catalog artifact.
    pub fn reconstruct_for_commit(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: DerivedIndexBuildRequest,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let (prepared, counts) = request_payment::run(execution, budget, |work| {
            work.charge(request.index_ids.len())?;
            let recorded = self
                .runtime
                .history
                .commit_artifact(request.source_commit_id)
                .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
            let identity = recorded.identity();
            let branch_bytes = request.branch_id.0.len();
            // Compare the authoring name and copy the registry cell identity.
            work.prepare(2 * branch_bytes as u64 + 2, branch_bytes as u64)?;
            if identity.authoring_branch() != &request.branch_id {
                return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
            }
            let projection = match self.project_registered_retained_commit(
                execution,
                &request,
                identity.version_id(),
                work,
            )? {
                Some(projection) => projection,
                None => {
                    work.prepare(branch_bytes as u64 + 1, branch_bytes as u64)?;
                    self.runtime
                        .read_truth()
                        .try_project_retained_commit(
                            request.source_commit_id,
                            request.branch_id.clone(),
                            identity.version_id(),
                        )
                        .map_err(DerivedIndexMaintenanceDenialKind::Basis)?
                }
            };
            let root = projection
                .selected_root()
                .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
            work.prepare(branch_bytes as u64 + 1, 0)?;
            if root.commit_receipt().is_none_or(|receipt| {
                receipt.commit_id != request.source_commit_id
                    || receipt.version_id != identity.version_id()
            }) || root
                .commit_evidence()
                .is_none_or(|evidence| evidence.authoring_branch() != &request.branch_id)
                || root.descriptor() != Some(recorded.roots())
            {
                return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
            }
            let prepared =
                self.prepare_refresh(execution, &request, None, None, &projection, work)?;
            prepared.publish_admitted(self.runtime, work)
        })?;
        Ok(DerivedIndexMaintenanceOutcome {
            generations: prepared.value,
            work: counts,
        })
    }

    /// Refresh one exact published basis. The optional before snapshot enables
    /// patch-local maintenance only when the canonical pre-commit root matches.
    /// Missing generations/snapshots require the explicit cold budget. All work
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
        basis::validate(request, &observation, before.as_ref(), &after, execution)?;
        self.prepare_refresh(execution, request, before.as_ref(), None, &after, work)
    }
}

#[cfg(test)]
mod persistent_allocation_tests;
