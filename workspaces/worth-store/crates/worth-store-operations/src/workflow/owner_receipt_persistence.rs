use sha2::Digest;
use worth_store_physical_backend::NonCurrentStagingExecutionReceipt;

use crate::{
    AuthorizationConsumptionReceipt, OperationalControlAppendDenial, OperationalControlRecord,
    OperationalControlStorePort, OperationalOperationId, OperationalOwnerReceiptKind,
    OperationalTransitionId, OperationalWorkflowKind,
};

pub(crate) fn persist_recovery_owner_receipts(
    control: &(impl OperationalControlStorePort + ?Sized),
    authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    operation_id: &OperationalOperationId,
    authorization: AuthorizationConsumptionReceipt,
    workflow: OperationalWorkflowKind,
    backend: &NonCurrentStagingExecutionReceipt,
    recovery_receipt_identity: [u8; 32],
) -> Result<(), OperationalControlAppendDenial> {
    persist(
        control,
        authority_identity,
        operation_id,
        OperationalTransitionId::backend_owner_receipt(),
        workflow,
        authorization.plan_fingerprint(),
        backend_owner_receipt_identity(backend),
        OperationalOwnerReceiptKind::Backend,
    )?;
    persist(
        control,
        authority_identity,
        operation_id,
        OperationalTransitionId::recovery_owner_receipt(),
        workflow,
        authorization.plan_fingerprint(),
        recovery_receipt_identity,
        OperationalOwnerReceiptKind::Recovery,
    )
}

fn persist(
    control: &(impl OperationalControlStorePort + ?Sized),
    authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    operation_id: &OperationalOperationId,
    transition_id: OperationalTransitionId,
    workflow: OperationalWorkflowKind,
    plan_fingerprint: [u8; 32],
    receipt_fingerprint: [u8; 32],
    owner_kind: OperationalOwnerReceiptKind,
) -> Result<(), OperationalControlAppendDenial> {
    control.append(
        &OperationalControlRecord::operational_owner_receipt_persisted(
            authority_identity,
            operation_id.clone(),
            transition_id,
            workflow,
            plan_fingerprint,
            receipt_fingerprint,
            owner_kind,
        ),
    )?;
    Ok(())
}

fn backend_owner_receipt_identity(value: &NonCurrentStagingExecutionReceipt) -> [u8; 32] {
    crate::workflow::recovery_replay::fingerprint(
        b"worth-store-backend-recovery-owner-receipt-v1",
        |digest| {
            digest.update(value.plan_fingerprint());
            digest.update(value.bytes_copied().to_be_bytes());
            digest.update(value.artifacts_materialized().to_be_bytes());
            digest.update(value.media().content_fingerprint());
        },
    )
}
