use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::runtime::RelationalRuntime;

use super::WorthQueryPrimaryGraphIntegrationHandle;

#[path = "index_currency/selected_field.rs"]
mod selected_field;
pub(crate) use selected_field::ensure_selected_field_indexes_admitted;
pub(crate) use selected_field::SelectedFieldIndexAdmissionStop;

#[derive(Debug)]
pub enum WorthQueryPrimaryIndexCurrencyDenial {
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Basis(super::exact_basis_access::WorthQueryExactBasisSnapshotDenial),
    IndexUnavailable(&'static str),
}

/// Probe the one selected published generation through the native index
/// authority. The primary integration handle's installed index inventory is
/// irrelevant to this exact read.
pub(crate) fn selected_index_is_current(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    index: DerivedIndexId,
) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
    runtime
        .index_access()
        .read_published_generation_for_observation(execution, index, &basis.observation())
        .map(|generation| generation.is_some())
}

impl WorthQueryPrimaryGraphIntegrationHandle {
    /// Carries installed selections to an admitted fork's identical root.
    /// Each selected read still admits its own required index generation.
    pub(crate) fn carry_primary_index_basis_for_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &mut RelationalRuntime,
        basis: &AdmittedRelationalBranchBasis,
    ) -> Result<(), WorthQueryPrimaryIndexCurrencyDenial> {
        execution
            .consult()
            .map_err(|stop| WorthQueryPrimaryIndexCurrencyDenial::Execution(stop.into()))?;
        runtime
            .index_authority()
            .carry_index_basis_for_basis(execution, basis)
            .map(|_| ())
            .map_err(currency_denial)
    }
}

fn currency_denial(
    denial: worth_relational::facade::indexes::DerivedIndexMaintenanceDenial,
) -> WorthQueryPrimaryIndexCurrencyDenial {
    use worth_relational::facade::indexes::DerivedIndexMaintenanceDenialKind as Native;
    match denial.kind {
        Native::Execution(cause) => WorthQueryPrimaryIndexCurrencyDenial::Execution(cause),
        Native::Basis(basis) => {
            WorthQueryPrimaryIndexCurrencyDenial::Basis(index_basis_denial(basis))
        }
        _ => WorthQueryPrimaryIndexCurrencyDenial::IndexUnavailable(
            "installed index predecessor is unavailable; explicit index recovery is required",
        ),
    }
}

fn index_basis_denial(
    denial: worth_relational::facade::branch::RelationalBranchBasisDenial,
) -> super::exact_basis_access::WorthQueryExactBasisSnapshotDenial {
    use super::exact_basis_access::WorthQueryExactBasisSnapshotDenial as QueryDenial;
    use worth_relational::facade::branch::RelationalBranchBasisDenial as Denial;
    match denial {
        Denial::RetentionCapacityExhausted => QueryDenial::RetentionCapacityExhausted,
        Denial::RetentionIdentityExhausted => QueryDenial::RetentionIdentityExhausted,
        Denial::SnapshotIdentityExhausted => QueryDenial::SnapshotIdentityExhausted,
        Denial::MaterializationUnavailable => QueryDenial::BranchMaterializationSuspended,
        Denial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        } => QueryDenial::ForeignRuntime {
            expected_runtime_instance_id,
            actual_runtime_instance_id,
        },
        _ => QueryDenial::BranchObservationUnavailable,
    }
}

mod court_doors_for_test;
