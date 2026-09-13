use crate::{ChunkTreeRoot, LogicalContentDigest};

use super::super::classification::BlobPublicationCrashPoint;
use super::super::evidence::{
    operation_digest, recovery_evidence_digest, BlobPublicationRecoveryOperationDigest,
};
use super::super::types::recovery_types::{
    BlobPublicationPreWalReplayEvidence, BlobPublicationRecoveryEvidence,
};
use super::super::types::{
    reachability_staging::BlobReachabilityStaging, root_candidate::BlobRootCandidateForPublication,
    session_closeout::BlobPublicationSessionCloseout,
};
use super::super::verification::replayable_wal;
use super::super::{BlobPublicationCrashBoundaryReport, BlobPublicationDenial};

fn build_pre_wal_evidence(
    crash_point: BlobPublicationCrashPoint,
    phase_label: &str,
    digest: &LogicalContentDigest,
    replay: BlobPublicationPreWalReplayEvidence,
    operation_digest: BlobPublicationRecoveryOperationDigest,
) -> Result<BlobPublicationRecoveryEvidence, BlobPublicationDenial> {
    let replay = replay.require_operation(&operation_digest)?;
    Ok(BlobPublicationRecoveryEvidence::new(
        crash_point,
        recovery_evidence_digest(
            phase_label,
            replay.replay_read_identity(),
            digest.digest().as_str(),
        ),
    ))
}

pub(crate) fn chunk_write_replayed(
    digest: &LogicalContentDigest,
    replay: BlobPublicationPreWalReplayEvidence,
) -> Result<BlobPublicationRecoveryEvidence, BlobPublicationDenial> {
    build_pre_wal_evidence(
        BlobPublicationCrashPoint::AfterChunkWrite,
        "chunk-write",
        digest,
        replay,
        operation_digest::chunk_write_recovery_operation_digest(digest),
    )
}

pub(crate) fn checksum_admitted(
    digest: &LogicalContentDigest,
    replay: BlobPublicationPreWalReplayEvidence,
) -> Result<BlobPublicationRecoveryEvidence, BlobPublicationDenial> {
    build_pre_wal_evidence(
        BlobPublicationCrashPoint::AfterChecksumAdmission,
        "checksum-admitted",
        digest,
        replay,
        operation_digest::checksum_recovery_operation_digest(digest),
    )
}

pub(crate) fn chunk_tree_node_durable(
    root: &ChunkTreeRoot,
    replay: BlobPublicationPreWalReplayEvidence,
) -> Result<BlobPublicationRecoveryEvidence, BlobPublicationDenial> {
    let operation = operation_digest::chunk_tree_recovery_operation_digest(root);
    let replay = replay.require_operation(&operation)?;
    Ok(BlobPublicationRecoveryEvidence::new(
        BlobPublicationCrashPoint::AfterChunkTreeNodeDurability,
        recovery_evidence_digest(
            "chunk-tree-durable",
            replay.replay_read_identity(),
            root.digest().as_str(),
        ),
    ))
}

pub(crate) fn root_candidate(
    candidate: &BlobRootCandidateForPublication,
    replay: BlobPublicationPreWalReplayEvidence,
) -> Result<BlobPublicationRecoveryEvidence, BlobPublicationDenial> {
    let operation = operation_digest::root_candidate_recovery_operation_digest(candidate);
    let replay = replay.require_operation(&operation)?;
    Ok(BlobPublicationRecoveryEvidence::new(
        BlobPublicationCrashPoint::AfterRootCandidateFormation,
        recovery_evidence_digest(
            "root-candidate",
            replay.replay_read_identity(),
            candidate.intent().chunk_tree_root().digest().as_str(),
        ),
    ))
}

pub(crate) fn reachability_staged(
    staged: &BlobReachabilityStaging,
    replay: BlobPublicationPreWalReplayEvidence,
) -> Result<BlobPublicationRecoveryEvidence, BlobPublicationDenial> {
    let operation = operation_digest::reachability_recovery_operation_digest(staged);
    let replay = replay.require_operation(&operation)?;
    Ok(BlobPublicationRecoveryEvidence::new(
        BlobPublicationCrashPoint::AfterReachabilityStaging,
        recovery_evidence_digest(
            "reachability-staged",
            replay.replay_read_identity(),
            staged
                .staging_identity()
                .publication_record_digest()
                .as_str(),
        ),
    ))
}

pub(crate) fn publication_record_replayable(
    report: &BlobPublicationCrashBoundaryReport,
) -> Result<BlobPublicationRecoveryEvidence, BlobPublicationDenial> {
    replayable_wal::verify_replayable_report(report)?;
    Ok(BlobPublicationRecoveryEvidence::new(
        BlobPublicationCrashPoint::AfterPublicationRecordWrite,
        report.classification_digest(),
    ))
}

pub(crate) fn session_closed(
    closeout: &BlobPublicationSessionCloseout,
) -> BlobPublicationRecoveryEvidence {
    BlobPublicationRecoveryEvidence::new(
        BlobPublicationCrashPoint::AfterSessionClose,
        recovery_evidence_digest(
            "session-closed",
            closeout.wal_commit().replay_classification_digest(),
            closeout
                .wal_commit()
                .staging_identity()
                .publication_record_digest()
                .as_str(),
        ),
    )
}
