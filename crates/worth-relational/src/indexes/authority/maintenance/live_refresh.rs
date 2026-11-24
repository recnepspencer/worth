//! Run installed index refresh against the retained predecessor and successor.
use super::super::live_access::IndexAuthority;
use super::*;

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
            prepared.publish_admitted(&self.runtime.indexes, work)
        })?;
        Ok(DerivedIndexMaintenanceOutcome {
            generations: prepared.value,
            work: counts,
        })
    }

    pub(super) fn prepare_basis_refresh(
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
        let after = root_view::IndexRootView::from_projection(&after)?;
        let before = before
            .as_ref()
            .map(root_view::IndexRootView::from_projection)
            .transpose()?;
        self.maintenance()
            .prepare_refresh(execution, request, before.as_ref(), &after, work)
    }
}
