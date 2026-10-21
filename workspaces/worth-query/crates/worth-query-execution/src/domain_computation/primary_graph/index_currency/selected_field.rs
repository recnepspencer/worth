//! Selected native EntityField currency for fresh principal and retained scope.

use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::RelationalRuntime;

use super::selected_index_is_current;

#[derive(Debug)]
pub(crate) enum SelectedFieldIndexAdmissionStop {
    Admission(CompanionPreflightStop),
    Native,
}

/// Prepare only the installed EntityField indexes consumed by this
/// principal/scope request. Missing generations require an authentic native seed;
/// this ordinary selector never reconstructs the population. Every action uses the caller's
/// original cumulative admission and the native maintenance owner.
pub(crate) fn ensure_selected_field_indexes_admitted(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    indexes: [Option<DerivedIndexId>; 2],
) -> Result<(), SelectedFieldIndexAdmissionStop> {
    use SelectedFieldIndexAdmissionStop as Stop;
    let mut stale = [None, None];
    let mut stale_count = 0_usize;
    let mut seen = [None, None];
    let mut seen_count = 0_usize;
    for index in indexes.into_iter().flatten() {
        if seen[..seen_count].contains(&Some(index)) {
            continue;
        }
        seen[seen_count] = Some(index);
        seen_count += 1;
        if !selected_index_is_current(runtime, basis, index) {
            stale[stale_count] = Some(index);
            stale_count += 1;
        }
    }
    if stale_count == 0 {
        return Ok(());
    }

    let observation = basis.observation();
    let Some(source_commit_id) = observation.commit_id() else {
        return Ok(());
    };
    let branch = observation.identity().branch_id();

    let mut selected = Vec::with_capacity(stale_count);
    for index in stale.into_iter().flatten() {
        selected.push(index);
    }
    let build = runtime
        .index_authority()
        .refresh_field_indexes_for_basis(
            DerivedIndexBuildRequest {
                source_commit_id,
                branch_id: branch.clone(),
                index_ids: selected,
            },
            basis,
            None,
            super::super::index_maintenance_budget::ordinary_index_maintenance_budget(),
            execution,
        )
        .map_err(|_| Stop::Native)?;
    if build.generations.len() != stale_count {
        return Err(Stop::Native);
    }
    Ok(())
}
