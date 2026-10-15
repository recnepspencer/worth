//! Assessment ownership transfers the admitted demand, never its diagnostic denial.
use super::*;
use crate::application_entry::WorthQueryApplicationOutputDemandContinuation;
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
mod advance;
mod close;
/// Owns an assessment's original admitted demand and native cleanup custody.
///
/// Call `advance` with a fresh request for the same runtime and branch. Close
/// this observation independently; observing evidence is not workflow approval.
#[must_use = "retain the actual assessment observation until lawful close"]
pub struct WorthQueryWorkflowAssessmentOperation<S, Spec, D, P>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
    P: ApplicationProgramDefinition<S>,
{
    runtime: WorthQueryProgramApplicationRuntime<S, P>,
    branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    required: RequiredWorkflowAssessment,
    continuation: WorthQueryApplicationOutputDemandContinuation<S, D>,
    demand: D,
    marker: std::marker::PhantomData<fn() -> Spec>,
}
/// Closing observer interest and releasing failed producer custody proves neither approval nor business completion.
pub struct WorthQueryClosedAssessmentObservation<S, D, P>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S>,
    P: ApplicationProgramDefinition<S>,
{
    runtime: WorthQueryProgramApplicationRuntime<S, P>,
    observation: WorthQueryApplicationOutputDemandContinuation<S, D>,
    cleanup: Option<
        worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt,
    >,
}
impl<S, D, P> WorthQueryClosedAssessmentObservation<S, D, P>
where
    S: ApplicationSchema,
    D: WorthQueryApplicationOutputDemand<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn runtime(&self) -> &WorthQueryProgramApplicationRuntime<S, P> {
        &self.runtime
    }
    pub fn cleanup_receipt(
        &self,
    ) -> Option<&worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt>
    {
        self.cleanup.as_ref()
    }
}
impl<'a, 'p, 's, S, Spec, D> WorthQueryWorkflowAssessmentDemandRequest<'a, 'p, 's, S, Spec, D>
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
    pub fn retain_operation<P: ApplicationProgramDefinition<S>>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        WorthQueryWorkflowAssessmentOperation<S, Spec, D, P>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if !std::ptr::eq(self.workflow.runtime(), program.runtime()) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let handle = self.start_native()?;
        Ok(WorthQueryWorkflowAssessmentOperation {
            runtime: program.clone(),
            branch: handle.branch,
            required: handle.required,
            continuation: WorthQueryApplicationOutputDemandContinuation::new(
                handle.admitted,
                handle.controls,
            ),
            demand: handle.demand,
            marker: std::marker::PhantomData,
        })
    }
}
impl<S, Spec, D, P> WorthQueryWorkflowAssessmentOperation<S, Spec, D, P>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.branch
    }
    pub fn required(&self) -> &RequiredWorkflowAssessment {
        &self.required
    }
    pub fn notifications(
        &self,
    ) -> Result<WorthQueryOutputDemandNotifications, WorthQueryApplicationOutputDemandDenial> {
        self.continuation.notifications()
    }
    pub fn managed_cleanup_failure(
        &mut self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.continuation.managed_cleanup_failure()
    }
}
