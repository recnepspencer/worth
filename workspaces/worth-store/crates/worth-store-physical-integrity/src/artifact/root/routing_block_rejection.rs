use worth_store_physical_format::{
    BoundedRootRoutingBlockDecodeDenial, PhysicalRootRoutingBlock, RootManifestDenial,
    RootRoutingBlockDenial,
};

use crate::artifact::durable_frame_rejection::{
    damaged, field_damage, from_frame_denial, DurableFrameFieldRange,
};
use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
};
use crate::validation::{PhysicalArtifactScope, PhysicalIntegrityRejection};

const FORMAT_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
const ENVELOPE_BLOCK_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
const BLOCK_COPIES_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 36);
const TREE_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 8);
const PAYLOAD_BLOCK_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(56, 8);
const LEVEL_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(64, 2);
const COUNT_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(66, 2);
const KIND_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(68, 1);
const PREFIX_RESERVED_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(69, 3);
const GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(72, 8);
const TRAILING_RESERVED_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(80, 8);
const FRAME_HEADER_BYTES: u64 = 48;
const BODY_OFFSET: u64 = 88;
const LEAF_ENTRY_BYTES: u64 = 88;
const BRANCH_REFERENCE_BYTES: u64 = 72;

pub(super) fn scope_mismatch(
    scope: PhysicalArtifactScope,
    block: &PhysicalRootRoutingBlock,
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
    block: &PhysicalRootRoutingBlock,
) -> Option<PhysicalIntegrityRejection> {
    let expected = scope.root_routing_block_identity().unwrap().reference();
    let observed = block.reference(expected.checksum());
    let field = if block.entries().is_some() {
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

fn first_range_offset(block: &PhysicalRootRoutingBlock) -> u64 {
    if block.entries().is_some() {
        BODY_OFFSET
    } else {
        BODY_OFFSET + 24
    }
}

fn last_range_offset(block: &PhysicalRootRoutingBlock) -> u64 {
    if let Some(entries) = block.entries() {
        BODY_OFFSET + (entries.len() as u64 - 1) * LEAF_ENTRY_BYTES
    } else {
        BODY_OFFSET + (block.children().unwrap().len() as u64 - 1) * BRANCH_REFERENCE_BYTES + 48
    }
}

pub(super) fn routing_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: BoundedRootRoutingBlockDecodeDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        BoundedRootRoutingBlockDecodeDenial::Format(denial) => format_denial(scope, bytes, denial),
        BoundedRootRoutingBlockDecodeDenial::LeafEntries { .. }
        | BoundedRootRoutingBlockDecodeDenial::BranchChildren { .. } => count_damage(scope),
    }
}

fn format_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: RootRoutingBlockDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        RootRoutingBlockDenial::Frame(denial) => from_frame_denial(scope, denial),
        RootRoutingBlockDenial::MalformedPrefix => malformed_prefix(scope, bytes),
        RootRoutingBlockDenial::IdentityOrCapacity => identity_or_capacity(scope, bytes),
        RootRoutingBlockDenial::LevelOrKind => level_or_kind(scope, bytes),
        RootRoutingBlockDenial::MalformedLength => count_damage(scope),
        RootRoutingBlockDenial::Placement(denial) => placement_damage(scope, denial),
        RootRoutingBlockDenial::InvalidReference => reference_damage(scope, bytes),
        RootRoutingBlockDenial::CanonicalOrder if read_u16(bytes, 64) == 0 => body_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            PhysicalFormatField::RecordIdentity,
        ),
        RootRoutingBlockDenial::CanonicalOrder => reference_damage(scope, bytes),
    }
}

fn malformed_prefix(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    if bytes.len() < BODY_OFFSET as usize {
        let range = if bytes.len() > FRAME_HEADER_BYTES as usize {
            PhysicalByteRange::new(
                scope.byte_range().offset() + FRAME_HEADER_BYTES,
                bytes.len() as u64 - FRAME_HEADER_BYTES,
            )
            .unwrap()
        } else {
            scope.byte_range()
        };
        return damaged(
            scope,
            PhysicalDamageCause::MalformedStructure,
            range,
            Some(PhysicalFormatField::Reserved),
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    let field = if bytes[69..72] != [0; 3] {
        PREFIX_RESERVED_FIELD
    } else {
        TRAILING_RESERVED_FIELD
    };
    field_damage(
        scope,
        PhysicalDamageCause::MalformedStructure,
        field,
        PhysicalFormatField::Reserved,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn identity_or_capacity(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let expected = scope.root_routing_block_identity().unwrap();
    if read_u64(bytes, 48) == 0 {
        return field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            TREE_FIELD,
            PhysicalFormatField::TreeIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    let envelope = read_u64(bytes, 28);
    let payload = read_u64(bytes, 56);
    if envelope == 0 && payload == 0 {
        return block_identity_damage(scope, BLOCK_COPIES_FIELD);
    }
    if envelope != payload {
        let field = if envelope == expected.reference().block() {
            PAYLOAD_BLOCK_FIELD
        } else if payload == expected.reference().block() {
            ENVELOPE_BLOCK_FIELD
        } else {
            BLOCK_COPIES_FIELD
        };
        return block_identity_damage(scope, field);
    }
    if read_u64(bytes, 72) == 0 {
        return field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            GENERATION_FIELD,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    count_damage(scope)
}

fn block_identity_damage(
    scope: PhysicalArtifactScope,
    field: DurableFrameFieldRange,
) -> PhysicalIntegrityRejection {
    field_damage(
        scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        field,
        PhysicalFormatField::BlockIdentity,
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn level_or_kind(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let observed_level = read_u16(bytes, 64);
    let expected_level = scope
        .root_routing_block_identity()
        .unwrap()
        .reference()
        .level();
    let (cause, field, format_field, blast_radius) = if observed_level != expected_level {
        (
            PhysicalDamageCause::ChildReferenceMismatch,
            LEVEL_FIELD,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::ReachableSubtree,
        )
    } else {
        (
            PhysicalDamageCause::RecordKindMismatch,
            KIND_FIELD,
            PhysicalFormatField::RoutingNodeKind,
            PhysicalBlastRadius::CompleteArtifact,
        )
    };
    field_damage(scope, cause, field, format_field, blast_radius)
}

fn reference_damage(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let parent_level = read_u16(bytes, 64);
    let parent_generation = read_u64(bytes, 72);
    let count = usize::from(read_u16(bytes, 66));
    for index in 0..count {
        let start = BODY_OFFSET as usize + index * BRANCH_REFERENCE_BYTES as usize;
        if read_u64(bytes, start) > parent_generation {
            return child_field_damage(scope, start as u64, 8);
        }
        if read_u16(bytes, start + 16).checked_add(1) != Some(parent_level) {
            return child_field_damage(scope, start as u64 + 16, 2);
        }
    }
    body_damage(
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        PhysicalFormatField::ChildReference,
    )
}

fn child_field_damage(
    scope: PhysicalArtifactScope,
    offset: u64,
    length: u64,
) -> PhysicalIntegrityRejection {
    field_damage(
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        DurableFrameFieldRange::new(offset, length),
        PhysicalFormatField::ChildReference,
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn placement_damage(
    scope: PhysicalArtifactScope,
    denial: RootManifestDenial,
) -> PhysicalIntegrityRejection {
    let (cause, field) = match denial {
        RootManifestDenial::InvalidRecordIdentity => (
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PhysicalFormatField::RecordIdentity,
        ),
        RootManifestDenial::InvalidPlacement => (
            PhysicalDamageCause::ChildReferenceMismatch,
            PhysicalFormatField::ChildReference,
        ),
        _ => (
            PhysicalDamageCause::MalformedStructure,
            PhysicalFormatField::Payload,
        ),
    };
    body_damage(scope, cause, field)
}

pub(super) fn format_mismatch(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    field_damage(
        scope,
        PhysicalDamageCause::FormatMismatch,
        FORMAT_FIELD,
        PhysicalFormatField::FormatDeclaration,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn count_damage(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    field_damage(
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        COUNT_FIELD,
        PhysicalFormatField::EncodedLength,
        PhysicalBlastRadius::CanonicalFrame,
    )
}

pub(super) fn recursive_checksum_mismatch(
    scope: PhysicalArtifactScope,
) -> PhysicalIntegrityRejection {
    damaged(
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        Some(PhysicalFormatField::Checksum),
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn body_damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    field: PhysicalFormatField,
) -> PhysicalIntegrityRejection {
    damaged(
        scope,
        cause,
        PhysicalByteRange::new(
            scope.byte_range().offset() + BODY_OFFSET,
            scope.byte_range().length() - BODY_OFFSET,
        )
        .unwrap(),
        Some(field),
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}
