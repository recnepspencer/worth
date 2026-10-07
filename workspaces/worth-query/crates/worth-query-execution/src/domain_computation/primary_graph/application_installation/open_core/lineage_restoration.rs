//! Restore lineage only after retained outputs have been readmitted.
use super::super::{
    checkpoint_lineage, home_opening::HomeStarted, open_refusal::OpenFailure,
    WorthQueryApplicationOpenDenial as Denial,
};
use super::output_readmission::ReadmittedInstallation;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

pub(super) fn restore<Schema: ApplicationSchema>(
    installation: ReadmittedInstallation<Schema>,
) -> Result<
    (
        WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        HomeStarted,
    ),
    OpenFailure,
> {
    let ReadmittedInstallation {
        application,
        started,
    } = installation;
    match checkpoint_lineage::restore(&application) {
        Ok(()) => Ok((application, started)),
        Err(denial) => Err(started.refused(Denial::Graph(denial))),
    }
}
