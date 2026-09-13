use super::PointInTimeRecoveryReceipt;

pub(crate) fn pitr_owner_receipt_identity(value: PointInTimeRecoveryReceipt) -> [u8; 32] {
    crate::workflow::recovery_replay::replay_owner_identity(
        b"worth-store-pitr-owner-receipt-v1",
        value.plan_fingerprint(),
        value.exact_frontier(),
        value.replay_source(),
        value.application(),
    )
}
