use sha2::{Digest, Sha256};
use worth_store::physical_runtime::recovery_wal::{WalLsnRange, WalSegmentArtifactIdentity};
use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_recovery_physics::PhysicalRecoveryResidueKind;

use crate::progression::RecoveryPublicationExpectation;

use crate::cleanup::{
    RecoveryCleanupDeferralReason, RecoveryCleanupDisposition, RecoveryCleanupDispositionKind,
    RecoveryCleanupEligibility, RecoveryCleanupTarget,
};

pub(super) fn plan_identity(
    publication: &RecoveryPublicationExpectation,
    checkpoint: PhysicalCheckpointIdentity,
    candidates: &[RecoveryCleanupEligibility],
    dispositions: &[RecoveryCleanupDisposition],
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.cleanup-plan.v1");
    digest.update(publication.plan_identity());
    digest.update(publication.recovered_root().generation().to_le_bytes());
    digest.update(checkpoint.store_identity().bytes());
    digest.update(checkpoint.sequence().get().to_le_bytes());
    digest.update((candidates.len() as u64).to_le_bytes());
    for candidate in candidates {
        hash_wal(
            &mut digest,
            candidate.artifact(),
            candidate.range(),
            candidate.byte_count(),
            candidate.artifact_digest(),
        );
    }
    digest.update((dispositions.len() as u64).to_le_bytes());
    for disposition in dispositions {
        hash_disposition(&mut digest, disposition);
    }
    digest.finalize().into()
}

fn hash_disposition(digest: &mut Sha256, disposition: &RecoveryCleanupDisposition) {
    hash_target(digest, disposition.target());
    digest.update([disposition_kind(disposition.kind())]);
    if let RecoveryCleanupDispositionKind::Deferred(reason) = disposition.kind() {
        digest.update([deferral_reason(reason)]);
    }
    match disposition.wal_range() {
        Some(range) => {
            digest.update([1]);
            digest.update(range.start().get().to_le_bytes());
            digest.update(range.end_exclusive().get().to_le_bytes());
        }
        None => digest.update([0]),
    }
    digest.update(disposition.byte_count().to_le_bytes());
    match disposition.wal_digest() {
        Some(artifact_digest) => {
            digest.update([1]);
            digest.update(artifact_digest);
        }
        None => digest.update([0]),
    }
}

fn hash_target(digest: &mut Sha256, target: &RecoveryCleanupTarget) {
    match target {
        RecoveryCleanupTarget::Record(artifact) => {
            digest.update([0]);
            hash_bytes(digest, artifact.file_name().as_bytes());
        }
        RecoveryCleanupTarget::Checkpoint(checkpoint) => {
            digest.update([1]);
            digest.update(checkpoint.store_identity().bytes());
            digest.update(checkpoint.sequence().get().to_le_bytes());
        }
        RecoveryCleanupTarget::Wal(artifact) => {
            digest.update([2]);
            digest.update(artifact.segment().get().to_le_bytes());
            digest.update(artifact.generation().get().to_le_bytes());
        }
        RecoveryCleanupTarget::Residue { name, kind } => {
            digest.update([3, residue_kind(*kind)]);
            hash_bytes(digest, name.as_bytes());
        }
    }
}

fn hash_bytes(digest: &mut Sha256, bytes: &[u8]) {
    digest.update((bytes.len() as u64).to_le_bytes());
    digest.update(bytes);
}

const fn disposition_kind(kind: RecoveryCleanupDispositionKind) -> u8 {
    match kind {
        RecoveryCleanupDispositionKind::Current => 0,
        RecoveryCleanupDispositionKind::Retained => 1,
        RecoveryCleanupDispositionKind::Eligible => 2,
        RecoveryCleanupDispositionKind::Deferred(_) => 3,
        RecoveryCleanupDispositionKind::QuarantinedOrUnsupported => 4,
        RecoveryCleanupDispositionKind::SafelyRemoved => 5,
    }
}

const fn deferral_reason(reason: RecoveryCleanupDeferralReason) -> u8 {
    match reason {
        RecoveryCleanupDeferralReason::CandidateLimit => 0,
        RecoveryCleanupDeferralReason::ByteLimit => 1,
        RecoveryCleanupDeferralReason::UnresolvedOperationFate => 2,
        RecoveryCleanupDeferralReason::FreshnessUnavailable => 3,
        RecoveryCleanupDeferralReason::PublishedGenerationChanged => 4,
        RecoveryCleanupDeferralReason::EligibilityChanged => 5,
        RecoveryCleanupDeferralReason::Cancelled => 6,
        RecoveryCleanupDeferralReason::CancellationBindingMismatch => 7,
        RecoveryCleanupDeferralReason::DeniedBeforeEffect => 8,
        RecoveryCleanupDeferralReason::IndeterminateEffect => 9,
    }
}

const fn residue_kind(kind: PhysicalRecoveryResidueKind) -> u8 {
    match kind {
        PhysicalRecoveryResidueKind::NonCanonicalWalArtifact => 0,
        PhysicalRecoveryResidueKind::NonRegularWalEntry => 1,
        PhysicalRecoveryResidueKind::TrailingEmptyWalSegment => 2,
        PhysicalRecoveryResidueKind::InterruptedWalSegmentStart => 3,
        PhysicalRecoveryResidueKind::UnreferencedCompactionProduct => 4,
    }
}

fn hash_wal(
    digest: &mut Sha256,
    artifact: WalSegmentArtifactIdentity,
    range: WalLsnRange,
    bytes: u64,
    artifact_digest: [u8; 32],
) {
    digest.update(artifact.segment().get().to_le_bytes());
    digest.update(artifact.generation().get().to_le_bytes());
    digest.update(range.start().get().to_le_bytes());
    digest.update(range.end_exclusive().get().to_le_bytes());
    digest.update(bytes.to_le_bytes());
    digest.update(artifact_digest);
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store::physical_runtime::recovery_wal::{
        LogSequenceNumber, WalSegmentGeneration, WalSegmentId,
    };

    #[test]
    fn exact_wal_digest_changes_candidate_identity() {
        let artifact = WalSegmentArtifactIdentity::new(
            WalSegmentId::new(1).unwrap(),
            WalSegmentGeneration::new(1).unwrap(),
        );
        let range = WalLsnRange::new(LogSequenceNumber::new(1), LogSequenceNumber::new(2)).unwrap();
        let mut left = Sha256::new();
        let mut right = Sha256::new();
        hash_wal(&mut left, artifact, range, 8, [0x11; 32]);
        hash_wal(&mut right, artifact, range, 8, [0x22; 32]);
        assert_ne!(left.finalize(), right.finalize());
    }
}
