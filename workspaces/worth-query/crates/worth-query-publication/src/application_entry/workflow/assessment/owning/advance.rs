//! Fresh assessment disclosure and native demand advancement share one admitted phase.
use super::*;
impl<S, Spec, D, P> WorthQueryWorkflowAssessmentOperation<S, Spec, D, P>
where
    S: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
    P: ApplicationProgramDefinition<S>,
    Family<S, D>: WorthQueryWorkflowAssessmentOutputFamily<S>,
    SourceValue<S, D>: 'static + WorthQueryApplicationProjection<S, SourceQuery<S, D>> + Clone,
    SourceQuery<S, D>: 'static,
    <SourceBinding<S, D> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = SourceBinding<S, D>>,
    <SourceBinding<S, D> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <SourceBinding<S, D> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
{
    pub fn advance(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryWorkflowAssessmentDemandProgress<SourceQuery<S, D>>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || request.branch != self.branch
        {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let keepalive = self.runtime.clone();
        keepalive
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                self.advance_in_advancement(&phase, request)
            })
            .map_err(WorthQueryApplicationOutputDemandDenial::advancement)?
    }
    fn advance_in_advancement(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryWorkflowAssessmentDemandProgress<SourceQuery<S, D>>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        super::super::advance_assessment_in_advancement(
            self.runtime.runtime(),
            self.branch,
            phase,
            &mut self.continuation.admitted,
            &self.demand,
            self.continuation.controls,
            &self.required,
            request,
        )
    }
}
