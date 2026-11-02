//! Generated endpoint payloads use the existing Catalog and maintained box Source.
use super::super::generated::source;
use super::super::schema::Catalog;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
#[cfg(test)]
mod court;
pub(super) mod operation;
pub(super) mod producer;
pub(in super::super) mod program;
mod readiness;
pub(in super::super) mod request;

pub(in super::super) fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    operation::declare(request::declare(builder))
}
pub(in super::super) fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<producer::Producer>()?;
    contracts.conditional::<readiness::Readiness>()?;
    Ok(())
}
pub(in super::super) fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<operation::Binding, _>(operation::Handler)?;
    setup.handler::<request::Binding, _>(request::Handler)?;
    setup.producer::<producer::Producer>(producer::Provider)?;
    setup.conditional::<readiness::Readiness>(())?;
    Ok(())
}
