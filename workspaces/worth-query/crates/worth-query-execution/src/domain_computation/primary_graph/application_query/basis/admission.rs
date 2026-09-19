#[cfg(test)]
use super::{map_index_currency_denial, map_registration_denial};
use crate::domain_computation::primary_graph::application_query::{
    controls::WorthQueryApplicationQueryBasis, WorthQueryApplicationQueryAdmissionDenial,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

pub(in crate::domain_computation::primary_graph::application_query) fn admit_application_query_basis<
    Schema,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    basis: WorthQueryApplicationQueryBasis,
) -> Result<super::WorthQueryApplicationQueryBasisCustody, WorthQueryApplicationQueryAdmissionDenial>
where
    Schema: ApplicationSchema,
{
    match basis {
        WorthQueryApplicationQueryBasis::Selected {
            product,
            application_basis,
        } => super::product_admission::admit(application, product, application_basis),
        WorthQueryApplicationQueryBasis::RetainedContinuation { product } => {
            super::product_admission::admit_retained(application, product)
        }
    }
}

#[cfg(test)]
pub(super) fn register_basis<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
) -> Result<
    super::super::resource_lifecycle::WorthQueryApplicationBasisLease,
    WorthQueryApplicationQueryAdmissionDenial,
>
where
    Schema: ApplicationSchema,
{
    let graph = application.primary_provider.graph.clone();
    graph
        .with_runtime_mut(|runtime| graph.ensure_primary_indexes_for_basis(runtime, &basis))
        .map_err(map_index_currency_denial)?;
    application
        .basis_leases
        .register(basis, graph)
        .map_err(map_registration_denial)
}

#[cfg(test)]
#[path = "admission/product_carriage.rs"]
mod product_carriage_tests;
