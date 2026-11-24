//! Recover missing catalog coverage without replacing admitted predecessor selections.
use super::work::MaintenanceWork;
use crate::branch::{
    basis::AdmittedRelationalBranchBasis, basis_denial::RelationalBranchBasisDenial,
};
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexId, DerivedIndexMaintenanceDenialKind as Denial,
};
use crate::mvcc::observation::RelationalBranchObservation;
use crate::runtime::state::subsystems::indexing::IndexingSubsystem;
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;

pub(super) fn missing_build_request(
    indexes: &IndexingSubsystem,
    lane: &ReconstructionLane<'_>,
    basis: &AdmittedRelationalBranchBasis,
    work: &mut MaintenanceWork<'_, '_, '_>,
    project: impl FnOnce(
        &RelationalBranchObservation,
    ) -> Result<VisibilityProjectionView, RelationalBranchBasisDenial>,
) -> Result<DerivedIndexBuildRequest, Denial> {
    let observation = basis.observation();
    let source_commit_id = observation.commit_id().ok_or(Denial::CommitMismatch)?;
    let branch = observation.identity().branch_id();
    let projection = project(&observation).map_err(Denial::Basis)?;
    let root = projection
        .retain_selected_root()
        .ok_or(Denial::SnapshotUnavailable)?;
    let selected = indexes.prepare_reconstruction_basis(
        lane,
        &root,
        branch,
        &mut |units, bytes| work.prepare(units, bytes),
        None,
    )?;
    work.prepare(branch.0.len() as u64, branch.0.len() as u64)?;
    let mut branch_name = String::new();
    branch_name
        .try_reserve_exact(branch.0.len())
        .map_err(|_| allocation_failed())?;
    branch_name.push_str(&branch.0);
    let mut index_ids = indexes
        .with_definitions(|definitions| collect_ids(&mut definitions.keys().copied(), work))?;
    retain_missing(&mut index_ids, &selected, work)?;
    Ok(DerivedIndexBuildRequest {
        source_commit_id,
        branch_id: crate::history::data::BranchId(branch_name),
        index_ids,
    })
}

fn retain_missing(
    ids: &mut Vec<DerivedIndexId>,
    selected: &crate::runtime::state::subsystems::indexing::PreparedIndexBasisSelection,
    work: &mut MaintenanceWork<'_, '_, '_>,
) -> Result<(), Denial> {
    let count = ids.len();
    let mut missing = 0;
    for read in 0..count {
        let index = ids[read];
        work.lookup(count, 1)?;
        if selected.generation(index).is_some() {
            continue;
        }
        work.lookup(count, 1)?;
        if !selected.generation_is_empty(index) {
            ids[missing] = index;
            missing += 1;
        }
    }
    ids.truncate(missing);
    Ok(())
}

fn collect_ids(
    ids: &mut dyn ExactSizeIterator<Item = DerivedIndexId>,
    work: &mut MaintenanceWork<'_, '_, '_>,
) -> Result<Vec<DerivedIndexId>, Denial> {
    let count = ids.len();
    work.charge(count)?;
    work.array::<DerivedIndexId>(count)?;
    let mut selected = Vec::new();
    selected
        .try_reserve_exact(count)
        .map_err(|_| allocation_failed())?;
    selected.extend(ids);
    Ok(selected)
}

fn allocation_failed() -> Denial {
    Denial::Execution(crate::execution::RelationalExecutionDenialCause::AllocationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexes::data::DerivedIndexMaintenanceBudget;

    #[test]
    fn catalog_selection_requires_work_and_backing_before_visiting_ids() {
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: Some(0),
            maximum_cold_record_slots: None,
            maximum_derived_rows: None,
        };
        let original = [DerivedIndexId(1), DerivedIndexId(2)];
        let mut unvisited = original
            .into_iter()
            .inspect(|_| panic!("unfunded traversal"));
        assert_eq!(
            collect_ids(&mut unvisited, &mut MaintenanceWork::new(budget)),
            Err(Denial::WorkBudgetExceeded)
        );
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: None,
            ..budget
        };
        let mut refuse_backing = |_, bytes| {
            if bytes > 0 {
                Err(super::super::work::MaintenanceAdmissionRefusal)
            } else {
                Ok(())
            }
        };
        assert_eq!(
            collect_ids(
                &mut unvisited,
                &mut MaintenanceWork::admitted(budget, &mut refuse_backing)
            ),
            Err(Denial::WorkBudgetExceeded)
        );
        assert_eq!(
            collect_ids(&mut original.into_iter(), &mut MaintenanceWork::new(budget)).unwrap(),
            original
        );
    }
}
