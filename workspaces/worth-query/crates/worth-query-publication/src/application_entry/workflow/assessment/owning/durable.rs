//! Graceful assessment handoff retains DATA; recovery needs a fresh workflow request.
use super::*;
use worth_query_execution::facade::primary_graph::WorthQueryOriginalOutputDemand;

mod recorded;

pub struct WorthQueryOriginalWorkflowAssessment<S, Spec, D>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    required: RequiredWorkflowAssessment,
    original: WorthQueryOriginalOutputDemand,
    demand: D,
    controls: WorthQueryOutputDemandControls,
    marker: std::marker::PhantomData<fn() -> (S, Spec)>,
}

impl<S, Spec, D, P> WorthQueryWorkflowAssessmentOperation<S, Spec, D, P>
where
    S: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S>,
    P: ApplicationProgramDefinition<S>,
{
    /// On refusal the live observer, including its native cleanup custody, is returned.
    pub fn graceful_handoff(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryOriginalWorkflowAssessment<S, Spec, D>,
        (Self, WorthQueryApplicationOutputDemandDenial),
    > {
        if let Err(cause) = self.release_cleanup(request) {
            return Err((self, cause));
        }
        let original = self
            .runtime
            .runtime()
            .with_application_advancement(request.scope, |phase| {
                self.continuation
                    .admitted
                    .retain_for_reopen(self.runtime.runtime(), &phase)
                    .map_err(WorthQueryApplicationOutputDemandDenial::Demand)
            })
            .map_err(WorthQueryApplicationOutputDemandDenial::advancement)
            .and_then(|value| value);
        let original = match original {
            Ok(original) => original,
            Err(cause) => return Err((self, cause)),
        };
        self.continuation.close();
        Ok(WorthQueryOriginalWorkflowAssessment {
            required: self.required,
            original,
            demand: self.demand,
            controls: self.continuation.controls,
            marker: std::marker::PhantomData,
        })
    }
}

impl<'a, 'p, 's, S, Spec, D> WorthQueryWorkflowAssessmentDemandRequest<'a, 'p, 's, S, Spec, D>
where
    S: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    D: WorthQueryApplicationOutputDemand<S> + PartialEq,
    Family<S, D>: WorthQueryWorkflowAssessmentOutputFamily<S>,
    SourceValue<S, D>: WorthQueryApplicationProjection<S, SourceQuery<S, D>> + 'static,
    SourceQuery<S, D>: 'static,
{
    /// The required occurrence is prepared again by the real workflow binding.
    /// A failed readmission leaves the original DATA with its caller for retry.
    pub fn readmit_operation<P: ApplicationProgramDefinition<S>>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        original: &WorthQueryOriginalWorkflowAssessment<S, Spec, D>,
    ) -> Result<
        WorthQueryWorkflowAssessmentOperation<S, Spec, D, P>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if !std::ptr::eq(self.workflow.runtime(), program.runtime())
            || self.required != original.required
        {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let (admitted, demand) = self.demand.readmit_original_for_workflow(
            self.workflow,
            &original.original,
            &original.demand,
        )?;
        Ok(WorthQueryWorkflowAssessmentOperation {
            runtime: program.clone(),
            branch: self.branch,
            required: self.required,
            continuation: WorthQueryApplicationOutputDemandContinuation::new(
                admitted,
                original.controls,
            ),
            demand,
            marker: std::marker::PhantomData,
        })
    }
}
