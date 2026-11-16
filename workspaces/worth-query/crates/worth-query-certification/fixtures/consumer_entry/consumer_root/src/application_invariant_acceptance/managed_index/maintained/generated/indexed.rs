//! Genuine managed Source producer, installed in the existing Catalog program.
#[cfg(test)]
mod applied_support;
pub(in super::super) mod authorized;
#[cfg(test)]
mod court;
pub(in super::super) mod nested_source;
pub(super) mod operation;
mod producer;
pub(in super::super) mod program;
mod readiness;
pub(in super::super) mod request;
#[cfg(test)]
mod request_journey;
use super::super::schema::Catalog;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
pub(super) fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    nested_source::declare(authorized::declare(builder))
        .application_mutation_binding::<operation::Binding>()
        .application_mutation_binding::<request::Binding>()
}
pub(super) fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<producer::Producer>()?;
    authorized::contracts(contracts)?;
    nested_source::contracts(contracts)?;
    contracts.conditional::<readiness::Readiness>()?;
    Ok(())
}
pub(super) fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<operation::Binding, _>(super::operation::Handler)?;
    authorized::configure(setup)?;
    nested_source::configure(setup)?;
    setup.handler::<request::Binding, _>(super::source::operation::Handler)?;
    setup.producer::<producer::Producer>(producer::Provider)?;
    setup.conditional::<readiness::Readiness>(())?;
    Ok(())
}
