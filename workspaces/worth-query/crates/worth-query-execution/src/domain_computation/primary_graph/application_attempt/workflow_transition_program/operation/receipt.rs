use sha2::{Digest, Sha256};
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationRef, ApplicationStructuredValueBinding,
};

use super::*;

pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn validate_operation_receipt<
    Schema,
    Binding,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    required: &RequiredWorkflowOperation,
    subject: worth_relational::facade::identity::EntityId,
    branch: crate::basis::WorthQueryProductBranch,
    receipt: &WorthQueryApplicationCommitReceipt,
    recovery: Option<
        &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
    >,
) -> Result<[u8; 32], WorthQueryApplicationAttemptDenial>
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
{
    if required.operation() != Binding::Operation::IDENTIFIER
        || required.binding() != Some(Binding::IDENTITY)
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
    validate_operation_receipt_custody(required.node_path(), receipt, recovery)
}

pub(in crate::domain_computation::primary_graph::application_attempt) fn operation_receipt_requires_recovery(
    receipt: &WorthQueryApplicationCommitReceipt,
) -> bool {
    receipt.dispatch_outbox().is_some()
        && !receipt
            .external_dispatch()
            .is_some_and(|dispatch| dispatch.is_external_completion())
}

pub(super) fn validate_operation_receipt_custody(
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

pub(super) fn receipt_identity(receipt: &WorthQueryApplicationCommitReceipt) -> Option<[u8; 32]> {
    Some(receipt_identity_from_outcome(
        receipt.runtime_authority().as_u64(),
        receipt.outcome_identity()?.get(),
        receipt.installed_operation(),
    ))
}

pub(in crate::domain_computation::primary_graph) fn receipt_identity_from_outcome(
    runtime_authority: u64,
    outcome_identity: u64,
    installed_operation: &[u8; 32],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth-query.workflow-operation-receipt.v1");
    digest.update(runtime_authority.to_le_bytes());
    digest.update(outcome_identity.to_le_bytes());
    digest.update(installed_operation);
    digest.finalize().into()
}
