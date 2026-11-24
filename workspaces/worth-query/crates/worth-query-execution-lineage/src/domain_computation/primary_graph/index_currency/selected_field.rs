//! Selected native EntityField currency for fresh principal and retained scope.

use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::DerivedIndexBuildRequest;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::RelationalRuntime;

use super::selected_index_is_current;

#[derive(Debug)]
pub enum SelectedFieldIndexAdmissionStop {
    Admission(CompanionPreflightStop),
    Native,
}

/// Prepare only the installed EntityField indexes consumed by this
/// principal/scope request. Missing generations require an authentic native seed;
/// this ordinary selector never reconstructs the population. Every action uses the caller's
/// original cumulative admission and the native maintenance owner.
pub fn ensure_selected_field_indexes_admitted<const N: usize>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    indexes: [Option<DerivedIndexId>; N],
) -> Result<(), SelectedFieldIndexAdmissionStop> {
    use SelectedFieldIndexAdmissionStop as Stop;
    let mut stale = [None; N];
    let mut stale_count = 0_usize;
    let mut seen = [None; N];
    let mut seen_count = 0_usize;
    for index in indexes.into_iter().flatten() {
        if seen[..seen_count].contains(&Some(index)) {
            continue;
        }
        seen[seen_count] = Some(index);
        seen_count += 1;
        if !selected_index_is_current(execution, runtime, basis, index)
            .map_err(|cause| Stop::Admission(CompanionPreflightStop::Execution(cause)))?
        {
            stale[stale_count] = Some(index);
            stale_count += 1;
        }
    }
    if stale_count == 0 {
        return Ok(());
    }

    let mut selected = Vec::with_capacity(stale_count);
    for index in stale.into_iter().flatten() {
        selected.push(index);
    }
    refresh_stale_indexes(execution, runtime, basis, selected)
}

/// Slice variant for queries whose installed finite root shape combines
/// multiple root-entity structured equality predicates. Reserves scratch memory
/// and charges O(n log n) dedup work before allocating or scanning.
pub fn ensure_selected_field_index_slice_admitted(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    indexes: &[DerivedIndexId],
) -> Result<(), SelectedFieldIndexAdmissionStop> {
    use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    use SelectedFieldIndexAdmissionStop as Stop;
    if indexes.is_empty() {
        return Ok(());
    }
    let count = indexes.len();
    let log = if count <= 1 {
        1_u64
    } else {
        u64::from(usize::BITS - (count - 1).leading_zeros()) + 1
    };
    let work_units = (count as u64).checked_mul(log).ok_or(Stop::Admission(
        CompanionPreflightStop::Execution(Cause::WorkCounterOverflow),
    ))?;
    SourceFactsAllocation::work(execution, work_units)
        .map_err(|cause| Stop::Admission(CompanionPreflightStop::Execution(cause)))?;
    let allocation = SourceFactsAllocation::open(execution)
        .map_err(|cause| Stop::Admission(CompanionPreflightStop::Execution(cause)))?;
    let scratch_len = count.checked_mul(2).ok_or(Stop::Admission(
        CompanionPreflightStop::PreparationMemoryCounterOverflow,
    ))?;
    allocation
        .claim_slice::<DerivedIndexId>(execution, scratch_len)
        .map_err(|cause| Stop::Admission(CompanionPreflightStop::Execution(cause)))?;
    let mut sorted = Vec::new();
    sorted
        .try_reserve_exact(count)
        .map_err(|_| Stop::Admission(CompanionPreflightStop::PreparationMemoryCounterOverflow))?;
    sorted.extend_from_slice(indexes);
    sorted.sort_unstable();
    sorted.dedup();
    let mut selected = Vec::new();
    selected
        .try_reserve_exact(sorted.len())
        .map_err(|_| Stop::Admission(CompanionPreflightStop::PreparationMemoryCounterOverflow))?;
    for index in sorted {
        if !selected_index_is_current(execution, runtime, basis, index)
            .map_err(|cause| Stop::Admission(CompanionPreflightStop::Execution(cause)))?
        {
            selected.push(index);
        }
    }
    if selected.is_empty() {
        return Ok(());
    }
    refresh_stale_indexes(execution, runtime, basis, selected)
}

fn refresh_stale_indexes(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    selected: Vec<DerivedIndexId>,
) -> Result<(), SelectedFieldIndexAdmissionStop> {
    use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    use SelectedFieldIndexAdmissionStop as Stop;
    let stale_count = selected.len();
    let observation = basis.observation();
    let Some(source_commit_id) = observation.commit_id() else {
        return Ok(());
    };
    let branch = observation.identity().branch_id();
    let branch_work = u64::try_from(branch.0.len())
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or(Stop::Admission(CompanionPreflightStop::Execution(
            Cause::WorkCounterOverflow,
        )))?;
    SourceFactsAllocation::work(execution, branch_work)
        .map_err(|cause| Stop::Admission(CompanionPreflightStop::Execution(cause)))?;
    let branch_alloc = SourceFactsAllocation::open(execution)
        .map_err(|cause| Stop::Admission(CompanionPreflightStop::Execution(cause)))?;
    let branch_bytes = u64::try_from(branch.0.capacity())
        .map_err(|_| Stop::Admission(CompanionPreflightStop::PreparationMemoryCounterOverflow))?;
    if branch_bytes > 0 {
        branch_alloc
            .claim(execution, branch_bytes)
            .map_err(|cause| Stop::Admission(CompanionPreflightStop::Execution(cause)))?;
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
        .map_err(|denial| match denial.kind {
            worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::Execution(
                cause,
            ) => Stop::Admission(CompanionPreflightStop::Execution(cause)),
            _ => Stop::Native,
        })?;
    if build.generations.len() != stale_count {
        return Err(Stop::Native);
    }
    Ok(())
}
