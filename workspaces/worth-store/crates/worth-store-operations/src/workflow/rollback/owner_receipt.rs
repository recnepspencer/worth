use super::RollbackExecutionReceipt;

pub(crate) fn rollback_owner_receipt_identity(value: RollbackExecutionReceipt) -> [u8; 32] {
    crate::workflow::recovery_replay::replay_owner_identity(
        b"worth-store-rollback-owner-receipt-v1",
        value.plan_fingerprint(),
        value.frontier(),
        value.replay_source(),
        value.application(),
    )
}
