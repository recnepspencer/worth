//! Principal-bearing definition/subject pages at the explicit retained basis.
use super::workflow_read::WorthQueryWorkflowObservationDenial as Denial;
use super::WorthQueryApplicationRetainedRequest;
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowDefinitionReadTarget, ApplicationWorkflowReadBinding,
    ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_discovery::{
        WorthQueryWorkflowIndexRead, WorthQueryWorkflowInstancePage, WorthQueryWorkflowReadDenial,
        WorthQueryWorkflowReadExtent,
    },
    application_installation::WorthQueryWorkflowVocabulary,
};
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema> WorthQueryApplicationRetainedRequest<'_, '_, '_, S> {
    /// Returns a bounded scoped page, not a complete program/adoption inventory.
    /// Each continuation re-admits current disclosure; the token retains only exact data custody.
    pub fn read_workflow_instances<'w, 'target, W, B>(
        &self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'w, S, W>>,
        target: &'target ApplicationWorkflowDefinitionReadTarget<W>,
        input: B::Input,
        extent: WorthQueryWorkflowReadExtent,
        width: usize,
        indexes: WorthQueryWorkflowIndexRead,
        after: Option<&WorthQueryWorkflowInstancePage<'_, S, W, B>>,
    ) -> Result<WorthQueryWorkflowInstancePage<'target, S, W, B>, Denial>
    where
        S: 'w,
        W: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationWorkflowReadBinding<S>,
    {
        let workflow = workflow.into();
        if !std::ptr::eq(workflow.runtime(), self.application) {
            return Err(Denial::Read(WorthQueryWorkflowReadDenial::PolicyMismatch));
        }
        self.application
            .with_application_advancement(self.scope, |phase| {
                let current = self
                    .application
                    .on_branch(self.branch)
                    .select(&phase)
                    .map_err(Denial::ProductSelection)?;
                let data = self
                    .application
                    .select_application_read_observation(&phase, &self.observation)
                    .map_err(Denial::ProductSelection)?;
                current
                    .read_workflow_instances::<W, B>(
                        &data,
                        &phase,
                        &workflow,
                        self.principal,
                        input,
                        self.scope,
                        target,
                        extent,
                        width,
                        indexes,
                        after,
                    )
                    .map_err(Denial::Read)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
