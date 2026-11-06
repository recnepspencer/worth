//! Actual admitted Native owners, with no authority-erasing conversion.
use super::*;
use worth_query_execution::publication_boundary::document_history::WorthQueryAdmittedHistoryProgramOutput;

pub(super) enum NativeProgramDemandCustody<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    Ordinary(WorthQueryAdmittedProgramOutput<S, P, D>),
    History(WorthQueryAdmittedHistoryProgramOutput<S, P, D>),
}

impl<S, P, D> NativeProgramDemandCustody<S, P, D>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub(super) fn ordinary(
        &self,
    ) -> Result<&WorthQueryAdmittedProgramOutput<S, P, D>, WorthQueryApplicationOutputDemandDenial>
    {
        match self {
            Self::Ordinary(value) => Ok(value),
            Self::History(_) => Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch),
        }
    }
    pub(super) fn ordinary_mut(
        &mut self,
    ) -> Result<
        &mut WorthQueryAdmittedProgramOutput<S, P, D>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        match self {
            Self::Ordinary(value) => Ok(value),
            Self::History(_) => Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch),
        }
    }
    pub(super) fn observed_source(
        &self,
    ) -> &worth_query_execution::facade::primary_graph::WorthQueryObservedSource<SourceQuery<S, D>>
    {
        match self {
            Self::Ordinary(value) => value.observed_source(),
            Self::History(value) => value.observed_source(),
        }
    }
    pub(super) fn close_read_request(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), worth_query_execution::facade::primary_graph::WorthQueryOutputReadCloseDenial>
    {
        let access = worth_query_execution::publication_boundary::program_publication_access();
        match self {
            Self::Ordinary(value) => value.close_read_request(phase, &access, application, branch),
            Self::History(value) => value.close_read_request(phase, &access, application, branch),
        }
    }
}
