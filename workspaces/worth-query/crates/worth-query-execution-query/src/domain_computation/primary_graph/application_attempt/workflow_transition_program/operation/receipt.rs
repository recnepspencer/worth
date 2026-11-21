pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::operation::receipt::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationRef, ApplicationStructuredValueBinding,
};

use super::*;

pub fn validate_operation_receipt<'receipt, Schema, Binding>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    required: &RequiredWorkflowOperation,
    subject: worth_relational::facade::identity::EntityId,
    branch: crate::basis::WorthQueryProductBranch,
    receipt: &'receipt WorthQueryApplicationCommitReceipt,
    recovery: Option<
        &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
    >,
) -> Result<ValidatedWorkflowOperationReceipt<'receipt>, WorthQueryApplicationAttemptDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    if required.operation() != Binding::Operation::IDENTIFIER
        || required.binding() != Some(Binding::IDENTITY)
        || required.workflow_execution_requirements() != &Binding::workflow_execution_requirements()
        || required.input_type() != Binding::InputBinding::IDENTITY.as_str()
    {
        return Err(mismatch(required.node_path()));
    }
    let installed = runtime
        .installed_schema()
        .installed_operation(ApplicationOperationRef::<
            Schema,
            Binding::Operation,
            Binding::Input,
        >::from_declaration())
        .map_err(|_| mismatch(required.node_path()))?;
    let scope = receipt.authority_binding().principal_scope().scope();
    let receipt_idempotency = receipt.authority_binding().idempotency_binding();
    if receipt.runtime_authority() != runtime.runtime.authority_identity()
        || receipt
            .authority_binding()
            .principal_scope()
            .binding_identity()
            != &runtime.installed_schema().binding_identity()
        || receipt.installed_operation() != &installed.authority_identity_bytes()
        || receipt.product_branch() != branch
        || scope.partition_id() != subject.partition_value()
        || scope.local_slot() != subject.local_slot_value()
        || scope.generation() != subject.generation_value()
        || !receipt_idempotency.matches_workflow_operation(required.transition_identity_bytes())
        || receipt_idempotency.intent_identity() != required.input_identity()
    {
        return Err(mismatch(required.node_path()));
    }
    let identity = if receipt
        .dispatch_outbox()
        .and_then(|record| record.inbound())
        .is_some()
    {
        if !runtime
            .resolve_guarded_workflow_external_settlement(receipt)
            .unwrap_or(false)
        {
            return Err(mismatch(required.node_path()));
        }
        receipt_identity(receipt).ok_or_else(|| mismatch(required.node_path()))?
    } else {
        validate_operation_receipt_custody(required.node_path(), receipt, recovery)?
    };
    Ok(ValidatedWorkflowOperationReceipt { receipt, identity })
}

pub fn validate_operation_receipt_custody(
    node_path: &str,
    receipt: &WorthQueryApplicationCommitReceipt,
    recovery: Option<
        &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
    >,
) -> Result<[u8; 32], WorthQueryApplicationAttemptDenial> {
    if operation_receipt_requires_recovery(receipt)
        && !recovery.is_some_and(|proof| proof.completes_receipt(receipt))
    {
        return Err(mismatch(node_path));
    }
    receipt_identity(receipt).ok_or_else(|| mismatch(node_path))
}

pub fn receipt_identity(receipt: &WorthQueryApplicationCommitReceipt) -> Option<[u8; 32]> {
    Some(receipt_identity_from_outcome(
        receipt.runtime_authority().as_u64(),
        receipt.outcome_identity()?.get(),
        receipt.installed_operation(),
    ))
}
