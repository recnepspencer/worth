//! Unknown origin tags fail payload admission after a valid Native frame.
use super::*;
use sha2::{Digest, Sha256};
use worth_relational::facade::{
    durability::{RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactRefusal},
    history::CommitId,
};
pub(super) fn require_unknown_origin_refusal(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    commit: CommitId,
) {
    let artifact = runtime
        .durability()
        .canonical_commit_artifacts(execution.allocation_policy())
        .map(Result::unwrap)
        .find(|artifact| artifact.commit_id() == commit)
        .unwrap();
    RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution).unwrap();
    // Named MessagePack enum tag, outside the opaque JSON payloads of current facts.
    let mut tag = vec![0xa0 + b"HistoricalSelection".len() as u8];
    tag.extend_from_slice(b"HistoricalSelection");
    let mut bytes = artifact.as_bytes().to_vec();
    let offset = bytes
        .windows(tag.len())
        .position(|window| window == tag)
        .unwrap();
    bytes[offset + tag.len() - 1] = b'X';
    // These are the documented public frame offsets, not decoder internals.
    let digest = Sha256::digest(&bytes[66..]);
    bytes[26..58].copy_from_slice(&digest);
    RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.clone()).unwrap();
    let error = RelationalCanonicalCommitArtifact::readmit(&bytes, execution).unwrap_err();
    assert!(
        matches!(error, RelationalCanonicalCommitArtifactRefusal::PayloadDecode { detail } if detail.contains("unknown variant"))
    );
}
