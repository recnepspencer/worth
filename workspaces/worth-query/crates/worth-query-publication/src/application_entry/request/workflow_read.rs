//! Principal-bearing workflow observation pinned to an explicit retained data basis.
use super::WorthQueryApplicationRetainedRequest;
use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowReadBinding, ApplicationWorkflowSpec,
};
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    application_discovery::{WorthQueryWorkflowInstanceRead, WorthQueryWorkflowReadDenial},
    application_installation::WorthQueryWorkflowVocabulary,
    primary_graph::WorthQueryProductBranchAdmissionDenial,
};
use worth_query_execution::publication_boundary::workflow_instance::WorthQueryWorkflowInstanceDiscoveryIntent;
use worth_query_installation::facade::ApplicationSchema;

/// Technical and permission refusals carry no target existence or lifecycle payload.
#[derive(Debug)]
pub enum WorthQueryWorkflowObservationDenial {
    ExecutionRequest(WorthQueryAdvancementDenial),
    ProductSelection(WorthQueryProductBranchAdmissionDenial),
    Read(WorthQueryWorkflowReadDenial),
}
impl<Schema: ApplicationSchema> WorthQueryApplicationRetainedRequest<'_, '_, '_, Schema> {
    /// Locates the original native start at this exact retained observation.
    /// Fresh subject and audit admission use the current branch in the same host call.
    /// Actor permission failures are explicit withholding; technical failures remain errors.
    /// An installed recorded-command actor requirement cannot substitute for a read context:
    /// resolving such a read binding refuses rather than borrowing Advance or Close authority.
    pub fn read_original_workflow<'workflow, Spec, Subject, Audit>(
        &self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        original: &WorthQueryWorkflowInstanceDiscoveryIntent<Schema>,
        input: Subject::Input,
        audit_input: Audit::Input,
    ) -> Result<WorthQueryWorkflowInstanceRead, WorthQueryWorkflowObservationDenial>
    where
        Schema: 'workflow,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Subject: ApplicationWorkflowReadBinding<Schema>,
        Audit: ApplicationWorkflowReadBinding<Schema>,
    {
        use WorthQueryWorkflowObservationDenial as Denial;
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
                    .read_original_workflow::<Spec, Subject, Audit>(
                        &data,
                        &phase,
                        &workflow,
                        self.principal,
                        input,
                        audit_input,
                        self.scope,
                        original,
                    )
                    .map_err(Denial::Read)
            })
            .map_err(Denial::ExecutionRequest)?
    }
}

impl<Schema: ApplicationSchema> WorthQueryApplicationRetainedRequest<'_, '_, '_, Schema> {
    /// Discovers exactly one definition's metadata after its independently declared read gate.
    /// ```
    /// use worth_query_declaration::facade::application_program::{ApplicationWorkflowSpec,ApplicationWorkflowReadBinding,ApplicationWorkflowDefinitionReadTarget};
    /// use worth_query_installation::facade::ApplicationSchema;
    /// use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;
    /// use worth_query_publication::facade::application_entry::WorthQueryApplicationRetainedRequest;
    /// fn admitted<'w,S,W,B>(read:&WorthQueryApplicationRetainedRequest<'_, '_, '_,S>,workflow:WorthQueryWorkflowVocabulary<'w,S,W>,target:&ApplicationWorkflowDefinitionReadTarget<W>,input:B::Input)
    /// where S:ApplicationSchema+'w,W:ApplicationWorkflowSpec<Schema=S>,B:ApplicationWorkflowReadBinding<S> {
    ///     let _ = read.discover_workflow_definition::<W,B>(workflow,target,input);
    /// }
    /// ```
    /// This does not disclose instances or borrow a subject/audit grant.
    pub fn discover_workflow_definition<'workflow, Spec, B>(
        &self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec>>,
        target:&worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionReadTarget<Spec>,
        input: B::Input,
    ) -> Result<
        worth_query_execution::facade::application_discovery::WorthQueryWorkflowDefinitionRead,
        WorthQueryWorkflowObservationDenial,
    >
    where
        Schema: 'workflow,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        B: ApplicationWorkflowReadBinding<Schema>,
    {
        use WorthQueryWorkflowObservationDenial as Denial;
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
                    .discover_workflow_definition::<Spec, B>(
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
}
