use super::super::evidence::BlobPublicationRecoveryOperationDigest;
use super::super::types::BlobPublicationPreWalReplayEvidence;
use super::super::{
    BlobPublicationCounterSnapshot, BlobPublicationCrashOutcome, BlobPublicationDenial,
    BlobPublicationReplayedCrashEdge,
};

pub(crate) fn from_replayed_crash_edge(
    replay: &BlobPublicationReplayedCrashEdge,
    expected_operation_digest: &BlobPublicationRecoveryOperationDigest,
) -> Result<BlobPublicationPreWalReplayEvidence, BlobPublicationDenial> {
    if replay.outcome() == BlobPublicationCrashOutcome::NoWalAppendObserved
        && replay.before_wal_append_operation_digest() == Some(expected_operation_digest.as_str())
    {
        Ok(BlobPublicationPreWalReplayEvidence {
            operation_digest: expected_operation_digest.as_str().to_owned(),
            classification_digest: replay.classification_digest().to_owned(),
            replay_read_identity: replay.replay_read_identity().to_owned(),
            counters: replay.counters(),
        })
    } else {
        Err(BlobPublicationDenial::WalReplayEvidenceRequired {
            counters: BlobPublicationCounterSnapshot::start().with_denied_promotion(),
        })
    }
}

pub(crate) fn require_operation(
    evidence: BlobPublicationPreWalReplayEvidence,
    expected_operation_digest: &BlobPublicationRecoveryOperationDigest,
) -> Result<BlobPublicationPreWalReplayEvidence, BlobPublicationDenial> {
    if evidence.operation_digest == expected_operation_digest.as_str() {
        Ok(evidence)
    } else {
        Err(BlobPublicationDenial::WalReplayEvidenceRequired {
            counters: BlobPublicationCounterSnapshot::start().with_denied_promotion(),
        })
    }
}
