use super::{
    PreparedWorkflowAdvance, WorkflowLocalActionKind, WorkflowTransitionPreparationDenial,
    WorkflowTransitionRequestKind, WorthQueryWorkflowAdvanceAdapter,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{
        PublishedWorkflowInstanceRef, WorthQueryApplicationAttemptDenial,
        WorthQueryApplicationAttemptDenialKind,
    },
    workflow::instance::SelectedWorkflowOperation,
    WorthQueryAdmittedApplicationOperation, WorthQuerySelectedProductOperation,
};
use worth_query_declaration::facade::{
    application_capability::{
        ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRef, ApplicationCapabilityRequest,
    },
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationScopeBinding,
    },
    application_program::ApplicationWorkflowSpec,
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef,
        ApplicationStructuredValueBinding,
    },
};
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationWorkflowSpec,
};

impl WorthQueryWorkflowAdvanceAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_local_action<Schema, Capability, Operation, Input, Scope, Spec, Binding>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        kind: WorkflowLocalActionKind,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorkflowTransitionPreparationDenial,
    >
    where
        Schema: ApplicationSchema,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>
            + ApplicationCapabilityMutationBinding<Schema, Capability = Capability>,
        Input: ApplicationCapabilityRequest<
                Schema,
                Capability,
                Scope = <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            > + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
    {
        let deny = |subject| {
            WorkflowTransitionPreparationDenial::Attempt(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                subject,
            ))
        };
        validate_action_admission::<Schema, Spec, Binding, Scope>(selected, installed, &admission)?;
        let input_identity = admission
            .governed_input_identity()
            .copied()
            .ok_or_else(|| deny("workflow action requires admitted input provenance"))?;
        let execution = phase
            .execution_request_for(&selected.application().product_runtime)
            .map_err(|_| deny("foreign workflow action preparation phase"))?;
        let requested = SelectedWorkflowOperation {
            operation: Operation::IDENTIFIER.to_owned(),
            input_type: Binding::InputBinding::IDENTITY.as_str().to_owned(),
            binding: Some(Binding::IDENTITY.to_owned()),
            workflow_execution_requirements: Binding::workflow_execution_requirements(),
        };
        selected.prepare_workflow_transition::<Capability, Operation, Input, Scope, Spec>(
            &execution,
            installed,
            instance,
            admission,
            WorkflowTransitionRequestKind::LocalAction {
                kind,
                requested,
                input_identity,
                completion: None,
            },
        )
    }
}

/// Fresh operation capability correspondence, shared by first execution and completion.
pub(super) fn validate_action_admission<S, Spec, B, Scope>(
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
    admission: &WorthQueryAdmittedApplicationOperation<S, B::Operation, B::Input, Scope>,
) -> Result<(), WorkflowTransitionPreparationDenial>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    B: ApplicationCapabilityMutationBinding<S>,
    B::Operation: ApplicationOperationMarkerIdentity<S>,
    B::Input: ApplicationCapabilityRequest<
        S,
        B::Capability,
        Scope = <B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
    >,
    <B::Operation as ApplicationOperationMarkerIdentity<S>>::InputBinding:
        ApplicationStructuredValueBinding<Value = B::Input>,
{
    let deny = |subject| {
        WorkflowTransitionPreparationDenial::Attempt(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
            subject,
        ))
    };
    if !installed.supports_local_workflow_action::<B>() {
        return Err(deny(
            "binding has no installed local native workflow action body",
        ));
    }
    let capability = selected
        .application()
        .installed_schema()
        .capability(
            ApplicationCapabilityRef::<S, B::Capability>::from_declaration(),
            ApplicationOperationRef::<S, B::Operation, B::Input>::from_declaration(),
        )
        .map_err(|_| deny("workflow action binding capability is not installed"))?;
    if admission.installed_capability_identity() != Some(*capability.identity().bytes()) {
        return Err(deny(
            "workflow action requires the binding's exact admitted capability",
        ));
    }
    Ok(())
}
