//! Real generated Catalog rows use the same installed maintained Source.
use super::schema::Catalog;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
#[cfg(test)]
mod court;
pub(super) mod operation;
pub(super) mod producer;
pub(super) mod program;
mod readiness;
pub(super) mod source;

pub(super) fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    operation::declare(source::operation::declare(
        builder
            .application_query(source::definition())
            .application_query_binding::<source::Binding>(),
    ))
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
    setup.handler::<operation::Binding, _>(operation::Handler)?;
    setup.handler::<source::operation::Binding, _>(source::operation::Handler)?;
    setup.producer::<producer::Producer>(producer::Provider)?;
    setup.conditional::<readiness::Readiness>(())?;
    Ok(())
}
