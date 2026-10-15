//! Fresh native cleanup discharge precedes closing this observer only.
use super::*;
impl<S, Spec, D, P> WorthQueryWorkflowAssessmentOperation<S, Spec, D, P>
where
    S: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        Option<
            worth_query_execution::facade::domain_computation::WorthQueryDirectRunCleanupReceipt,
        >,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if request.branch != self.branch {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        self.continuation
            .release_managed_cleanup(self.runtime.runtime(), request)
    }
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryClosedAssessmentObservation<S, D, P>,
        (Self, WorthQueryApplicationOutputDemandDenial),
    > {
        match self.release_cleanup(request) {
            Ok(cleanup) => {
                self.continuation.close();
                Ok(WorthQueryClosedAssessmentObservation {
                    cleanup,
                    runtime: self.runtime,
                    observation: self.continuation,
                })
            }
            Err(cause) => Err((self, cause)),
        }
    }
}
