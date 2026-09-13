use worth_store_physical_format::{
    BoundedFreeSpaceMembershipBlockDecodeDenial, FreeSpaceRoutingDenial,
};

use crate::artifact::durable_frame_rejection::{
    damaged, field_damage, from_frame_denial, DurableFrameFieldRange,
};
use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
};
use crate::validation::{PhysicalArtifactScope, PhysicalIntegrityRejection};

pub(super) struct MembershipField;

impl MembershipField {
    pub(super) const FORMAT: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
    pub(super) const ENVELOPE_IDENTITY: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
    pub(super) const TREE_IDENTITY: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 8);
    const BLOCK_IDENTITY: DurableFrameFieldRange = DurableFrameFieldRange::new(56, 8);
    pub(super) const LEVEL: DurableFrameFieldRange = DurableFrameFieldRange::new(64, 2);
    const COUNT: DurableFrameFieldRange = DurableFrameFieldRange::new(66, 2);
    const KIND: DurableFrameFieldRange = DurableFrameFieldRange::new(68, 1);
    pub(super) const GENERATION: DurableFrameFieldRange = DurableFrameFieldRange::new(72, 8);
    const RESERVED_KIND: DurableFrameFieldRange = DurableFrameFieldRange::new(69, 3);
    const RESERVED_PREFIX: DurableFrameFieldRange = DurableFrameFieldRange::new(80, 8);
}

pub(super) fn free_space_membership_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: BoundedFreeSpaceMembershipBlockDecodeDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        BoundedFreeSpaceMembershipBlockDecodeDenial::LeafEntries { .. }
        | BoundedFreeSpaceMembershipBlockDecodeDenial::BranchChildren { .. } => count_damage(scope),
        BoundedFreeSpaceMembershipBlockDecodeDenial::Format(denial) => match denial {
            FreeSpaceRoutingDenial::Frame(denial) => from_frame_denial(scope, denial),
            FreeSpaceRoutingDenial::Malformed => malformed_membership(scope, bytes),
            FreeSpaceRoutingDenial::IdentityOrCapacity => {
                membership_identity_or_capacity(scope, bytes)
            }
            FreeSpaceRoutingDenial::InvalidReference => invalid_reference(scope, bytes),
            FreeSpaceRoutingDenial::CanonicalOrder => canonical_order(scope, bytes),
        },
    }
}

fn malformed_membership(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    if bytes.len() < 88 {
        return malformed_payload(scope);
    }
    for range in [
        MembershipField::RESERVED_KIND,
        MembershipField::RESERVED_PREFIX,
    ] {
        if range.bytes(bytes).iter().any(|byte| *byte != 0) {
            return field_damage(
                scope,
                PhysicalDamageCause::MalformedStructure,
                range,
                PhysicalFormatField::Reserved,
                PhysicalBlastRadius::CompleteArtifact,
            );
        }
    }
    let level = read_u16(bytes, MembershipField::LEVEL);
    let kind = MembershipField::KIND.bytes(bytes)[0];
    let expected_level = scope
        .free_space_membership_block_identity()
        .expect("membership scope carries identity")
        .reference()
        .level();
    let expected_kind = if expected_level == 0 { 1 } else { 2 };
    if level != expected_level {
        return field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            MembershipField::LEVEL,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    if kind != expected_kind {
        return field_damage(
            scope,
            PhysicalDamageCause::RecordKindMismatch,
            MembershipField::KIND,
            PhysicalFormatField::MembershipKind,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    let width = if kind == 1 { 40 } else { 56 };
    let count = usize::from(read_u16(bytes, MembershipField::COUNT));
    if bytes.len() != 88 + count * width {
        return count_damage(scope);
    }
    if kind == 1 {
        if let Some(offending) = first_invalid_entry(bytes) {
            return damaged(
                scope,
                PhysicalDamageCause::MalformedStructure,
                body_item_range(scope, bytes, width, offending),
                Some(PhysicalFormatField::Payload),
                PhysicalBlastRadius::CompleteArtifact,
            );
        }
    }
    damaged(
        scope,
        PhysicalDamageCause::MalformedStructure,
        membership_body_range(scope, bytes),
        Some(PhysicalFormatField::Payload),
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn membership_identity_or_capacity(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> PhysicalIntegrityRejection {
    if bytes.len() >= 88 {
        let expected = scope
            .free_space_membership_block_identity()
            .expect("membership scope carries identity");
        for range in [
            MembershipField::ENVELOPE_IDENTITY,
            MembershipField::BLOCK_IDENTITY,
        ] {
            if read_u64(bytes, range) != expected.reference().block() {
                return identity_damage(scope, range, PhysicalFormatField::BlockIdentity);
            }
        }
        if read_u64(bytes, MembershipField::TREE_IDENTITY) != expected.tree().get() {
            return identity_damage(
                scope,
                MembershipField::TREE_IDENTITY,
                PhysicalFormatField::TreeIdentity,
            );
        }
        if read_u64(bytes, MembershipField::GENERATION) != expected.reference().generation() {
            return field_damage(
                scope,
                PhysicalDamageCause::PhysicalGenerationMismatch,
                MembershipField::GENERATION,
                PhysicalFormatField::PhysicalGeneration,
                PhysicalBlastRadius::ReachableSubtree,
            );
        }
    }
    count_damage(scope)
}

fn invalid_reference(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let offending = first_invalid_child(bytes).unwrap_or(0);
    damaged(
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        body_item_range(scope, bytes, 56, offending),
        Some(PhysicalFormatField::ChildReference),
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn canonical_order(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let kind = bytes.get(68).copied().unwrap_or(0);
    let width = if kind == 1 { 40 } else { 56 };
    if kind == 2 {
        if let Some(offending) = first_nonmember_child(bytes) {
            return damaged(
                scope,
                PhysicalDamageCause::ChildReferenceMismatch,
                body_item_range(scope, bytes, width, offending),
                Some(PhysicalFormatField::ChildReference),
                PhysicalBlastRadius::ReachableSubtree,
            );
        }
    }
    let offending = first_noncanonical_item(bytes, kind, width).unwrap_or(1);
    damaged(
        scope,
        PhysicalDamageCause::SequenceMismatch,
        body_item_range(scope, bytes, width, offending),
        Some(PhysicalFormatField::MembershipRange),
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn first_invalid_child(bytes: &[u8]) -> Option<usize> {
    bytes.get(88..)?.chunks_exact(56).position(|child| {
        child[18..20] != [0; 2]
            || read_at_u64(child, 0) == 0
            || read_at_u64(child, 8) == 0
            || !valid_key(&child[24..40])
            || !valid_key(&child[40..56])
            || read_key(child, 24) > read_key(child, 40)
    })
}

fn first_invalid_entry(bytes: &[u8]) -> Option<usize> {
    bytes.get(88..)?.chunks_exact(40).position(|entry| {
        !valid_key(&entry[..16])
            || read_at_u64(entry, 16) == 0
            || read_at_u64(entry, 24) == 0
            || read_at_u64(entry, 32) == 0
    })
}

fn first_nonmember_child(bytes: &[u8]) -> Option<usize> {
    let parent_level = read_u16(bytes, MembershipField::LEVEL);
    let parent_generation = read_u64(bytes, MembershipField::GENERATION);
    bytes.get(88..)?.chunks_exact(56).position(|child| {
        u16::from_le_bytes(child[16..18].try_into().unwrap()).checked_add(1) != Some(parent_level)
            || read_at_u64(child, 0) > parent_generation
    })
}

fn valid_key(bytes: &[u8]) -> bool {
    matches!(bytes[0], 1 | 2) && bytes[1..8] == [0; 7] && read_at_u64(bytes, 8) != 0
}

fn first_noncanonical_item(bytes: &[u8], kind: u8, width: usize) -> Option<usize> {
    let body = bytes.get(88..)?;
    let count = body.len() / width;
    (1..count).find(|index| {
        let previous = &body[(index - 1) * width..index * width];
        let current = &body[index * width..(index + 1) * width];
        if kind == 1 {
            read_key(previous, 0) >= read_key(current, 0)
        } else {
            read_key(previous, 40) >= read_key(current, 24)
        }
    })
}

fn read_key(bytes: &[u8], offset: usize) -> (u8, u64) {
    (
        bytes[offset],
        u64::from_le_bytes(bytes[offset + 8..offset + 16].try_into().unwrap()),
    )
}

fn count_damage(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    field_damage(
        scope,
        PhysicalDamageCause::MalformedStructure,
        MembershipField::COUNT,
        PhysicalFormatField::MembershipCount,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn identity_damage(
    scope: PhysicalArtifactScope,
    range: DurableFrameFieldRange,
    field: PhysicalFormatField,
) -> PhysicalIntegrityRejection {
    field_damage(
        scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        range,
        field,
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn malformed_payload(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    damaged(
        scope,
        PhysicalDamageCause::MalformedStructure,
        scope.byte_range(),
        Some(PhysicalFormatField::Payload),
        PhysicalBlastRadius::CompleteArtifact,
    )
}

pub(super) fn membership_body_range(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> PhysicalByteRange {
    PhysicalByteRange::new(
        scope.byte_range().offset() + 88,
        u64::try_from(bytes.len() - 88).expect("bounded length fits u64"),
    )
    .expect("validated membership blocks have a nonempty body")
}

fn body_item_range(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    width: usize,
    index: usize,
) -> PhysicalByteRange {
    let available = bytes.len().saturating_sub(88 + index * width);
    let length = available.min(width).max(1);
    PhysicalByteRange::new(
        scope.byte_range().offset() + 88 + (index * width) as u64,
        length as u64,
    )
    .expect("bounded membership item range is nonempty")
}

fn read_u16(bytes: &[u8], range: DurableFrameFieldRange) -> u16 {
    u16::from_le_bytes(range.bytes(bytes).try_into().unwrap())
}

fn read_u64(bytes: &[u8], range: DurableFrameFieldRange) -> u64 {
    u64::from_le_bytes(range.bytes(bytes).try_into().unwrap())
}

fn read_at_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
