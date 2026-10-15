//! Existing-envelope workflow marks with fresh disclosure on every receiving call.
use super::{
    workflow_read::WorthQueryWorkflowObservationDenial as Denial,
    WorthQueryApplicationRetainedRequest,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowReadBinding, ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_discovery::{
        WorthQueryWorkflowHistoryMark, WorthQueryWorkflowHistorySubscription,
        WorthQueryWorkflowReadDenial, WorthQueryWorkflowReadTarget,
    },
    application_installation::WorthQueryWorkflowVocabulary,
};
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema> WorthQueryApplicationRetainedRequest<'_, '_, '_, S> {
    /// Watches dependencies through native envelope custody; no command authority is produced.
    pub fn subscribe_workflow_history<'w, W, B>(
        &self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'w, S, W>>,
        target: &WorthQueryWorkflowReadTarget,
        input: B::Input,
    ) -> Result<WorthQueryWorkflowHistorySubscription, Denial>
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
                    .subscribe_workflow_history::<W, B>(
                        &data,
                        &phase,
                        &workflow,
                        self.principal,
                        input,
                        self.scope,
                        target,
                    )
                    .map_err(Denial::Read)
            })
            .map_err(Denial::ExecutionRequest)?
    }
    /// A refused poll retains the subscription, target and all unacknowledged native positions.
    pub fn poll_workflow_history<'w, W, B>(
        &self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'w, S, W>>,
        subscription: &mut WorthQueryWorkflowHistorySubscription,
        input: B::Input,
    ) -> Result<WorthQueryWorkflowHistoryMark, Denial>
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
                    .poll_workflow_history::<W, B>(
                        &data,
                        &phase,
                        &workflow,
                        self.principal,
                        input,
                        self.scope,
                        subscription,
                    )
                    .map_err(Denial::Read)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
