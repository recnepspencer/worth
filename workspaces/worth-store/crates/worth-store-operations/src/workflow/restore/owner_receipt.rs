use sha2::Digest;

use super::RecoveredBackupFrontierReceipt;

pub(crate) fn restored_frontier_owner_receipt_identity(
    value: RecoveredBackupFrontierReceipt,
) -> [u8; 32] {
    let replay = value.replay_source();
    let application = value.application();
    crate::workflow::recovery_replay::fingerprint(
        b"worth-store-recovery-owner-receipt-v1",
        |digest| {
            digest.update(value.plan_fingerprint());
            digest.update(value.durable_checkpoint_lsn().to_be_bytes());
            digest.update(value.wal_end_exclusive_lsn().to_be_bytes());
            digest.update(value.acknowledged_frontier().to_be_bytes());
            digest.update(value.root_generation().to_be_bytes());
            digest.update(replay.identity());
            digest.update(replay.manifest_digest());
            digest.update(replay.frame_count().to_be_bytes());
            digest.update(replay.bytes_verified().to_be_bytes());
            digest.update(replay.interval().0.to_be_bytes());
            digest.update(replay.interval().1.to_be_bytes());
            digest.update(application.identity());
            digest.update(application.application_identity());
            digest.update(application.replay_source_identity());
            digest.update(application.resulting_frontier_identity());
            digest.update(application.applied_frames().to_be_bytes());
        },
    )
}
