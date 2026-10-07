//! The ordered phases every application open runs.

mod backend_construction;
mod contribution_configuration;
mod graph_start;
mod lineage_restoration;
mod output_readmission;
mod package_admission;
mod publication;

use super::{
    home_opening::HomeStarted, open_plan::OpenPlan, open_refusal::OpenFailure,
    WorthQueryApplicationLimits,
};
use crate::domain_computation::primary_graph::{
    application_contribution::WorthQueryApplicationContributionTuple,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchemaComposition, ApplicationSchemaDeclaration,
};

pub(in crate::domain_computation::primary_graph) fn open_with_contributions<Schema, Contributions>(
    declaration: ApplicationSchemaDeclaration<Schema>,
    configuration: <Contributions as WorthQueryApplicationContributionTuple<Schema>>::Configuration,
    limits: WorthQueryApplicationLimits,
    plan: OpenPlan<'_, Schema>,
) -> Result<
    (
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        HomeStarted,
    ),
    OpenFailure,
>
where
    Schema: ApplicationSchemaComposition,
    Contributions: WorthQueryApplicationContributionTuple<Schema>,
{
    let OpenPlan {
        entry,
        start,
        authorization_time_source,
    } = plan;
    let admitted = package_admission::admit::<Schema, Contributions>(declaration, &limits, entry)?;
    let configured =
        contribution_configuration::configure::<Schema, Contributions>(admitted, configuration)?;
    let backend = backend_construction::construct(configured, &limits);
    let started = graph_start::start(backend, &limits, start)?;
    let published = publication::publish(started, &limits, authorization_time_source)?;
    let readmitted = output_readmission::readmit(published)?;
    lineage_restoration::restore(readmitted)
}
