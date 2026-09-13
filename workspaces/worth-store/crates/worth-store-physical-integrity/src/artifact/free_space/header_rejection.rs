use worth_store_physical_format::FreeSpaceRoutingDenial;

use crate::artifact::durable_frame_rejection::{
    damaged, field_damage, from_frame_denial, DurableFrameFieldRange,
};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::validation::{PhysicalArtifactScope, PhysicalIntegrityRejection};

pub(super) struct HeaderField;

impl HeaderField {
    pub(super) const FORMAT: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
    pub(super) const ENVELOPE_IDENTITY: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
    pub(super) const GENERATION: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 8);
    pub(super) const TREE_IDENTITY: DurableFrameFieldRange = DurableFrameFieldRange::new(56, 8);
    pub(super) const NODE_CAPACITY: DurableFrameFieldRange = DurableFrameFieldRange::new(64, 2);
    const SEGMENT_PAGE_CAPACITY: DurableFrameFieldRange = DurableFrameFieldRange::new(66, 4);
    const ENTRY_COUNT: DurableFrameFieldRange = DurableFrameFieldRange::new(72, 8);
    const NEXT_SEGMENT: DurableFrameFieldRange = DurableFrameFieldRange::new(80, 8);
    const NEXT_PAGE: DurableFrameFieldRange = DurableFrameFieldRange::new(88, 8);
    const NEXT_EXTENT: DurableFrameFieldRange = DurableFrameFieldRange::new(96, 8);
    const NEXT_BLOCK: DurableFrameFieldRange = DurableFrameFieldRange::new(104, 8);
    const ROOT_PRESENCE: DurableFrameFieldRange = DurableFrameFieldRange::new(112, 1);
    pub(super) const ROOT_REFERENCE: DurableFrameFieldRange = DurableFrameFieldRange::new(112, 64);
    const RESERVED_PREFIX: DurableFrameFieldRange = DurableFrameFieldRange::new(70, 2);
    const RESERVED_ROOT: DurableFrameFieldRange = DurableFrameFieldRange::new(113, 7);
    const ABSENT_ROOT_REFERENCE: DurableFrameFieldRange = DurableFrameFieldRange::new(120, 56);
}

pub(super) fn free_space_header_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: FreeSpaceRoutingDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        FreeSpaceRoutingDenial::Frame(denial) => from_frame_denial(scope, denial),
        FreeSpaceRoutingDenial::Malformed => malformed_header(scope, bytes),
        FreeSpaceRoutingDenial::IdentityOrCapacity => header_identity_or_capacity(scope, bytes),
        FreeSpaceRoutingDenial::InvalidReference => field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            HeaderField::ROOT_REFERENCE,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::ReachableSubtree,
        ),
        FreeSpaceRoutingDenial::CanonicalOrder => malformed_payload(scope),
    }
}

fn malformed_header(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    if bytes.len() != 176 {
        return field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            DurableFrameFieldRange::new(20, 8),
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        );
    }
    if let Some(rejection) = header_reserved_damage(scope, bytes) {
        return rejection;
    }
    if let Some(rejection) = header_scope_identity_damage(scope, bytes) {
        return rejection;
    }
    if HeaderField::ROOT_PRESENCE.bytes(bytes)[0] > 1 {
        return field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            HeaderField::ROOT_PRESENCE,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    if let Some(rejection) = required_header_field_damage(scope, bytes) {
        return rejection;
    }
    header_shape_damage(scope, bytes)
}

fn header_reserved_damage(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> Option<PhysicalIntegrityRejection> {
    let absent_root = (HeaderField::ROOT_PRESENCE.bytes(bytes)[0] == 0
        && read_u64(bytes, HeaderField::ENTRY_COUNT) == 0)
        .then_some(HeaderField::ABSENT_ROOT_REFERENCE);
    [HeaderField::RESERVED_PREFIX, HeaderField::RESERVED_ROOT]
        .into_iter()
        .chain(absent_root)
        .find(|range| range.bytes(bytes).iter().any(|byte| *byte != 0))
        .map(|range| {
            field_damage(
                scope,
                PhysicalDamageCause::MalformedStructure,
                range,
                PhysicalFormatField::Reserved,
                PhysicalBlastRadius::CompleteArtifact,
            )
        })
}

fn required_header_field_damage(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> Option<PhysicalIntegrityRejection> {
    let required_nonzero = [
        (
            HeaderField::GENERATION,
            PhysicalFormatField::PhysicalGeneration,
        ),
        (
            HeaderField::TREE_IDENTITY,
            PhysicalFormatField::TreeIdentity,
        ),
        (
            HeaderField::NEXT_SEGMENT,
            PhysicalFormatField::AllocationFrontier,
        ),
        (
            HeaderField::NEXT_PAGE,
            PhysicalFormatField::AllocationFrontier,
        ),
        (
            HeaderField::NEXT_EXTENT,
            PhysicalFormatField::AllocationFrontier,
        ),
        (
            HeaderField::NEXT_BLOCK,
            PhysicalFormatField::AllocationFrontier,
        ),
    ];
    required_nonzero
        .into_iter()
        .find(|(range, _)| read_u64(bytes, *range) == 0)
        .map(|(range, field)| {
            field_damage(
                scope,
                PhysicalDamageCause::MalformedStructure,
                range,
                field,
                PhysicalBlastRadius::CompleteArtifact,
            )
        })
}

fn header_shape_damage(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    if read_u16(bytes, HeaderField::NODE_CAPACITY) < 2 {
        return field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            HeaderField::NODE_CAPACITY,
            PhysicalFormatField::NodeCapacity,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    if read_u32(bytes, HeaderField::SEGMENT_PAGE_CAPACITY) == 0 {
        return field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            HeaderField::SEGMENT_PAGE_CAPACITY,
            PhysicalFormatField::SegmentPageCapacity,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    let has_entries = read_u64(bytes, HeaderField::ENTRY_COUNT) != 0;
    let has_root = HeaderField::ROOT_PRESENCE.bytes(bytes)[0] == 1;
    if has_entries != has_root {
        let expected_has_root = scope
            .free_space_header_identity()
            .expect("free-space header scope carries identity")
            .root()
            .is_some();
        if has_root != expected_has_root {
            return field_damage(
                scope,
                PhysicalDamageCause::ChildReferenceMismatch,
                HeaderField::ROOT_REFERENCE,
                PhysicalFormatField::ChildReference,
                PhysicalBlastRadius::ReachableSubtree,
            );
        }
        return field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            HeaderField::ENTRY_COUNT,
            PhysicalFormatField::FreeSpaceEntryCount,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    if has_root {
        return field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            HeaderField::ROOT_REFERENCE,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::ReachableSubtree,
        );
    }
    malformed_payload(scope)
}

fn header_identity_or_capacity(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> PhysicalIntegrityRejection {
    if bytes.len() == 176 {
        if let Some(rejection) = header_scope_identity_damage(scope, bytes) {
            return rejection;
        }
        return field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            HeaderField::SEGMENT_PAGE_CAPACITY,
            PhysicalFormatField::SegmentPageCapacity,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    malformed_payload(scope)
}

fn header_scope_identity_damage(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> Option<PhysicalIntegrityRejection> {
    let expected = scope
        .free_space_header_identity()
        .expect("free-space header scope carries identity");
    for range in [HeaderField::ENVELOPE_IDENTITY, HeaderField::GENERATION] {
        if read_u64(bytes, range) != expected.generation().get() {
            return Some(field_damage(
                scope,
                PhysicalDamageCause::PhysicalGenerationMismatch,
                range,
                PhysicalFormatField::PhysicalGeneration,
                PhysicalBlastRadius::ReachableSubtree,
            ));
        }
    }
    (read_u64(bytes, HeaderField::TREE_IDENTITY) != expected.tree().get()).then(|| {
        field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            HeaderField::TREE_IDENTITY,
            PhysicalFormatField::TreeIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        )
    })
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

fn read_u16(bytes: &[u8], range: DurableFrameFieldRange) -> u16 {
    u16::from_le_bytes(range.bytes(bytes).try_into().unwrap())
}

fn read_u32(bytes: &[u8], range: DurableFrameFieldRange) -> u32 {
    u32::from_le_bytes(range.bytes(bytes).try_into().unwrap())
}

fn read_u64(bytes: &[u8], range: DurableFrameFieldRange) -> u64 {
    u64::from_le_bytes(range.bytes(bytes).try_into().unwrap())
}
