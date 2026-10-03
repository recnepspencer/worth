//! Selected native EntityField currency for fresh principal and retained scope.

use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::{
    DerivedIndexBuildRequest, DerivedIndexId, DerivedIndexMaintenanceAdmissionStop,
    DerivedIndexMaintenanceDenialKind,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::RelationalRuntime;

use super::super::output_lineage::invalidation::InvalidationEditAdmission;
use super::selected_index_is_current_admitted;

#[derive(Debug)]
pub(crate) enum SelectedFieldIndexAdmissionStop {
    Admission(CompanionPreflightStop),
    Native(DerivedIndexMaintenanceDenialKind),
}

/// Prepare only the installed EntityField indexes consumed by this
/// principal/scope request. Every cold native action uses the caller's
/// original cumulative admission and the native maintenance owner.
pub(crate) fn ensure_selected_field_indexes_admitted(
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    indexes: [Option<DerivedIndexId>; 2],
    admission: &mut InvalidationEditAdmission,
) -> Result<(), SelectedFieldIndexAdmissionStop> {
    use SelectedFieldIndexAdmissionStop as Stop;
    let mut stale = [None, None];
    let mut stale_count = 0_usize;
    let mut seen = [None, None];
    let mut seen_count = 0_usize;
    for index in indexes.into_iter().flatten() {
        admission.charge_external_work(1).map_err(Stop::Admission)?;
        if seen[..seen_count].contains(&Some(index)) {
            continue;
        }
        seen[seen_count] = Some(index);
        seen_count += 1;
        if !selected_index_is_current_admitted(runtime, basis, index, admission)
            .map_err(Stop::Admission)?
        {
            stale[stale_count] = Some(index);
            stale_count += 1;
        }
    }
    if stale_count == 0 {
        return Ok(());
    }
    admission.charge_external_work(1).map_err(Stop::Admission)?;
    let observation = basis.observation();
    let Some(source_commit_id) = observation.commit_id() else {
        return Ok(());
    };
    let branch = observation.identity().branch_id();
    let branch_bytes = u64::try_from(branch.0.len())
        .map_err(|_| Stop::Admission(CompanionPreflightStop::WorkCounterOverflow))?;
    let id_bytes = stale_count
        .checked_mul(std::mem::size_of::<DerivedIndexId>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Stop::Admission(
            CompanionPreflightStop::PreparationMemoryCounterOverflow,
        ))?;
    let preparation_bytes = id_bytes.checked_add(branch_bytes).ok_or(Stop::Admission(
        CompanionPreflightStop::PreparationMemoryCounterOverflow,
    ))?;
    let copy_work = branch_bytes
        .checked_add(2)
        .ok_or(Stop::Admission(CompanionPreflightStop::WorkCounterOverflow))?;
    admission
        .admit_read_scratch(preparation_bytes)
        .and_then(|()| admission.charge_external_work(copy_work))
        .map_err(Stop::Admission)?;
    let mut selected = Vec::with_capacity(stale_count);
    for index in stale.into_iter().flatten() {
        selected.push(index);
    }
    let build = runtime
        .index_authority()
        .refresh_field_indexes_for_basis_admitted(
            DerivedIndexBuildRequest {
                source_commit_id,
                branch_id: branch.clone(),
                index_ids: selected,
            },
            basis,
            super::super::index_maintenance_budget::cold_index_reconstruction_budget(),
            |work, bytes| {
                admission.admit_read_scratch(bytes)?;
                admission.charge_external_work(work)
            },
        )
        .map_err(|stop| match stop {
            DerivedIndexMaintenanceAdmissionStop::Admission(stop) => Stop::Admission(stop),
            DerivedIndexMaintenanceAdmissionStop::Native(denial) => Stop::Native(denial.kind),
        })?;
    if build.generations.len() != stale_count {
        return Err(Stop::Native(
            DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired,
        ));
    }
    Ok(())
}
