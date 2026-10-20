//! One isolated installed counted producer; no huge input allocation.
use super::*;
use worth_query_host::facade::{application_contribution::*, declaration::application_schema::*};
mod composition;
mod installation;
mod journey;
mod operation;
mod output;
mod producer;
mod program;
mod readiness;
mod source;
use composition::{QuoteContribution, QuoteSchema};
use operation::*;
pub(crate) use producer::Producer;
use source::*;

pub(crate) fn declare<S: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let schema = schema
        .aspect(Body::reference(), QuoteSourceFacts::reference())
        .aspect(Body::reference(), QuoteOutputFacts::reference())
        .field(Body::reference(), RequestedRows::reference())
        .field(Body::reference(), CountedTotal::reference())
        .application_query(source::definition::<S>())
        .application_query_binding::<ReadBinding<S>>()
        .application_query(output::definition::<S>())
        .application_query_binding::<output::OutputReadBinding<S>>();
    operation::declare(schema)
}

pub(crate) fn configure<S: TopologySchemaBinding>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<ProduceBinding<S>, _>(Handler)?;
    setup.handler::<EditBinding<S>, _>(Handler)?;
    setup.producer::<Producer<S>>(producer::Provider)?;
    setup.conditional::<readiness::Readiness<S>>(())
}

pub(crate) fn contracts<S: TopologySchemaBinding>(
    contracts: &mut WorthQueryApplicationContributionContracts<S>,
) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
    contracts.producer::<Producer<S>>()?;
    contracts.conditional::<readiness::Readiness<S>>()?;
    Ok(())
}
