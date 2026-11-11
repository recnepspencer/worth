use super::*;

#[test]
fn memory_frame_sizing_obeys_the_reader_limit_before_allocation() {
    let maximum = super::super::MAXIMUM_ARTIFACT_BYTES;
    assert_eq!(bounded_frame_len(maximum - HEADER_LEN).unwrap(), maximum);
    assert!(matches!(bounded_frame_len(maximum - HEADER_LEN + 1),
        Err(Denial::SizeLimit { observed, maximum: limit })
        if observed == maximum + 1 && limit == maximum));
    assert!(matches!(
        bounded_frame_len(usize::MAX),
        Err(Denial::FrameSizeOverflow)
    ));
    let header = CanonicalCommitArtifactHeader {
        position: PatchStreamPosition(1),
        commit_id: crate::history::data::CommitId(1),
        digest: super::super::payload_digest::RelationalCanonicalCommitArtifactDigest::unsealed(),
    };
    assert!(matches!(
        CanonicalCommitArtifactHeader::inspect(&header.emit((maximum - HEADER_LEN) as u64)),
        Err(super::super::RelationalCanonicalCommitArtifactFrameRefusal::TruncatedPayload { .. })
    ));
    assert!(matches!(
        CanonicalCommitArtifactHeader::inspect(&header.emit((maximum - HEADER_LEN + 1) as u64)),
        Err(super::super::RelationalCanonicalCommitArtifactFrameRefusal::SizeLimit { .. })
    ));
}
