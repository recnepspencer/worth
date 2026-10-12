use super::*;

impl WorthQueryWorkflowInstanceAdapter {
    pub fn prepare<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        published: PublishedWorkflowDefinitionRef,
        start_key_identity: [u8; 32],
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorkflowInstancePreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowInstancePreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_instance_start::<Capability, Operation, Input, Scope, Spec>(
            execution,
            installed,
            published,
            start_key_identity,
            admission,
        )
    }

    /// Prepares a successor on the current target definition.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_migration<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        source: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
        start_key_identity: [u8; 32],
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorkflowInstancePreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowInstancePreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_instance_migration::<Capability, Operation, Input, Scope, Spec>(
            execution,
            installed,
            WorkflowSuccession::Migration,
            source,
            target,
            resume_at,
            start_key_identity,
            admission,
        )
    }

    /// Prepares a successor for the selected fork copy.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_fork_continuation<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        source: PublishedWorkflowInstanceRef,
        target: PublishedWorkflowDefinitionRef,
        resume_at: &str,
        start_key_identity: [u8; 32],
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        WorkflowInstancePreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowInstancePreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected.prepare_workflow_instance_migration::<Capability, Operation, Input, Scope, Spec>(
            execution,
            installed,
            WorkflowSuccession::ForkContinuation,
            source,
            target,
            resume_at,
            start_key_identity,
            admission,
        )
    }

    /// Prepares cancellation on the selected branch.
    pub fn prepare_cancellation<Schema, Capability, Operation, Input, Scope, Spec>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        cancel_key_identity: [u8; 32],
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>,
        WorkflowInstancePreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let phase_request = match phase.execution_request_for(&selected.application().product_runtime) {
            Ok(request) => request,
            Err(_) => return Err(WorkflowInstancePreparationDenial::Attempt(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::new(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignPhase, "workflow-phase"))),
        };
        let execution = &phase_request;

        selected
            .prepare_workflow_instance_cancellation::<Capability, Operation, Input, Scope, Spec>(
                execution,
                installed,
                instance,
                cancel_key_identity,
                admission,
            )
    }

    pub fn compare_and_commit_cancellation<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowInstanceCancellation<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowInstanceCancellationOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request =
            match phase.execution_request_for(&runtime.product_runtime) {
                Ok(request) => request,
                Err(cause) => return WorkflowInstanceCancellationOutcome::Application(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
                        cause,
                    )
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit"),
                ),
            };
        let execution = &phase_request;

        runtime.compare_and_commit_workflow_instance_cancellation(
            execution,
            phase,
            prepared,
            idempotency,
        )
    }

    pub fn compare_and_commit<Schema, Operation, Input, Scope>(
        phase: &WorthQueryAdvancementPhase<'_>,

        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        prepared: PreparedWorkflowInstanceStart<Schema, Operation, Input, Scope>,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorkflowInstanceStartOutcome
    where
        Schema: ApplicationSchema,
        Operation: 'static,
        Input: Clone + Send + Sync + 'static,
    {
        let phase_request =
            match phase.execution_request_for(&runtime.product_runtime) {
                Ok(request) => request,
                Err(cause) => return WorkflowInstanceStartOutcome::Application(
                    crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::from(
                        cause,
                    )
                    .into_commit_outcome()
                    .landed()
                    .expect_err("a foreign phase cannot commit"),
                ),
            };
        let execution = &phase_request;

        runtime.compare_and_commit_workflow_instance_start(execution, phase, prepared, idempotency)
    }
}
