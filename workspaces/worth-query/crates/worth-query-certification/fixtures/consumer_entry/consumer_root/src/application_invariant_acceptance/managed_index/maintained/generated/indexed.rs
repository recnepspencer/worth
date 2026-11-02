//! Genuine managed Source producer, installed in the existing Catalog program.
#[cfg(test)]
mod court;
pub(super) mod operation;
mod producer;
pub(in super::super) mod program;
mod readiness;
pub(in super::super) mod request;
use super::super::schema::Catalog;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
pub(super) fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    builder
        .application_mutation_binding::<operation::Binding>()
        .application_mutation_binding::<request::Binding>()
}
pub(super) fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<producer::Producer>()?;
    contracts.conditional::<readiness::Readiness>()?;
    Ok(())
}
pub(super) fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<operation::Binding, _>(super::operation::Handler)?;
    setup.handler::<request::Binding, _>(super::source::operation::Handler)?;
    setup.producer::<producer::Producer>(producer::Provider)?;
    setup.conditional::<readiness::Readiness>(())?;
    Ok(())
}
