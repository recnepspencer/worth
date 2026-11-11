//! Fund the installed catalog selection before recovery traverses or copies it.
use super::{request_payment, work::MaintenanceWork, IndexAuthority};
use crate::indexes::data::{
    DerivedIndexBuildRequest, DerivedIndexId, DerivedIndexMaintenanceBudget,
    DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind as Denial,
    DerivedIndexMaintenanceOutcome,
};

impl IndexAuthority<'_> {
    pub(crate) fn reconstruct_installed_for_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> Result<DerivedIndexMaintenanceOutcome, DerivedIndexMaintenanceDenial> {
        let budget = DerivedIndexMaintenanceBudget {
            maximum_work_units: None,
            maximum_cold_record_slots: None,
            maximum_derived_rows: None,
        };
        let (prepared, _) = request_payment::run(execution, budget, |work| {
            let observation = basis.observation();
            let source_commit_id = observation.commit_id().ok_or(Denial::CommitMismatch)?;
            let branch = observation.identity().branch_id();
            work.prepare(branch.0.len() as u64, branch.0.len() as u64)?;
            let mut branch_name = String::new();
            branch_name
                .try_reserve_exact(branch.0.len())
                .map_err(|_| allocation_failed())?;
            branch_name.push_str(&branch.0);
            let index_ids = self.runtime.indexes.with_definitions(|definitions| {
                collect_ids(&mut definitions.keys().copied(), work)
            })?;
            Ok(DerivedIndexBuildRequest {
                source_commit_id,
                branch_id: crate::history::data::BranchId(branch_name),
                index_ids,
            })
        })?;
        let (request, funding) = prepared.into_funded_parts();
        let result = self.reconstruct_for_basis(execution, request, basis, budget);
        drop(funding);
        result
    }
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
