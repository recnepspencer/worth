//! The head tree a pending edge without ordered replays mutates is the
//! checkpoint head tree itself.

use worth_store_physical_format::{
    DurablePhysicalRootManifest, ReleaseCustodyHeadBlockReferenceV1,
};

use super::EffectiveReleaseHeadDenial as Denial;

/// With no ordered replay between the checkpoint and the pending edge,
/// nothing accounts for a head tree that moved above the checkpoint. The
/// pending edge must leave the selected root, and that root must still carry
/// the head root and block frontier of the checkpoint source root. Returns
/// that checkpoint tree: the only tree the effective roster may fold from.
pub(super) fn unmoved_checkpoint_head_tree(
    selected_root: &DurablePhysicalRootManifest,
    checkpoint_source_root: &DurablePhysicalRootManifest,
    source_root: &DurablePhysicalRootManifest,
) -> Result<(Option<ReleaseCustodyHeadBlockReferenceV1>, u64), Denial> {
    let head_root = checkpoint_source_root.release_custody_head_root();
    let next_block = checkpoint_source_root.next_release_custody_head_block();
    if selected_root != source_root
        || selected_root.release_custody_head_root() != head_root
        || selected_root.next_release_custody_head_block() != next_block
    {
        return Err(Denial::Source);
    }
    Ok((head_root, next_block))
}

#[cfg(test)]
mod tests {
    use worth_store_physical_format::{
        PersistedRecordIdentity, PhysicalRecordFormatDeclaration, ReleaseCustodyHeadBlockV1,
        ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
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
    fn pending_source_without_ordered_replays_must_carry_the_checkpoint_head_tree() {
        let checkpoint_tree = Some(head_tree(1, 1));
        let checkpoint = root(2, checkpoint_tree, 2);
        // A later root that left the head tree alone is the checkpoint tree.
        let unmoved = root(4, checkpoint_tree, 2);
        assert_eq!(
            unmoved_checkpoint_head_tree(&unmoved, &checkpoint, &unmoved),
            Ok((checkpoint_tree, 2))
        );
        assert_eq!(
            unmoved_checkpoint_head_tree(&checkpoint, &checkpoint, &checkpoint),
            Ok((checkpoint_tree, 2))
        );
        // Another object's release above the checkpoint rewrote the head root
        // and advanced the frontier; no ordered replay explains either.
        let rewritten = root(4, Some(head_tree(2, 2)), 3);
        assert_eq!(
            unmoved_checkpoint_head_tree(&rewritten, &checkpoint, &rewritten),
            Err(Denial::Source)
        );
        let advanced_frontier = root(4, checkpoint_tree, 3);
        assert_eq!(
            unmoved_checkpoint_head_tree(&advanced_frontier, &checkpoint, &advanced_frontier),
            Err(Denial::Source)
        );
        let same_frontier_other_root = root(4, Some(head_tree(2, 1)), 2);
        assert_eq!(
            unmoved_checkpoint_head_tree(
                &same_frontier_other_root,
                &checkpoint,
                &same_frontier_other_root
            ),
            Err(Denial::Source)
        );
        // A checkpoint without a head tree is not extended by a root with one.
        let headless = root(2, None, 1);
        assert_eq!(
            unmoved_checkpoint_head_tree(&unmoved, &headless, &unmoved),
            Err(Denial::Source)
        );
        // The pending edge must leave the selected root itself.
        assert_eq!(
            unmoved_checkpoint_head_tree(&unmoved, &checkpoint, &checkpoint),
            Err(Denial::Source)
        );
    }
}
