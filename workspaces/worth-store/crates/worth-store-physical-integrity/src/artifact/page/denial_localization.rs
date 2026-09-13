use worth_store_physical_format::{
    InlinePageDenial, InlinePageGeometry, PersistedRecordIdentity,
    DURABLE_INLINE_PAGE_PREFIX_BYTES, DURABLE_INLINE_SLOT_BYTES,
};

use crate::artifact::durable_frame_rejection::{
    damaged, field_damage, from_frame_denial, DurableFrameFieldRange,
};
use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField,
};
use crate::validation::{PhysicalArtifactScope, PhysicalIntegrityRejection};

const FRAME_FORMAT: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
const PAYLOAD_LENGTH: DurableFrameFieldRange = DurableFrameFieldRange::new(24, 4);
const FRAME_GENERATION: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
const SEGMENT_IDENTITY: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 8);
const PAGE_IDENTITY: DurableFrameFieldRange = DurableFrameFieldRange::new(56, 8);
const SLOT_COUNT: DurableFrameFieldRange = DurableFrameFieldRange::new(64, 2);
const PAGE_RESERVED: DurableFrameFieldRange = DurableFrameFieldRange::new(66, 6);
const FRAME_HEADER_BYTES: usize = 48;

pub(super) fn page_integrity_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: InlinePageDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        InlinePageDenial::Frame(denial) => from_frame_denial(scope, denial),
        InlinePageDenial::InvalidPageIdentity => invalid_page_identity(scope, bytes),
        InlinePageDenial::InvalidGeometry => invalid_geometry(scope, bytes),
        InlinePageDenial::InvalidRecordIdentity => invalid_record_identity(scope, bytes),
        InlinePageDenial::ReservedFieldNonZero => noncanonical_gap(scope, bytes),
        InlinePageDenial::PageFull
        | InlinePageDenial::InvalidSlot
        | InlinePageDenial::RecordIdentityMismatch
        | InlinePageDenial::SlotGenerationMismatch => malformed_payload(scope),
    }
}

pub(super) fn page_identity_mismatch(
    scope: PhysicalArtifactScope,
    geometry: InlinePageGeometry,
) -> PhysicalIntegrityRejection {
    let expected = scope
        .page_identity()
        .expect("page-family scope carries a page identity");
    if geometry.segment() != expected.segment_id() {
        return field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            SEGMENT_IDENTITY,
            PhysicalFormatField::SegmentIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    if geometry.page() != expected.page_id() {
        return field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PAGE_IDENTITY,
            PhysicalFormatField::PageIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    field_damage(
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        FRAME_GENERATION,
        PhysicalFormatField::PhysicalGeneration,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn invalid_page_identity(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let expected = scope
        .page_identity()
        .expect("page-family scope carries a page identity");
    let observed_segment = read_u64(bytes, SEGMENT_IDENTITY);
    if observed_segment != expected.segment_id().get() {
        return field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            SEGMENT_IDENTITY,
            PhysicalFormatField::SegmentIdentity,
            identity_blast_radius(observed_segment),
        );
    }
    let observed_page = read_u64(bytes, PAGE_IDENTITY);
    if observed_page != expected.page_id().get() {
        return field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PAGE_IDENTITY,
            PhysicalFormatField::PageIdentity,
            identity_blast_radius(observed_page),
        );
    }
    if read_u64(bytes, FRAME_GENERATION) == 0 {
        return field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            FRAME_GENERATION,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::CanonicalFrame,
        );
    }
    field_damage(
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        FRAME_GENERATION,
        PhysicalFormatField::PhysicalGeneration,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

const fn identity_blast_radius(observed: u64) -> PhysicalBlastRadius {
    if observed == 0 {
        PhysicalBlastRadius::CanonicalFrame
    } else {
        PhysicalBlastRadius::CompleteArtifact
    }
}

fn invalid_geometry(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    if FRAME_FORMAT.bytes(bytes) != scope.record_format().canonical_identity_bytes() {
        return field_damage(
            scope,
            PhysicalDamageCause::FormatMismatch,
            FRAME_FORMAT,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    if bytes.len() != scope.record_format().page_size().bytes() as usize {
        return field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            PAYLOAD_LENGTH,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        );
    }
    if PAGE_RESERVED.bytes(bytes) != [0; 6] {
        return field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            PAGE_RESERVED,
            PhysicalFormatField::Reserved,
            PhysicalBlastRadius::CanonicalFrame,
        );
    }
    let payload_bytes = bytes.len() - FRAME_HEADER_BYTES;
    let slot_count = read_u16(bytes, SLOT_COUNT);
    let directory_end = DURABLE_INLINE_PAGE_PREFIX_BYTES
        .saturating_add(usize::from(slot_count).saturating_mul(DURABLE_INLINE_SLOT_BYTES));
    if directory_end > payload_bytes {
        return field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            SLOT_COUNT,
            PhysicalFormatField::Payload,
            PhysicalBlastRadius::CanonicalFrame,
        );
    }
    invalid_slot_geometry(scope, bytes, slot_count, payload_bytes, directory_end)
}

fn invalid_slot_geometry(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    slot_count: u16,
    payload_bytes: usize,
    directory_end: usize,
) -> PhysicalIntegrityRejection {
    let mut identities = std::collections::BTreeSet::new();
    let mut previous_start = payload_bytes;
    for slot in 0..usize::from(slot_count) {
        let fields = SlotFields::at(slot);
        let record = record_identity(bytes, fields).expect("record identity denial is separate");
        if read_u64(bytes, fields.generation) == 0 {
            return field_damage(
                scope,
                PhysicalDamageCause::PhysicalGenerationMismatch,
                fields.generation,
                PhysicalFormatField::PhysicalGeneration,
                PhysicalBlastRadius::CanonicalFrame,
            );
        }
        if !identities.insert(record) {
            return field_damage(
                scope,
                PhysicalDamageCause::MalformedStructure,
                fields.record,
                PhysicalFormatField::RecordIdentity,
                PhysicalBlastRadius::CanonicalFrame,
            );
        }
        let offset = read_u32(bytes, fields.offset) as usize;
        let length = read_u32(bytes, fields.length) as usize;
        let end = offset.saturating_add(length);
        if offset < directory_end || end > previous_start || end > payload_bytes {
            return field_damage(
                scope,
                PhysicalDamageCause::MalformedStructure,
                fields.location,
                PhysicalFormatField::Payload,
                PhysicalBlastRadius::CanonicalFrame,
            );
        }
        previous_start = offset;
    }
    malformed_payload(scope)
}

fn invalid_record_identity(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> PhysicalIntegrityRejection {
    let count = read_u16(bytes, SLOT_COUNT);
    for slot in 0..usize::from(count) {
        let fields = SlotFields::at(slot);
        if record_identity(bytes, fields).is_none() {
            return field_damage(
                scope,
                PhysicalDamageCause::ArtifactIdentityMismatch,
                fields.record,
                PhysicalFormatField::RecordIdentity,
                PhysicalBlastRadius::CanonicalFrame,
            );
        }
    }
    malformed_payload(scope)
}

fn noncanonical_gap(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    if PAGE_RESERVED.bytes(bytes) != [0; 6] {
        return field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            PAGE_RESERVED,
            PhysicalFormatField::Reserved,
            PhysicalBlastRadius::CanonicalFrame,
        );
    }
    if let Some(offset) = first_nonzero_gap(bytes) {
        let range = PhysicalByteRange::new(scope.byte_range().offset() + offset as u64, 1)
            .expect("one damaged gap byte is a valid range");
        return damaged(
            scope,
            PhysicalDamageCause::MalformedStructure,
            range,
            Some(PhysicalFormatField::Reserved),
            PhysicalBlastRadius::DamagedRange,
        );
    }
    malformed_payload(scope)
}

fn first_nonzero_gap(bytes: &[u8]) -> Option<usize> {
    let payload = &bytes[FRAME_HEADER_BYTES..];
    let count = read_u16(bytes, SLOT_COUNT);
    let directory_end =
        DURABLE_INLINE_PAGE_PREFIX_BYTES + usize::from(count) * DURABLE_INLINE_SLOT_BYTES;
    let mut previous_start = payload.len();
    for slot in 0..usize::from(count) {
        let fields = SlotFields::at(slot);
        let offset = read_u32(bytes, fields.offset) as usize;
        let length = read_u32(bytes, fields.length) as usize;
        let end = offset + length;
        if let Some(relative) = payload[end..previous_start]
            .iter()
            .position(|byte| *byte != 0)
        {
            return Some(FRAME_HEADER_BYTES + end + relative);
        }
        previous_start = offset;
    }
    payload[directory_end..previous_start]
        .iter()
        .position(|byte| *byte != 0)
        .map(|relative| FRAME_HEADER_BYTES + directory_end + relative)
}

fn malformed_payload(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    damaged(
        scope,
        PhysicalDamageCause::MalformedStructure,
        scope.byte_range(),
        Some(PhysicalFormatField::Payload),
        PhysicalBlastRadius::CanonicalFrame,
    )
}

#[derive(Clone, Copy)]
struct SlotFields {
    record: DurableFrameFieldRange,
    offset: DurableFrameFieldRange,
    length: DurableFrameFieldRange,
    location: DurableFrameFieldRange,
    generation: DurableFrameFieldRange,
}

impl SlotFields {
    fn at(slot: usize) -> Self {
        let base = FRAME_HEADER_BYTES
            + DURABLE_INLINE_PAGE_PREFIX_BYTES
            + slot * DURABLE_INLINE_SLOT_BYTES;
        let base = base as u64;
        Self {
            record: DurableFrameFieldRange::new(base, 24),
            offset: DurableFrameFieldRange::new(base + 24, 4),
            length: DurableFrameFieldRange::new(base + 28, 4),
            location: DurableFrameFieldRange::new(base + 24, 8),
            generation: DurableFrameFieldRange::new(base + 32, 8),
        }
    }
}

fn record_identity(bytes: &[u8], fields: SlotFields) -> Option<PersistedRecordIdentity> {
    let encoded = fields.record.bytes(bytes);
    PersistedRecordIdentity::new(
        encoded[..16]
            .try_into()
            .expect("record epoch width is fixed"),
        u64::from_le_bytes(
            encoded[16..24]
                .try_into()
                .expect("record ordinal width is fixed"),
        ),
    )
}

fn read_u16(bytes: &[u8], field: DurableFrameFieldRange) -> u16 {
    u16::from_le_bytes(
        field
            .bytes(bytes)
            .try_into()
            .expect("u16 field width is fixed"),
    )
}

fn read_u32(bytes: &[u8], field: DurableFrameFieldRange) -> u32 {
    u32::from_le_bytes(
        field
            .bytes(bytes)
            .try_into()
            .expect("u32 field width is fixed"),
    )
}

fn read_u64(bytes: &[u8], field: DurableFrameFieldRange) -> u64 {
    u64::from_le_bytes(
        field
            .bytes(bytes)
            .try_into()
            .expect("u64 field width is fixed"),
    )
}
