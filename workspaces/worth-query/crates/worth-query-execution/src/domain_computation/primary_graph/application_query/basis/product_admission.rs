use super::{admission_denial, map_index_currency_denial, WorthQueryApplicationQueryBasisCustody};
use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) fn admit<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: WorthQueryProductObservationLease,
    application_basis: super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
) -> Result<WorthQueryApplicationQueryBasisCustody, WorthQueryApplicationQueryAdmissionDenial> {
    let selected =
        admit_selected_permission_basis(execution, application, product, application_basis)?;
    let graph = application
        .runtime
        .primary_graph()
        .ok_or_else(|| {
            admission_denial(
                WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
                "primary graph",
            )
        })?
        .integration_handle();
    graph
        .with_runtime_mut(execution, |runtime| {
            graph.ensure_primary_indexes_for_basis(
                execution,
                runtime,
                selected.selected_product().relational_basis(),
            )
        })
        .map_err(map_index_currency_denial)?;
    Ok(selected)
}

/// Admit the selected Query/Product basis before deciding whether this read
/// needs graph execution. Read and disclosure permission can use this custody
/// without rebuilding unrelated indexes on a Clean demand.
pub(super) fn admit_selected_permission_basis<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: WorthQueryProductObservationLease,
    application_basis: super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
) -> Result<WorthQueryApplicationQueryBasisCustody, WorthQueryApplicationQueryAdmissionDenial> {
    admit_selected_permission_basis_with_native_access(
        execution,
        application,
        product,
        application_basis,
        &mut super::super::ApplicationQueryNativeAccess::acquire(),
    )
}

pub(super) fn admit_selected_permission_basis_with_native_access<Schema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: WorthQueryProductObservationLease,
    application_basis: super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
    native: &mut super::super::ApplicationQueryNativeAccess<'_>,
) -> Result<WorthQueryApplicationQueryBasisCustody, WorthQueryApplicationQueryAdmissionDenial> {
    if product.observation().owner_identity() != application.product_runtime.owner.owner_identity()
    {
        return Err(admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
            "product World owner",
        ));
    }
    let graph = application.runtime.primary_graph().ok_or_else(|| {
        admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
            "primary graph",
        )
    })?;
    let live = native.with_runtime_mut(
        graph,
        execution,
        || {
            admission_denial(
                WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                "query native owner",
            )
        },
        |runtime| Ok(application_basis.is_live_at(runtime)),
    )?;
    if !live {
        return Err(admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
            "retained product snapshot",
        ));
    }
    Ok(WorthQueryApplicationQueryBasisCustody::new(
        product,
        application_basis,
    ))
}

pub(super) fn admit_retained<Schema: worth_query_installation::facade::ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: WorthQueryProductObservationLease,
) -> Result<WorthQueryApplicationQueryBasisCustody, WorthQueryApplicationQueryAdmissionDenial> {
    if product.observation().owner_identity() != application.product_runtime.owner.owner_identity()
    {
        return Err(admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
            "product World owner",
        ));
    }
    let mut application_basis = application
        .retain_product_application_basis(execution, product.observation())
        .map_err(map_product_admission_denial)?;
    let _ = application
        .bind_selected_program_interpretation(
            execution,
            product.relational_basis(),
            &mut application_basis,
        )
        .map_err(map_product_admission_denial)?;
    admit(execution, application, product, application_basis)
}

fn map_product_admission_denial(
    denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use crate::basis::WorthQueryProductBranchAdmissionDenial as Product;
    let kind = match denial {
        Product::Refused(AdmissionRefusal::ForeignOwner) => {
            WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis
        }
        Product::Refused(AdmissionRefusal::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        }) => WorthQueryApplicationQueryAdmissionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Product::Refused(AdmissionRefusal::RetentionCapacityExhausted) => {
            WorthQueryApplicationQueryAdmissionDenialKind::RetentionCapacityExhausted
        }
        Product::Refused(AdmissionRefusal::RetentionIdentityExhausted) => {
            WorthQueryApplicationQueryAdmissionDenialKind::RetentionIdentityExhausted
        }
        Product::Refused(AdmissionRefusal::SnapshotIdentityExhausted) => {
            WorthQueryApplicationQueryAdmissionDenialKind::SnapshotIdentityExhausted
        }
        Product::Refused(AdmissionRefusal::RetiredBranch)
        | Product::Refused(AdmissionRefusal::IncarnationChanged) => {
            WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis
        }
        Product::Stopped(stop) => match stop {
            Ok(kind) => WorthQueryApplicationQueryAdmissionDenialKind::ExecutionRequest(kind),
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled) => {
                WorthQueryApplicationQueryAdmissionDenialKind::Cancelled
            }
            Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut) => {
                WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded
            }
        },
        Product::Refused(_) => WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
    };
    admission_denial(
        kind,
        format!("retained product admission denied: {denial:?}"),
    )
}
