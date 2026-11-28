use super::*;
use contracts::durability::canonical_commit_artifact::payload_digest::RelationalCanonicalCommitArtifactDigestOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;

#[test]
fn memory_frame_sizing_obeys_the_reader_limit_before_allocation() {
    let maximum =
        contracts::durability::canonical_commit_artifact::frame_contract::MAXIMUM_ARTIFACT_BYTES;
    assert_eq!(bounded_frame_len(maximum - HEADER_LEN).unwrap(), maximum);
    assert!(matches!(bounded_frame_len(maximum - HEADER_LEN + 1),
        Err(Denial::SizeLimit { observed, maximum: limit })
        if observed == maximum + 1 && limit == maximum));
    assert!(matches!(
        bounded_frame_len(usize::MAX),
        Err(Denial::FrameSizeOverflow)
    ));
    let header = CanonicalCommitArtifactHeader {
        format: contracts::durability::canonical_commit_artifact::frame::CanonicalCommitArtifactFormat::Ordinary,
        position: PatchStreamPosition(1),
        commit_id: contracts::history::data::coordinates::CommitId(1),
        digest: contracts::durability::canonical_commit_artifact::payload_digest::RelationalCanonicalCommitArtifactDigest::unsealed(),
    };
    assert!(matches!(
        CanonicalCommitArtifactHeader::inspect(&header.emit((maximum - HEADER_LEN) as u64)),
        Err(contracts::durability::canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal::TruncatedPayload { .. })
    ));
    assert!(matches!(
        CanonicalCommitArtifactHeader::inspect(&header.emit((maximum - HEADER_LEN + 1) as u64)),
        Err(contracts::durability::canonical_commit_artifact::frame_refusal::RelationalCanonicalCommitArtifactFrameRefusal::SizeLimit { .. })
    ));
}
