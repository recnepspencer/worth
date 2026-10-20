//! The original local effect's fresh capability owns completion, not generic Advance.
use super::super::{local_actions::VerifiedLocalActionCompletion, WorkflowLocalActionKind};
use super::*;
use crate::domain_computation::primary_graph::{
    workflow::instance::SelectedWorkflowOperation, WorthQueryApplicationCommitReceipt,
};
mod replay;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationScopeBinding,
    },
    application_schema::ApplicationStructuredValueBinding,
};

impl WorthQueryWorkflowAdvanceAdapter {
    /// Reobserve the original route only after exact committed effect evidence.
    /// No old operation authority is restored or made executable again.
    #[allow(clippy::too_many_arguments)]
    pub fn complete_local_operation<S, Spec, B>(
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        workflow: WorthQueryWorkflowVocabulary<'_, S, Spec>,
        required: &RequiredWorkflowOperation,
        admission: WorthQueryAdmittedApplicationOperation<
            S,
            B::Operation,
            B::Input,
            <B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        receipt: &WorthQueryApplicationCommitReceipt,
        recovery: Option<
            &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
        >,
    ) -> Result<WorkflowProgressOutcome, WorkflowTransitionPreparationDenial>
    where
        S: ApplicationSchema,
        Spec: ApplicationWorkflowSpec<Schema = S>,
        B: ApplicationCapabilityMutationBinding<S>,
        B::Operation: ApplicationOperationMarkerIdentity<S>,
        B::Input: Clone
            + Send
            + Sync
            + ApplicationCapabilityRequest<
                S,
                B::Capability,
                Scope = <B::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
            > + 'static,
        <B::Operation as ApplicationOperationMarkerIdentity<S>>::InputBinding:
            ApplicationStructuredValueBinding<Value = B::Input>,
    {
        use WorkflowTransitionPreparationDenial as D;
        let runtime = selected.application();
        if !std::ptr::eq(workflow.runtime(), runtime) {
            return Err(D::Attempt(local_mismatch(
                "foreign local completion runtime",
            )));
        }
        let installed = workflow
            .workflow_spec_at_revision(required.original_program_revision())
            .ok_or_else(|| {
                D::Attempt(local_mismatch("original local vocabulary is unavailable"))
            })?;
        super::local_actions::validate_action_admission::<S, Spec, B, _>(
            selected, installed, &admission,
        )?;
        let kind = validate_original(selected, installed, required)?;
        let performed = super::super::operation::validate_operation_receipt::<S, B>(
            runtime,
            required,
            admission.scope_entity_id(),
            required.branch(),
            receipt,
            recovery,
        )
        .map_err(D::Attempt)?;
        let receipt_identity = performed.identity();
        let execution = phase
            .execution_request_for(&runtime.product_runtime)
            .map_err(|_| D::Attempt(local_mismatch("foreign local completion phase")))?;
        let binding = idempotency.bind_local_workflow_settlement(
            required.transition_identity_bytes(),
            &receipt_identity,
        );
        if let Some(replayed) = replay::resolve(
            &execution,
            runtime,
            &admission,
            required,
            binding,
            receipt,
            receipt_identity,
        )? {
            return Ok(replayed);
        }
        let requested = SelectedWorkflowOperation {
            operation: required.operation().to_owned(),
            input_type: required.input_type().to_owned(),
            binding: required.binding().map(str::to_owned),
            workflow_execution_requirements: required.workflow_execution_requirements().clone(),
        };
        let prepared = selected
            .prepare_workflow_transition::<B::Capability, B::Operation, B::Input, _, Spec>(
                &execution,
                installed,
                required.original_instance().clone(),
                admission,
                WorkflowTransitionRequestKind::LocalAction {
                    kind,
                    requested,
                    input_identity: *required.input_identity(),
                    completion: Some(VerifiedLocalActionCompletion {
                        transition: required.transition_identity().to_owned(),
                    }),
                },
            )?;
        let PreparedWorkflowAdvance::AwaitingOperation(prepared) = prepared else {
            return Err(D::Attempt(local_mismatch(
                "original local occurrence no longer awaits completion",
            )));
        };
        if prepared.required.transition_identity_bytes() != required.transition_identity_bytes()
            || prepared.required.input_identity() != required.input_identity()
        {
            return Err(D::Attempt(local_mismatch(
                "local completion identity changed",
            )));
        }
        let prepared = match recovery {
            Some(recovery) => prepared.settle_recovered::<B>(runtime, receipt, recovery),
            None => prepared.settle::<B>(runtime, receipt),
        }
        .map_err(D::Attempt)?;
        Ok(runtime.compare_and_commit_workflow_advance(&execution, phase, prepared, binding))
    }
}
fn validate_original<S, Spec>(
    selected: &WorthQuerySelectedProductOperation<'_, S>,
    installed: &WorthQueryInstalledApplicationWorkflowSpec<S, Spec>,
    required: &RequiredWorkflowOperation,
) -> Result<WorkflowLocalActionKind, WorkflowTransitionPreparationDenial>
where
    S: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = S>,
{
    use WorkflowTransitionPreparationDenial as D;
    let origin = required
        .local_action
        .as_ref()
        .ok_or_else(|| D::Attempt(local_mismatch("not an issued local action")))?;
    if required.original_runtime() != selected.application().runtime.authority_identity().as_u64()
        || required.original_program_revision() != installed.program_revision()
        || origin.spec != Spec::IDENTITY.as_str()
        || &origin.support != installed.support_identity_bytes()
        || selected.product().product_branch() != required.branch()
        || selected
            .inspect_selected_program()
            .map_err(|_| D::Attempt(local_mismatch("selected program unavailable")))?
            .revision()
            != installed.program_revision()
    {
        return Err(D::Attempt(local_mismatch(
            "original local runtime or program changed",
        )));
    }
    Ok(origin.kind)
}
fn local_mismatch(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        super::super::super::WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
        subject,
    )
}
