//! A retained history demand reenters only through fresh current-control custody.
use super::*;
mod dependent;
mod discovery;
mod permission;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedHistoryProgramOutput, WorthQueryApplicationProjection,
    WorthQueryHistoryOutputAdvanceDenial, WorthQueryHistoryOutputAuthority,
    WorthQueryHistoryProgramOutputAdvance, WorthQuerySettledHistoryProgramOutput,
};

impl<S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>>
    WorthQueryProgramApplicationRuntime<S, P>
{
    pub fn advance_history_program_output<D>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedHistoryProgramOutput<S, P, D>,
        authority: &WorthQueryHistoryOutputAuthority<'_, '_>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<S>,
        request: &WorthQueryRequestScope,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<WorthQueryHistoryProgramOutputAdvance<S, P, D>, WorthQueryHistoryOutputAdvanceDenial>
    where
        D: WorthQueryApplicationOutputDemand<S>,
        SourceValue<S, D>: WorthQueryApplicationProjection<S, SourceQuery<S, D>> + 'static,
        SourceQuery<S, D>: 'static,
    {
        use WorthQueryHistoryOutputAdvanceDenial as Denial;
        let runtime = self.runtime();
        self.validate_history_demand(phase, demand, authority, request, branch)?;
        authority
            .confirm_read(phase, runtime, request, ())
            .map_err(Denial::Authority)?;
        let execution = phase.execution_request_for(&runtime.runtime.product_runtime).map_err(|cause| {
            Denial::Demand(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind::ExecutionRequest(cause.into()),
                "history output advancement phase",
            ))
        })?;
        let admitted = &mut demand.admitted;
        let progress = runtime.advance_history_output_demand(
            &execution,
            phase,
            &mut admitted.admitted,
            principal,
            request,
            branch,
            authority.producer_invocation(),
        );
        // The actual demand retains partial work and cleanup while delivery is
        // withheld. Do not move that cleanup into a diagnostic before this seal.
        let progress = authority
            .confirm_read(phase, runtime, request, progress)
            .map_err(Denial::Authority)?;
        if let Ok(WorthQueryOutputDemandAdvance::Settled(retained)) = &progress
            && let Some(completion) = &admitted.original_root_completion
        {
            completion.completed(retained);
        }
        progress
            .map_err(|mut stop| {
                if let Some(cleanup) = admitted.admitted.take_read_cleanup() {
                    stop = stop.with_read_cleanup(cleanup);
                }
                Denial::Demand(stop)
            })
            .map(|progress| match progress {
                WorthQueryOutputDemandAdvance::Pending => {
                    WorthQueryHistoryProgramOutputAdvance::Pending
                }
                WorthQueryOutputDemandAdvance::Progressed => {
                    WorthQueryHistoryProgramOutputAdvance::Progressed
                }
                WorthQueryOutputDemandAdvance::Settled(retained) => {
                    WorthQueryHistoryProgramOutputAdvance::Settled(
                        WorthQuerySettledHistoryProgramOutput {
                            settled: WorthQuerySettledProgramOutput {
                                retained,
                                target_feature: admitted.target_feature,
                                artifact: admitted.artifact,
                                marker: std::marker::PhantomData,
                            },
                            profile: demand.profile.clone(),
                            original: std::sync::Arc::clone(&demand.original),
                        },
                    )
                }
            })
    }
}

impl<S, P, D> WorthQueryAdmittedHistoryProgramOutput<S, P, D>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    /// Original disclosed query description; this grants no read or advancement.
    pub fn observed_source(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryObservedSource<SourceQuery<S, D>>
    {
        self.admitted.observed_source()
    }

    pub fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOutputReadCloseDenial> {
        self.admitted
            .close_read_request(phase, access, runtime, branch)
    }

    pub fn close(&mut self) {
        self.admitted.close();
    }
}
