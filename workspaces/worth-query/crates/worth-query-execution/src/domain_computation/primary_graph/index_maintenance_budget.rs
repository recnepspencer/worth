use worth_relational::facade::indexes::DerivedIndexMaintenanceBudget;

/// Ordinary commit publication pays for changed records, bounded adjacency,
/// and persistent entry paths. It never admits a full-root reconstruction.
pub(super) const fn ordinary_index_maintenance_budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: 4_000_000,
        maximum_cold_record_slots: 0,
        maximum_derived_rows: 100_000,
    }
}

/// Recovery after an interrupted publication has a separate, finite budget.
/// Its counters are returned by Relational with typed exhaustion.
pub(super) const fn cold_index_reconstruction_budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: 32_000_000,
        maximum_cold_record_slots: 1_000_000,
        maximum_derived_rows: 1_000_000,
    }
}

/// A captured pre-commit root does not prove that every index has a compatible
/// prior generation. Retry only that specific absence through the explicit
/// finite cold lane; ordinary work exhaustion never becomes an unbounded scan.
pub(super) fn refresh_with_cold_fallback(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    request: worth_relational::facade::indexes::DerivedIndexBuildRequest,
    basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    before: Option<&worth_relational::facade::snapshots::SnapshotHandle>,
) -> Result<
    worth_relational::facade::indexes::DerivedIndexMaintenanceOutcome,
    worth_relational::facade::indexes::DerivedIndexMaintenanceDenial,
> {
    let initial = runtime.index_authority().refresh_for_basis(
        request.clone(),
        basis,
        before,
        if before.is_some() {
            ordinary_index_maintenance_budget()
        } else {
            cold_index_reconstruction_budget()
        },
    );
    if matches!(
        &initial,
        Err(denial) if matches!(denial.kind,
            worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired)
    ) {
        runtime.index_authority().refresh_for_basis(
            request,
            basis,
            before,
            cold_index_reconstruction_budget(),
        )
    } else {
        initial
    }
}
