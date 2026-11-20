use super::*;
use worth_store_physical_backend::RecoveryStagingExistingArtifact::{DiscardUnpublished, Verify};

/// Only a name above the selected root is an unpublished candidate; every
/// name a published root may reach, or that carries no generation, verifies.
#[test]
fn only_names_above_the_selected_root_are_discarded_unpublished_candidates() {
    let successor = RecoveryCandidateGeneration::successor(7).unwrap();
    assert_eq!(successor.generation(), 8);
    let republication = RecoveryCandidateGeneration::republication(7);
    assert_eq!(republication.generation(), 7);
    let cases = [
        (
            RecordArtifactFile::RootManifest { generation: 8 },
            DiscardUnpublished,
        ),
        (
            RecordArtifactFile::RootRoutingBlock {
                generation: 8,
                block: 22,
            },
            DiscardUnpublished,
        ),
        (
            RecordArtifactFile::Segment {
                segment: 3,
                generation: 9,
            },
            DiscardUnpublished,
        ),
        (RecordArtifactFile::RootManifest { generation: 7 }, Verify),
        (
            RecordArtifactFile::SegmentManifest {
                segment: 3,
                generation: 2,
            },
            Verify,
        ),
        (RecordArtifactFile::ExtentArena { arena: 1 }, Verify),
        (
            RecordArtifactFile::CatalogCandidate { publication: 8 },
            Verify,
        ),
        (RecordArtifactFile::CurrentRootSelector, Verify),
    ];
    for (artifact, expected) in cases {
        assert_eq!(
            successor.existing_artifact(artifact),
            expected,
            "{artifact:?}"
        );
        assert_eq!(
            republication.existing_artifact(artifact),
            expected,
            "{artifact:?}"
        );
    }
    assert_eq!(RecoveryCandidateGeneration::successor(u64::MAX), None);
}
