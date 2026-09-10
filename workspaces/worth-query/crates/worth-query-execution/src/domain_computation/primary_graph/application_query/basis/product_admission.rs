use super::{admission_denial, map_index_currency_denial, WorthQueryApplicationQueryBasisCustody};
use crate::basis::WorthQueryProductObservationLease;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) fn admit<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: WorthQueryProductObservationLease,
    application_basis: super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
) -> Result<WorthQueryApplicationQueryBasisCustody, WorthQueryApplicationQueryAdmissionDenial> {
    if product.observation().owner_identity() != application.product_runtime.owner.owner_identity()
    {
        return Err(admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
            "product World owner",
        ));
    }
    if !application_basis.is_live() {
        return Err(admission_denial(
            WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
            "retained product snapshot",
        ));
    }
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
        .with_runtime_mut(|runtime| {
            graph.ensure_primary_indexes_for_basis(runtime, product.relational_basis())
        })
        .map_err(map_index_currency_denial)?;
    Ok(WorthQueryApplicationQueryBasisCustody::new(
        product,
        application_basis,
    ))
}

pub(super) fn admit_retained<Schema>(
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
    let application_basis = application
        .retain_product_application_basis(product.observation())
        .map_err(map_product_admission_denial)?;
    admit(application, product, application_basis)
}

fn map_product_admission_denial(
    denial: crate::basis::WorthQueryProductBranchAdmissionDenial,
) -> WorthQueryApplicationQueryAdmissionDenial {
    use crate::basis::WorthQueryProductBranchAdmissionDenial as Product;
    let kind = match denial {
        Product::ForeignOwner => WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
        Product::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => WorthQueryApplicationQueryAdmissionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Product::RetentionCapacityExhausted => {
            WorthQueryApplicationQueryAdmissionDenialKind::RetentionCapacityExhausted
        }
        Product::RetentionIdentityExhausted => {
            WorthQueryApplicationQueryAdmissionDenialKind::RetentionIdentityExhausted
        }
        Product::SnapshotIdentityExhausted => {
            WorthQueryApplicationQueryAdmissionDenialKind::SnapshotIdentityExhausted
        }
        Product::RetiredBranch | Product::IncarnationChanged => {
            WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis
        }
        _ => WorthQueryApplicationQueryAdmissionDenialKind::BasisUnavailable,
    };
    admission_denial(
        kind,
        format!("retained product admission denied: {denial:?}"),
    )
}
