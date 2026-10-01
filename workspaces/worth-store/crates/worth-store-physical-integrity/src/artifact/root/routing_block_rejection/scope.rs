//! One selected-root binding predicate for owned and borrowed route blocks.
use super::*;
use worth_store_physical_format::{
    ManifestBlockReference, PhysicalRootRoutingBlock, PhysicalRootRoutingBlockView,
};

pub(in crate::artifact::root) trait RoutingBlockMeaning {
    fn tree_identity(&self) -> u64;
    fn generation(&self) -> u64;
    fn block(&self) -> u64;
    fn level(&self) -> u16;
    fn count(&self) -> usize;
    fn is_leaf(&self) -> bool;
    fn reference(&self, checksum: u32) -> ManifestBlockReference;
}

impl RoutingBlockMeaning for PhysicalRootRoutingBlock {
    fn tree_identity(&self) -> u64 {
        self.tree_identity()
    }
    fn generation(&self) -> u64 {
        self.generation()
    }
    fn block(&self) -> u64 {
        self.block()
    }
    fn level(&self) -> u16 {
        self.level()
    }
    fn count(&self) -> usize {
        self.entries()
            .map_or_else(|| self.children().unwrap().len(), |entries| entries.len())
    }
    fn is_leaf(&self) -> bool {
        self.entries().is_some()
    }
    fn reference(&self, checksum: u32) -> ManifestBlockReference {
        self.reference(checksum)
    }
}

impl RoutingBlockMeaning for PhysicalRootRoutingBlockView<'_> {
    fn tree_identity(&self) -> u64 {
        (*self).tree_identity()
    }
    fn generation(&self) -> u64 {
        (*self).generation()
    }
    fn block(&self) -> u64 {
        (*self).block()
    }
    fn level(&self) -> u16 {
        (*self).level()
    }
    fn count(&self) -> usize {
        (*self).count()
    }
    fn is_leaf(&self) -> bool {
        self.entries().is_some()
    }
    fn reference(&self, checksum: u32) -> ManifestBlockReference {
        (*self).reference(checksum)
    }
}

pub(in crate::artifact::root) fn scope_mismatch(
    scope: PhysicalArtifactScope,
    block: &impl RoutingBlockMeaning,
) -> Option<PhysicalIntegrityRejection> {
    let expected = scope.root_routing_block_identity().unwrap();
    let reference = expected.reference();
    if block.tree_identity() != expected.tree().get() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            TREE_FIELD,
            PhysicalFormatField::TreeIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if block.block() != reference.block() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            ENVELOPE_BLOCK_FIELD,
            PhysicalFormatField::BlockIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if block.generation() != reference.generation() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            GENERATION_FIELD,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if block.level() != reference.level() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            LEVEL_FIELD,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    range_mismatch(scope, block)
}

fn range_mismatch(
    scope: PhysicalArtifactScope,
    block: &impl RoutingBlockMeaning,
) -> Option<PhysicalIntegrityRejection> {
    let expected = scope.root_routing_block_identity().unwrap().reference();
    let observed = block.reference(expected.checksum());
    let field = if block.is_leaf() {
        PhysicalFormatField::RecordIdentity
    } else {
        PhysicalFormatField::ChildReference
    };
    let offset = if observed.first() != expected.first() {
        first_range_offset(block)
    } else if observed.last() != expected.last() {
        last_range_offset(block)
    } else {
        return None;
    };
    Some(damaged(
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        PhysicalByteRange::new(scope.byte_range().offset() + offset, 24).unwrap(),
        Some(field),
        PhysicalBlastRadius::ReachableSubtree,
    ))
}

fn first_range_offset(block: &impl RoutingBlockMeaning) -> u64 {
    if block.is_leaf() {
        BODY_OFFSET
    } else {
        BODY_OFFSET + 24
    }
}

fn last_range_offset(block: &impl RoutingBlockMeaning) -> u64 {
    if block.is_leaf() {
        BODY_OFFSET + (block.count() as u64 - 1) * LEAF_ENTRY_BYTES
    } else {
        BODY_OFFSET + (block.count() as u64 - 1) * BRANCH_REFERENCE_BYTES + 48
    }
}
