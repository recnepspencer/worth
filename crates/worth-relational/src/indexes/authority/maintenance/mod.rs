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
mod preparation;
mod publication;
mod refresh_preparation;
use publication::PreparedRefresh;
mod reads;
mod record_metadata;
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
    /// the root's envelope must match the requested commit and branch.
    pub fn reconstruct_for_commit(
        &self,
        request: DerivedIndexBuildRequest,
        budget: DerivedIndexMaintenanceBudget,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let mut work = MaintenanceWork::new(budget);
        let result = (|| {
            work.charge(request.index_ids.len())?;
            let recorded = self
                .runtime
                .history
                .recorded_commit_envelope(request.source_commit_id)
                .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
            if recorded.branch_context != request.branch_id {
                return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
            }
            let projection = self
                .runtime
                .read_truth()
                .try_project_retained_commit(
                    request.source_commit_id,
                    request.branch_id.clone(),
                    recorded.commit.version_id,
                )
                .map_err(DerivedIndexMaintenanceDenialKind::Basis)?;
            let root = projection
                .selected_root()
                .ok_or(DerivedIndexMaintenanceDenialKind::CommitMismatch)?;
            if root.commit_id() != Some(request.source_commit_id)
                || root
                    .canonical_envelope()
                    .is_none_or(|envelope| envelope.branch_context != request.branch_id)
            {
                return Err(DerivedIndexMaintenanceDenialKind::CommitMismatch);
            }
            self.prepare_refresh(&request, None, &projection, &mut work)
        })();
        result
            .map(|prepared| DerivedIndexMaintenanceOutcome {
                generations: prepared.publish(self.runtime),
                work: work.counts,
            })
            .map_err(|kind| work.deny(kind))
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
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let mut work = MaintenanceWork::new(budget);
        let result = self.prepare_basis_refresh(&request, basis, before, &mut work);
        result
            .map(|prepared| DerivedIndexMaintenanceOutcome {
                generations: prepared.publish(self.runtime),
                work: work.counts,
            })
            .map_err(|kind| work.deny(kind))
    }

    fn prepare_basis_refresh(
        &self,
        request: &DerivedIndexBuildRequest,
        basis: &AdmittedRelationalBranchBasis,
        before: Option<&SnapshotHandle>,
        work: &mut MaintenanceWork,
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
        self.prepare_refresh(request, before.as_ref(), &after, work)
    }
}
