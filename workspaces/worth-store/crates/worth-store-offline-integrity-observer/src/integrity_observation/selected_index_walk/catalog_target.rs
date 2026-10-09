use std::path::Path;

use super::super::{
    blob_record::{self, BlobFact},
    families::index::OfflineBTreeNodeFacts,
    record_walk::damage,
    BoundedMediaWalk, OfflineIntegrityOutcome as Outcome, OfflinePhysicalBlastRadius as Blast,
    OfflinePhysicalDamageCause as Cause,
};
use super::{resolver::resolve, OfflineRootManifestFacts};

/// A catalog leaf must point at selected authoritative publication bytes with
/// the same object and generation as its physical key.
pub(super) fn verify_catalog_targets(
    root: &Path,
    selected: &OfflineRootManifestFacts,
    node: &OfflineBTreeNodeFacts,
    store: Option<[u8; 16]>,
    walk: &mut BoundedMediaWalk,
) -> Result<(), Outcome> {
    for (key, target) in node.keys.iter().zip(&node.leaf_records) {
        let selected_target = resolve(root, selected, *target, walk)?;
        let fact = blob_record::decode(&selected_target.payload, store, walk.counters_mut())?;
        let BlobFact::Publication {
            object, generation, ..
        } = fact
        else {
            return Err(target_mismatch());
        };
        if key[..16] != object || key[16..24] != generation.to_be_bytes() {
            return Err(target_mismatch());
        }
    }
    Ok(())
}

fn target_mismatch() -> Outcome {
    damage(Cause::Pointer, None, Blast::ReachableRootSubtree)
}
