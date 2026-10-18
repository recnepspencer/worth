//! Only the selected page index may enter explicitly budgeted native cold work.
use super::*;
use crate::domain_computation::primary_graph::index_currency::selected_index_is_current_admitted;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    indexes::{DerivedIndexBuildRequest, DerivedIndexId},
    runtime::RelationalRuntime,
};
pub(in crate::domain_computation::primary_graph::application_discovery::workflow) fn prepare(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    index: DerivedIndexId,
    mode: WorthQueryWorkflowIndexRead,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), Denial> {
    if selected_index_is_current_admitted(runtime, basis, index, admission)
        .map_err(Denial::Preparation)?
    {
        return Ok(());
    }
    let WorthQueryWorkflowIndexRead::Reconstruct(requested) = mode else {
        return Err(Denial::SelectedIndexUnavailable);
    };
    let host=crate::domain_computation::primary_graph::index_maintenance_budget::cold_index_reconstruction_budget();
    let budget = DerivedIndexMaintenanceBudget {
        maximum_work_units: requested
            .maximum_work_units
            .into_iter()
            .chain(host.maximum_work_units)
            .min(),
        maximum_cold_record_slots: requested
            .maximum_cold_record_slots
            .into_iter()
            .chain(host.maximum_cold_record_slots)
            .min(),
        maximum_derived_rows: requested
            .maximum_derived_rows
            .into_iter()
            .chain(host.maximum_derived_rows)
            .min(),
    };
    let observation = basis.observation();
    let commit = observation.commit_id().ok_or(Denial::BasisUnavailable)?;
    let branch = observation.identity().branch_id();
    let bytes = (branch.0.len() + std::mem::size_of::<DerivedIndexId>()) as u64;
    admission
        .admit_read_scratch(bytes)
        .and_then(|()| admission.charge_external_work(branch.0.len() as u64 + 2))
        .map_err(Denial::Preparation)?;
    runtime
        .index_authority()
        .refresh_for_basis(
            DerivedIndexBuildRequest {
                source_commit_id: commit,
                branch_id: branch.clone(),
                index_ids: vec![index],
            },
            basis,
            None,
            budget,
            execution,
        )
        .map_err(Denial::Reconstruction)?;
    if !selected_index_is_current_admitted(runtime, basis, index, admission)
        .map_err(Denial::Preparation)?
    {
        return Err(Denial::SelectedIndexUnavailable);
    }
    Ok(())
}
