use worth_store_physical_backend::QualifiedFilesystemMedia;

use crate::physical_runtime::durability::WalBoundPhysicalDataFrame;
use crate::physical_runtime::record_serving::residency::artifact_tree::PhysicalRecordArtifactTree;

/// Reads every maintenance candidate back from media, bypassing residency, and
/// compares it with the exact WAL-bound bytes before any root can name it.
/// A candidate that cannot be read, or reads differently, is never published.
pub(super) fn candidates_read_back_exactly(
    media: &QualifiedFilesystemMedia,
    frames: &[WalBoundPhysicalDataFrame],
) -> bool {
    let tree = PhysicalRecordArtifactTree::new(media);
    frames.iter().all(|frame| {
        let coordinate = frame.basis().target().coordinate();
        let expected = frame.bytes();
        let mut observed = Vec::new();
        if observed.try_reserve_exact(expected.len()).is_err() {
            return false;
        }
        observed.resize(expected.len(), 0);
        tree.read_exact_at(coordinate.artifact(), coordinate.offset(), &mut observed)
            .is_ok()
            && observed == expected
    })
}
