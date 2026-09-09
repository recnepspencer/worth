use super::{admission_denial, map_index_currency_denial, WorthQueryApplicationQueryBasisCustody};
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
    WorthQueryPrimaryGraphApplicationRuntime,
};

pub(super) fn admit<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    product: WorthQueryProductBranchLease,
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
    Ok(WorthQueryApplicationQueryBasisCustody::Product {
        product,
        application_basis,
    })
}
