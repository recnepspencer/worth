use super::*;

impl WorthQueryWorkflowAdvanceAdapter {
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        selected.prepare_workflow_advance::<Capability, Operation, Input, Scope, Spec>(
            installed, instance, admission,
        )
    }

    pub fn prepare_navigate_back<Schema, Capability, Operation, Input, Scope, Spec>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        selected.prepare_workflow_navigate_back::<Capability, Operation, Input, Scope, Spec>(
            installed, instance, admission,
        )
    }

    pub fn prepare_collect_assessment<Schema, Capability, Operation, Input, Scope, Spec>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        node_path: String,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        selected.prepare_workflow_collect_assessment::<Capability, Operation, Input, Scope, Spec>(
            installed, instance, node_path, admission,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_approval<Schema, Capability, Operation, Input, Scope, Spec>(
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        workflow: WorthQueryWorkflowVocabulary<'_, Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        required: &RequiredWorkflowApproval,
        proposal: &PublishedWorkflowProposalRef,
        decision: WorkflowApprovalDecision,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        selected.prepare_workflow_approval::<Capability, Operation, Input, Scope, Spec>(
            workflow, instance, required, proposal, decision, admission,
        )
    }
}
