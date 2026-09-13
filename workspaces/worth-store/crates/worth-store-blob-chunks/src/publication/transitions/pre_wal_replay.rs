use crate::{ChunkTreeRoot, LogicalContentDigest};

use super::super::evidence::operation_digest;
use super::super::types::recovery_types::BlobPublicationPreWalReplayEvidence;
use super::super::types::{
    reachability_staging::BlobReachabilityStaging, root_candidate::BlobRootCandidateForPublication,
};
use super::super::verification::pre_wal_replay as verify;
use super::super::{BlobPublicationDenial, BlobPublicationReplayedCrashEdge};

pub(crate) fn from_chunk_write_replay(
    digest: &LogicalContentDigest,
    replay: &BlobPublicationReplayedCrashEdge,
) -> Result<BlobPublicationPreWalReplayEvidence, BlobPublicationDenial> {
    verify::from_replayed_crash_edge(
        replay,
        &operation_digest::chunk_write_recovery_operation_digest(digest),
    )
}

pub(crate) fn from_checksum_admitted_replay(
    digest: &LogicalContentDigest,
    replay: &BlobPublicationReplayedCrashEdge,
) -> Result<BlobPublicationPreWalReplayEvidence, BlobPublicationDenial> {
    verify::from_replayed_crash_edge(
        replay,
        &operation_digest::checksum_recovery_operation_digest(digest),
    )
}

pub(crate) fn from_chunk_tree_node_durable_replay(
    root: &ChunkTreeRoot,
    replay: &BlobPublicationReplayedCrashEdge,
) -> Result<BlobPublicationPreWalReplayEvidence, BlobPublicationDenial> {
    verify::from_replayed_crash_edge(
        replay,
        &operation_digest::chunk_tree_recovery_operation_digest(root),
    )
}

pub(crate) fn from_root_candidate_replay(
    candidate: &BlobRootCandidateForPublication,
    replay: &BlobPublicationReplayedCrashEdge,
) -> Result<BlobPublicationPreWalReplayEvidence, BlobPublicationDenial> {
    verify::from_replayed_crash_edge(
        replay,
        &operation_digest::root_candidate_recovery_operation_digest(candidate),
    )
}

pub(crate) fn from_reachability_staged_replay(
    staged: &BlobReachabilityStaging,
    replay: &BlobPublicationReplayedCrashEdge,
) -> Result<BlobPublicationPreWalReplayEvidence, BlobPublicationDenial> {
    verify::from_replayed_crash_edge(
        replay,
        &operation_digest::reachability_recovery_operation_digest(staged),
    )
}
