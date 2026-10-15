//! Exact-data history and reacquisition through the same fresh disclosure owner.
use super::{
    workflow_read::WorthQueryWorkflowObservationDenial as Denial,
    WorthQueryApplicationRetainedRequest,
};
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowReadBinding, ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_discovery::{
        WorthQueryWorkflowHistoryPage, WorthQueryWorkflowIndexRead, WorthQueryWorkflowReadDenial,
        WorthQueryWorkflowReadTarget,
    },
    application_installation::WorthQueryWorkflowVocabulary,
};
use worth_query_installation::facade::ApplicationSchema;
impl<S: ApplicationSchema> WorthQueryApplicationRetainedRequest<'_, '_, '_, S> {
    /// Fresh current subject and independent audit admission for each exact-basis page.
    /// Proposal locators are native references, not evidence that any command is legal.
    /// ```
    /// use worth_query_declaration::facade::application_program::{ApplicationWorkflowSpec,ApplicationWorkflowReadBinding};
    /// use worth_query_installation::facade::ApplicationSchema;
    /// use worth_query_execution::facade::{application_discovery::{WorthQueryWorkflowReadTarget,WorthQueryWorkflowIndexRead},application_installation::WorthQueryWorkflowVocabulary};
    /// use worth_query_publication::facade::application_entry::WorthQueryApplicationRetainedRequest;
    /// fn read<'w,S,W,B,A>(request:&WorthQueryApplicationRetainedRequest<'_, '_, '_,S>,workflow:WorthQueryWorkflowVocabulary<'w,S,W>,target:&WorthQueryWorkflowReadTarget,input:B::Input,audit:A::Input)
    /// where S:ApplicationSchema+'w,W:ApplicationWorkflowSpec<Schema=S>,B:ApplicationWorkflowReadBinding<S>,A:ApplicationWorkflowReadBinding<S> {
    ///     let _=request.read_workflow_history::<W,B,A>(workflow,target,input,audit,16,WorthQueryWorkflowIndexRead::PublishedOnly,None);
    /// }
    /// ```
    pub fn read_workflow_history<'w, W, Subject, Audit>(
        &self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'w, S, W>>,
        target: &WorthQueryWorkflowReadTarget,
        input: Subject::Input,
        audit_input: Audit::Input,
        width: usize,
        indexes: WorthQueryWorkflowIndexRead,
        after: Option<&WorthQueryWorkflowHistoryPage>,
    ) -> Result<WorthQueryWorkflowHistoryPage, Denial>
    where
        S: 'w,
        W: ApplicationWorkflowSpec<Schema = S>,
        Subject: ApplicationWorkflowReadBinding<S>,
        Audit: ApplicationWorkflowReadBinding<S>,
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
                    .read_workflow_history::<W, Subject, Audit>(
                        &data,
                        &phase,
                        &workflow,
                        self.principal,
                        input,
                        audit_input,
                        self.scope,
                        target,
                        width,
                        indexes,
                        after,
                    )
                    .map_err(Denial::Read)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
