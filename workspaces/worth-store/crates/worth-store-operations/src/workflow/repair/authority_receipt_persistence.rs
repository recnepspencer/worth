use super::integrity_classification::IntegrityRepairClassificationReceipt;
use crate::workflow::restore::RecoveredBackupFrontierReceipt;
use sha2::{Digest, Sha256};
use worth_store_physical_backend::NonCurrentStagingExecutionReceipt;

use super::authority_affecting_execution::AuthorityAffectingRepairExecutionDenial;
use super::journal::RepairExecutionJournal;
use crate::OwnerPlanNodeIdentity;

pub(super) fn persist(
    journal: &mut RepairExecutionJournal<'_>,
    node: OwnerPlanNodeIdentity,
    receipt: [u8; 32],
    owner_tag: u8,
) -> Result<(), AuthorityAffectingRepairExecutionDenial> {
    if let Some(reopened) = journal.completed(node) {
        return if reopened == receipt {
            Ok(())
        } else {
            Err(AuthorityAffectingRepairExecutionDenial::RecoveredReceiptMismatch { node })
        };
    }
    journal
        .persist_owner_receipt(node, receipt, owner_tag)
        .map_err(AuthorityAffectingRepairExecutionDenial::Journal)
}

pub(super) fn integrity_receipt(value: IntegrityRepairClassificationReceipt) -> [u8; 32] {
    fingerprint(b"worth-store-integrity-repair-receipt-v2", |d| {
        d.update(value.plan_fingerprint());
        d.update(value.classified_regions().to_be_bytes());
        d.update(value.quarantined_regions().to_be_bytes());
    })
}
pub(super) fn backend_receipt(value: &NonCurrentStagingExecutionReceipt) -> [u8; 32] {
    fingerprint(b"worth-store-backend-repair-receipt-v2", |d| {
        d.update(value.plan_fingerprint());
        d.update(value.bytes_copied().to_be_bytes());
        d.update(value.artifacts_materialized().to_be_bytes());
        d.update(value.media().content_fingerprint());
    })
}
pub(super) fn recovery_receipt(value: RecoveredBackupFrontierReceipt) -> [u8; 32] {
    let replay = value.replay_source();
    let application = value.application();
    fingerprint(b"worth-store-recovery-repair-receipt-v2", |d| {
        d.update(value.plan_fingerprint());
        d.update(value.durable_checkpoint_lsn().to_be_bytes());
        d.update(value.wal_end_exclusive_lsn().to_be_bytes());
        d.update(value.acknowledged_frontier().to_be_bytes());
        d.update(value.root_generation().to_be_bytes());
        d.update(replay.identity());
        d.update(replay.manifest_digest());
        d.update(replay.frame_count().to_be_bytes());
        d.update(replay.bytes_verified().to_be_bytes());
        d.update(replay.interval().0.to_be_bytes());
        d.update(replay.interval().1.to_be_bytes());
        d.update(application.identity());
        d.update(application.application_identity());
        d.update(application.replay_source_identity());
        d.update(application.resulting_frontier_identity());
        d.update(application.applied_frames().to_be_bytes());
    })
}
pub(super) fn layout_receipt(
    value: worth_store_layout_indexes::LayoutRepairConsequenceReceipt,
) -> [u8; 32] {
    fingerprint(b"worth-store-layout-repair-receipt-v2", |d| {
        d.update(value.plan_fingerprint());
        d.update(value.verified_artifacts().to_be_bytes());
        d.update(value.verified_bytes().to_be_bytes());
        d.update([match value.consequence() {
            worth_store_layout_indexes::LayoutRepairConsequence::RestoreDamagedArtifact => 1,
            worth_store_layout_indexes::LayoutRepairConsequence::ReplaceQuarantinedArtifact => 2,
        }]);
    })
}
pub(super) fn blob_receipt(
    value: worth_store_blob_chunks::BlobRepairConsequenceReceipt,
) -> [u8; 32] {
    fingerprint(b"worth-store-blob-repair-receipt-v2", |d| {
        d.update(value.plan_fingerprint());
        d.update(value.verified_artifacts().to_be_bytes());
        d.update(value.verified_bytes().to_be_bytes());
        d.update([match value.consequence() {
            worth_store_blob_chunks::BlobRepairConsequence::RestoreDamagedArtifact => 1,
            worth_store_blob_chunks::BlobRepairConsequence::ReplaceQuarantinedArtifact => 2,
        }]);
    })
}
fn fingerprint(domain: &[u8], update: impl FnOnce(&mut Sha256)) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(domain);
    update(&mut digest);
    digest.finalize().into()
}
