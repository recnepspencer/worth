//! Explicit native observation controls keep their original admitted demand.
use super::*;
/// Explicit native request for starting an assessment demand.
///
/// Use `retain_operation` on the ordinary assessment request to retain the
/// admitted demand and its cleanup across fresh calls instead.
pub struct WorthQueryAdvancedAssessmentRequest<'a, 'p, 's, S, Spec, D>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    request: WorthQueryWorkflowAssessmentDemandRequest<'a, 'p, 's, S, Spec, D>,
}
impl<'a, 'p, 's, S, Spec, D> WorthQueryWorkflowAssessmentDemandRequest<'a, 'p, 's, S, Spec, D>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub fn advanced(self) -> WorthQueryAdvancedAssessmentRequest<'a, 'p, 's, S, Spec, D> {
        WorthQueryAdvancedAssessmentRequest { request: self }
    }
}
impl<'a, 'p, 's, S, Spec, D> WorthQueryAdvancedAssessmentRequest<'a, 'p, 's, S, Spec, D>
where
    S: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
    Family<S, D>: WorthQueryWorkflowAssessmentOutputFamily<S>,
    SourceValue<S, D>: WorthQueryApplicationProjection<S, SourceQuery<S, D>> + Clone,
    <SourceBinding<S, D> as ApplicationQueryBinding<S>>::Input:
        ApplicationQueryIntent<S, Binding = SourceBinding<S, D>>,
    <SourceBinding<S, D> as ApplicationQueryBinding<S>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            S,
            <SourceBinding<S, D> as ApplicationQueryBinding<S>>::PrincipalIdentity,
        >,
{
    pub fn start(
        self,
    ) -> Result<
        WorthQueryWorkflowAssessmentDemandHandle<'a, S, Spec, D>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        self.request.start_native()
    }
}
/// Borrows an existing assessment handle for explicit native settlement.
///
/// Each `settle` uses a fresh request; observation does not approve or accept
/// the workflow operation and does not discharge native cleanup.
pub struct WorthQueryAdvancedAssessmentObservation<'b, 'a, S, Spec, D>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    observation: &'b mut WorthQueryWorkflowAssessmentDemandHandle<'a, S, Spec, D>,
}
impl<'a, S, Spec, D> WorthQueryWorkflowAssessmentDemandHandle<'a, S, Spec, D>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub fn advanced(&mut self) -> WorthQueryAdvancedAssessmentObservation<'_, 'a, S, Spec, D> {
        WorthQueryAdvancedAssessmentObservation { observation: self }
    }
}
impl<S, Spec, D> WorthQueryAdvancedAssessmentObservation<'_, '_, S, Spec, D>
where
    S: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
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
    pub fn settle(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryWorkflowAssessmentDemandProgress<SourceQuery<S, D>>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        self.observation.settle_native(request)
    }
}
