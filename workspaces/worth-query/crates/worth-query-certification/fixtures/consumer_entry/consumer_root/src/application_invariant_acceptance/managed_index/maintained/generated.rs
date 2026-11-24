//! Real generated Catalog rows use the same installed maintained Source.
use super::schema::Catalog;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::{application_contribution::*, primary_graph::*};
#[cfg(test)]
mod court;
pub(super) mod indexed;
pub(super) mod operation;
pub(super) mod producer;
pub(super) mod program;
mod readiness;
pub(super) mod source;

pub(super) fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    indexed::declare(operation::declare(source::operation::declare(
        builder
            .application_query(source::definition())
            .application_query_binding::<source::Binding>(),
    )))
}
pub(super) fn contracts(
    contracts: &mut WorthQueryApplicationContributionContracts<Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    indexed::contracts(contracts)?;
    if producer::SOURCE_OWNED.get() {
        contracts.producer::<producer::OwnedProducer>()?;
    } else {
        contracts.producer::<producer::Producer>()?;
    }
    contracts.conditional::<readiness::Readiness>()?;
    Ok(())
}
pub(super) fn configure(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Catalog>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    indexed::configure(setup)?;
    let computation = setup.partitioned_computation::<
        super::host::Feature,
        operation::computation::Computation,
        _,
    >(operation::computation::Owner)?;
    setup.handler::<operation::Binding, _>(operation::ComputingHandler(computation))?;
    setup.handler::<source::operation::Binding, _>(source::operation::Handler)?;
    if producer::SOURCE_OWNED.get() {
        setup.producer::<producer::OwnedProducer>(producer::Provider)?;
    } else {
        setup.producer::<producer::Producer>(producer::Provider)?;
    }
    setup.conditional::<readiness::Readiness>(())?;
    Ok(())
}
