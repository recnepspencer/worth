//! History demand advancement consumes fresh Native control on every call.
use super::*;
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryAdmittedHistoryProgramOutput, WorthQueryHistoryOutputAuthority,
    WorthQueryHistoryProgramOutputAdvance,
};

impl<S, P, D> WorthQueryApplicationProgramDemandHandle<S, P, D>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
    SourceValue<S, D>: WorthQueryApplicationProjection<S, SourceQuery<S, D>> + Clone,
    <Source<S, D> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = Source<S, D>>,
    <Source<S, D> as ApplicationQueryBinding<S>>::ScopeBinding: ApplicationQueryScopeResolution<
        S,
        <Source<S, D> as ApplicationQueryBinding<S>>::PrincipalIdentity,
    >,
{
    pub(in crate::application_entry) fn authorize_in_history(
        &self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, S>,
        history: crate::application_entry::mutation::program_output_continuation::ProgramOutputHistoryFrame<'_, '_, '_>,
    ) -> Result<(), WorthQueryApplicationOutputDemandDenial> {
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let NativeProgramDemandCustody::History(admitted) = &self.admitted else {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        };
        application
            .authorize_history_program_output(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                admitted,
                history.authority,
                request.principal,
                request.scope,
                history.branch,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::HistoryRead)
    }

    pub(in crate::application_entry) fn advance_with_frame(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, S>,
        history: Option<crate::application_entry::mutation::program_output_continuation::ProgramOutputHistoryFrame<'_, '_, '_>>,
    ) -> Result<
        WorthQueryApplicationProgramDemandProgress<S, P, D>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        match history {
            Some(history) => self.advance_in_history(
                phase,
                application,
                request,
                history.authority,
                history.branch,
            ),
            None => self.advance(phase, application, request),
        }
    }

    pub(in crate::application_entry) fn new_history(
        admitted: WorthQueryAdmittedHistoryProgramOutput<S, P, D>,
        demand: D,
        original: std::sync::Arc<
            worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
        >,
    ) -> Self {
        Self {
            admitted: NativeProgramDemandCustody::History(admitted),
            demand,
            source_observation: Some(original),
        }
    }

    pub(in crate::application_entry) fn advance_in_history(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, S>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<
        WorthQueryApplicationProgramDemandProgress<S, P, D>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let NativeProgramDemandCustody::History(admitted) = &mut self.admitted else {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        };
        match application
            .advance_history_program_output(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                admitted,
                authority,
                request.principal,
                request.scope,
                branch,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::History)?
        {
            WorthQueryHistoryProgramOutputAdvance::Pending => {
                Ok(WorthQueryApplicationProgramDemandProgress::Pending)
            }
            WorthQueryHistoryProgramOutputAdvance::Progressed => {
                Ok(WorthQueryApplicationProgramDemandProgress::Progressed)
            }
            WorthQueryHistoryProgramOutputAdvance::Settled(authority) => {
                let settlement = WorthQueryApplicationOutputDemandSettlement::new(
                    authority.retained(),
                    admitted.observed_source().clone(),
                );
                Ok(WorthQueryApplicationProgramDemandProgress::Settled {
                    settlement,
                    authority: ProgramOutputParentAuthority::history(authority),
                })
            }
        }
    }
}
