//! A local business action uses its installed binding's native admission, never Advance.
use super::*;
use crate::domain_computation::primary_graph::{
    workflow::instance::SelectedWorkflowOperation, WorthQueryApplicationAttemptDenialKind,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationAdmissionBinding, ApplicationMutationIdentities,
};
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;

impl WorthQueryWorkflowAdvanceAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_local_action<S, O, I, Scope, Spec, B>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
        instance: PublishedWorkflowInstanceRef,
        admission: WorthQueryAdmittedApplicationOperation<S, O, I, Scope>,
        identities: &ApplicationMutationIdentities<'_, S, B>,
        kind: WorkflowLocalActionKind,
    ) -> Result<PreparedWorkflowAdvance<S, O, I, Scope>, WorkflowTransitionPreparationDenial>
    where
        S: ApplicationSchema,
        O: ApplicationOperationMarkerIdentity<S> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationMutationAdmissionBinding<S, Operation = O, Input = I>,
        I: 'static,
        O::InputBinding: ApplicationStructuredValueBinding<Value = I>,
    {
        validate_action_admission::<S, Spec, B, Scope>(installed, &admission)?;
        let input_identity = *identities.input_identity();
        if admission
            .governed_input_identity()
            .is_some_and(|admitted| admitted != &input_identity)
            || (admission.installed_capability_identity().is_some()
                && admission.governed_input_identity().is_none())
        {
            return Err(mismatch(
                "local request differs from its capability-admitted input",
            ));
        }
        let execution = phase
            .execution_request_for(&selected.application().product_runtime)
            .map_err(|_| mismatch("foreign workflow action preparation phase"))?;
        selected.prepare_local_workflow_transition(
            &execution,
            installed,
            instance,
            admission,
            WorkflowTransitionRequestKind::LocalAction {
                kind,
                requested: SelectedWorkflowOperation {
                    operation: O::IDENTIFIER.to_owned(),
                    input_type: B::InputBinding::IDENTITY.as_str().to_owned(),
                    binding: Some(B::IDENTITY.to_owned()),
                    workflow_execution_requirements: B::workflow_execution_requirements(),
                },
                input_identity,
                completion: None,
            },
        )
    }
}

/// The sealed native mode must match the exact installed binding and operation.
pub(super) fn validate_action_admission<S, Spec, B, Scope>(
    installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
    admission: &WorthQueryAdmittedApplicationOperation<S, B::Operation, B::Input, Scope>,
) -> Result<(), WorkflowTransitionPreparationDenial>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
    B: ApplicationMutationAdmissionBinding<S>,
{
    let expected = installed
        .local_action_admission_identity::<B>()
        .ok_or_else(|| mismatch("binding has no installed local native workflow action body"))?;
    if expected != admission.installed_capability_identity()
        || admission.operation() != B::Operation::IDENTIFIER
    {
        return Err(mismatch(
            "local action differs from the binding's installed native admission",
        ));
    }
    Ok(())
}

fn mismatch(subject: &str) -> WorkflowTransitionPreparationDenial {
    WorkflowTransitionPreparationDenial::Attempt(WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
        subject,
    ))
}
