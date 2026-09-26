use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
    application_program::{ApplicationProgramDefinition, ApplicationWorkflowSpec},
    application_schema::{ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding},
};
use worth_query_execution::facade::{
    application_installation::WorthQueryWorkflowVocabulary,
    workflow_advance::{
        PreparedWorkflowAdvance, RequiredWorkflowOperation, WorkflowProgressOutcome,
        WorthQueryWorkflowAdvanceAdapter,
    },
};
use worth_query_installation::facade::ApplicationSchema;

use super::progress::WorthQueryWorkflowAdvanceRequest;
use crate::application_entry::mutation::WorthQueryApplicationMutationRequestWithIdempotency;

#[path = "operation/owner.rs"]
mod owner;
#[path = "operation/recovery.rs"]
mod recovery;
pub use owner::{
    WorthQueryWorkflowOperationOwnerAcceptanceDenial, WorthQueryWorkflowOperationOwnerPosture,
};
pub use recovery::{
    WorthQueryPreparedWorkflowOperationRecovery, WorthQueryWorkflowOperationRecoveryDenial,
    WorthQueryWorkflowOperationRecoveryPreparationDenial,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowOperationBindingDenial {
    RuntimeMismatch,
    RequirementMismatch,
    AuthorityUnavailable,
}

#[derive(Debug)]
pub enum WorthQueryWorkflowOperationAcceptanceDenial {
    NotAwaitingOperation,
    RequirementMismatch,
    RecoveryRequired,
    RecoveryNotRequired,
    Replay(
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial,
    ),
    Attempt(worth_query_execution::facade::primary_graph::WorthQueryApplicationAttemptDenial),
}

impl std::fmt::Display for WorthQueryWorkflowOperationAcceptanceDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAwaitingOperation => formatter.write_str("workflow is not awaiting operation"),
            Self::RequirementMismatch => {
                formatter.write_str("operation receipt does not match the workflow requirement")
            }
            Self::RecoveryRequired => formatter.write_str(
                "operation receipt has unresolved external custody and requires recovery",
            ),
            Self::RecoveryNotRequired => {
                formatter.write_str("operation receipt does not require recovery")
            }
            Self::Replay(denial) => denial.fmt(formatter),
            Self::Attempt(denial) => denial.fmt(formatter),
        }
    }
}

impl std::error::Error for WorthQueryWorkflowOperationAcceptanceDenial {}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub fn for_workflow_operation<'workflow, Spec, Program>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec, Program>>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Self, WorthQueryWorkflowOperationBindingDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Program: ApplicationProgramDefinition<Schema> + 'workflow,
    {
        let workflow = workflow.into();
        self.validate_workflow_operation_binding(workflow, required)?;
        if !required.authority_slot().was_issued() {
            return Err(WorthQueryWorkflowOperationBindingDenial::AuthorityUnavailable);
        }
        Ok(self.bind_workflow_transition(
            *required.transition_identity_bytes(),
            required.authority_slot(),
        ))
    }

    /// Binds only the exact performed operation for outbox recovery. This does
    /// not issue authority to run a new guarded mutation.
    pub fn for_workflow_operation_recovery<'workflow, Spec, Program>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'workflow, Schema, Spec, Program>>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Self, WorthQueryWorkflowOperationBindingDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Program: ApplicationProgramDefinition<Schema> + 'workflow,
    {
        let workflow = workflow.into();
        self.validate_workflow_operation_binding(workflow, required)?;
        Ok(self.bind_workflow_recovery_transition(*required.transition_identity_bytes()))
    }

    fn validate_workflow_operation_binding<Spec, Program>(
        &self,
        workflow: WorthQueryWorkflowVocabulary<'_, Schema, Spec, Program>,
        required: &RequiredWorkflowOperation,
    ) -> Result<(), WorthQueryWorkflowOperationBindingDenial>
    where
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Program: ApplicationProgramDefinition<Schema>,
    {
        if !std::ptr::eq(self.application_runtime(), workflow.runtime()) {
            return Err(WorthQueryWorkflowOperationBindingDenial::RuntimeMismatch);
        }
        if required.operation()
            != <<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::Operation::IDENTIFIER
            || required.binding()
                != Some(<<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::IDENTITY)
            || !<<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::REQUIRES_WORKFLOW_AUTHORITY
            || required.input_type()
                != <<Intent as ApplicationMutationIntent<Schema>>::Binding as ApplicationMutationBinding<Schema>>::InputBinding::IDENTITY.as_str()
            || required.input_identity() != &self.input_identity()
            || required.branch() != self.product_branch()
        {
            return Err(WorthQueryWorkflowOperationBindingDenial::RequirementMismatch);
        }
        Ok(())
    }
}

impl<'application, 'principal, 'scope, Schema, Spec, Program, Operation, Input, Scope>
    WorthQueryWorkflowAdvanceRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Spec,
        Program,
        Operation,
        Input,
        Scope,
    >
where
    Schema: ApplicationSchema,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Operation: 'static,
    Input: Clone + Send + Sync + 'static,
{
    fn accept_operation<Binding, EffectOperation, EffectInput, EffectScope>(
        self,
        required: &RequiredWorkflowOperation,
        receipt: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        effect_admission: &worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, EffectOperation, EffectInput, EffectScope>,
        effect_idempotency: worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowOperationAcceptanceDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
        EffectInput: Clone + Send + Sync + 'static,
    {
        if required.operation() != Binding::Operation::IDENTIFIER
            || required.binding() != Some(Binding::IDENTITY)
            || required.input_type() != Binding::InputBinding::IDENTITY.as_str()
        {
            return Err(WorthQueryWorkflowOperationAcceptanceDenial::RequirementMismatch);
        }
        if WorthQueryWorkflowAdvanceAdapter::operation_receipt_requires_recovery(receipt) {
            return Err(WorthQueryWorkflowOperationAcceptanceDenial::RecoveryRequired);
        }
        if let Some(replayed) = WorthQueryWorkflowAdvanceAdapter::resolve_operation_replay::<
            Schema,
            Operation,
            Input,
            Scope,
            Binding,
        >(
            self.application,
            &self.prepared,
            required,
            receipt,
            self.idempotency,
        )
        .map_err(WorthQueryWorkflowOperationAcceptanceDenial::Replay)?
        {
            return Ok(replayed);
        }
        let prepared = match self.prepared {
            PreparedWorkflowAdvance::AwaitingOperation(prepared) => prepared,
            PreparedWorkflowAdvance::Transition { .. }
            | PreparedWorkflowAdvance::AwaitingAssessment(_)
            | PreparedWorkflowAdvance::AwaitingCondition(_)
            | PreparedWorkflowAdvance::AwaitingEvidence { .. }
            | PreparedWorkflowAdvance::AwaitingApproval { .. }
            | PreparedWorkflowAdvance::ReplayOnly { .. } => {
                return Err(WorthQueryWorkflowOperationAcceptanceDenial::NotAwaitingOperation)
            }
        };
        if !same_requirement(prepared.required(), required) {
            return Err(WorthQueryWorkflowOperationAcceptanceDenial::RequirementMismatch);
        }
        WorthQueryWorkflowAdvanceAdapter::compare_and_commit_operation::<
            Schema,
            Operation,
            Input,
            Scope,
            Binding,
            EffectOperation,
            EffectInput,
            EffectScope,
        >(
            self.application,
            prepared,
            effect_admission,
            effect_idempotency,
            required,
            self.idempotency,
        )
        .map_err(WorthQueryWorkflowOperationAcceptanceDenial::Attempt)
    }

    fn accept_recovered_operation<Binding, EffectOperation, EffectInput, EffectScope>(
        self,
        required: &RequiredWorkflowOperation,
        receipt: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        recovery: &worth_query_execution::facade::primary_graph::WorthQueryRecoverySafeRetryAdmission,
        effect_admission: &worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, EffectOperation, EffectInput, EffectScope>,
        effect_idempotency: worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> Result<WorkflowProgressOutcome, WorthQueryWorkflowOperationAcceptanceDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
        EffectInput: Clone + Send + Sync + 'static,
    {
        if required.operation() != Binding::Operation::IDENTIFIER
            || required.binding() != Some(Binding::IDENTITY)
            || required.input_type() != Binding::InputBinding::IDENTITY.as_str()
        {
            return Err(WorthQueryWorkflowOperationAcceptanceDenial::RequirementMismatch);
        }
        if !WorthQueryWorkflowAdvanceAdapter::operation_receipt_requires_recovery(receipt) {
            return Err(WorthQueryWorkflowOperationAcceptanceDenial::RecoveryNotRequired);
        }
        if let Some(replayed) =
            WorthQueryWorkflowAdvanceAdapter::resolve_recovered_operation_replay::<
                Schema,
                Operation,
                Input,
                Scope,
                Binding,
            >(
                self.application,
                &self.prepared,
                required,
                receipt,
                recovery,
                self.idempotency,
            )
            .map_err(WorthQueryWorkflowOperationAcceptanceDenial::Replay)?
        {
            return Ok(replayed);
        }
        let prepared = match self.prepared {
            PreparedWorkflowAdvance::AwaitingOperation(prepared) => prepared,
            PreparedWorkflowAdvance::Transition { .. }
            | PreparedWorkflowAdvance::AwaitingAssessment(_)
            | PreparedWorkflowAdvance::AwaitingCondition(_)
            | PreparedWorkflowAdvance::AwaitingEvidence { .. }
            | PreparedWorkflowAdvance::AwaitingApproval { .. }
            | PreparedWorkflowAdvance::ReplayOnly { .. } => {
                return Err(WorthQueryWorkflowOperationAcceptanceDenial::NotAwaitingOperation)
            }
        };
        if !same_requirement(prepared.required(), required) {
            return Err(WorthQueryWorkflowOperationAcceptanceDenial::RequirementMismatch);
        }
        WorthQueryWorkflowAdvanceAdapter::compare_and_commit_recovered_operation::<
            Schema,
            Operation,
            Input,
            Scope,
            Binding,
            EffectOperation,
            EffectInput,
            EffectScope,
        >(
            self.application,
            prepared,
            effect_admission,
            effect_idempotency,
            required,
            recovery,
            self.idempotency,
        )
        .map_err(WorthQueryWorkflowOperationAcceptanceDenial::Attempt)
    }
}

fn same_requirement(left: &RequiredWorkflowOperation, right: &RequiredWorkflowOperation) -> bool {
    left.branch() == right.branch()
        && left.instance() == right.instance()
        && left.node_path() == right.node_path()
        && left.transition_identity() == right.transition_identity()
        && left.occurrence() == right.occurrence()
        && left.operation() == right.operation()
        && left.binding() == right.binding()
        && left.input_type() == right.input_type()
        && left.input_identity() == right.input_identity()
}
