//! Store's own join of the pending source head tree to the checkpoint head
//! tree, from the two root frames Store read itself.

use worth_store_physical_format::DurablePhysicalRootManifest;

use super::Denial;

/// Without ordered history Store walks no edge between the checkpoint source
/// root and the root the pending edge leaves, so that root must still carry
/// the checkpoint head tree: the same head root and the same block frontier.
/// Another object's release above the checkpoint moves both, and Store would
/// seal a head whose release edge and drops it never walked. With ordered
/// history `ordered_walk` replays every edge between the two roots instead.
pub(super) fn require_unmoved_without_history(
    checkpoint_source_root: &DurablePhysicalRootManifest,
    source_root: &DurablePhysicalRootManifest,
    ordered_history: bool,
) -> Result<(), Denial> {
    if !ordered_history
        && (checkpoint_source_root.release_custody_head_root()
            != source_root.release_custody_head_root()
            || checkpoint_source_root.next_release_custody_head_block()
                != source_root.next_release_custody_head_block())
    {
        return Err(Denial::CertificateRoster);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::{
        PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
        ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadBlockV1, ReleaseCustodyHeadEntryV1,
        ReleaseCustodyHeadKeyV1,
    };

    use super::*;

    fn head_tree(object: u8, block: u64) -> ReleaseCustodyHeadBlockReferenceV1 {
        let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
        let record = |ordinal| PersistedRecordIdentity::new([object; 16], ordinal).unwrap();
        let entry = ReleaseCustodyHeadEntryV1::new(
            ReleaseCustodyHeadKeyV1::new([object; 16], 1).unwrap(),
            record(1),
            [1; 32],
            record(2),
            [2; 32],
            record(3),
            [3; 32],
            [4; 32],
            None,
            1,
            1,
            false,
        )
        .unwrap();
        ReleaseCustodyHeadBlockV1::leaf(9, 2, block, vec![entry], format)
            .unwrap()
            .reference(format)
    }

    fn root(
        generation: u64,
        head_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
        next_block: u64,
    ) -> DurablePhysicalRootManifest {
        DurablePhysicalRootManifest::builder(generation, 9, 2, 43)
            .release_custody_head_root(head_root)
            .next_release_custody_head_block(next_block)
            .admit()
            .unwrap()
    }

    #[test]
    fn source_head_tree_that_left_the_checkpoint_tree_needs_ordered_history() {
        let checkpoint_tree = Some(head_tree(1, 1));
        let checkpoint = root(2, checkpoint_tree, 2);
        let denied = |checkpoint: &DurablePhysicalRootManifest,
                      source: &DurablePhysicalRootManifest| {
            matches!(
                require_unmoved_without_history(checkpoint, source, false),
                Err(Denial::CertificateRoster)
            )
        };
        // The source root's head tree already carries another object's
        // release above the checkpoint.
        let released_above = root(4, Some(head_tree(2, 2)), 3);
        assert!(denied(&checkpoint, &released_above));
        // The same tree root over an advanced frontier, and another tree root
        // at the same frontier, are each a different tree.
        assert!(denied(&checkpoint, &root(4, checkpoint_tree, 3)));
        assert!(denied(&checkpoint, &root(4, Some(head_tree(2, 1)), 2)));
        // A checkpoint without a head tree is not joined by a root with one.
        assert!(denied(&root(2, None, 1), &checkpoint));
        // Ordered history hands the moved tree to the edge-by-edge walk.
        assert!(require_unmoved_without_history(&checkpoint, &released_above, true).is_ok());
        // A later root that left the head tree alone is the checkpoint tree.
        assert!(!denied(&checkpoint, &root(4, checkpoint_tree, 2)));
        assert!(!denied(&root(2, None, 1), &root(4, None, 1)));
    }
}
