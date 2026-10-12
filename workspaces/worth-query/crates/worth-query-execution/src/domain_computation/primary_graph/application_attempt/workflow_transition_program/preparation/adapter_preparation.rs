use super::*;

impl WorthQueryWorkflowAdvanceAdapter {
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowTransitionPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_advance::<Capability, Operation, Input, Scope, Spec>(
            execution, installed, instance, admission,
        )
    }

    pub fn prepare_navigate_back<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowTransitionPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_navigate_back::<Capability, Operation, Input, Scope, Spec>(
            execution, installed, instance, admission,
        )
    }

    pub fn prepare_collect_assessment<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowTransitionPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_collect_assessment::<Capability, Operation, Input, Scope, Spec>(
            execution, installed, instance, node_path, admission,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn prepare_approval<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
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
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowTransitionPreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_approval::<Capability, Operation, Input, Scope, Spec>(
            execution, workflow, instance, required, proposal, decision, admission,
        )
    }
}
