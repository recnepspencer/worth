use worth_store_physical_format::{
    PhysicalRecordFormatDeclaration, DURABLE_EXTENT_FRAME_HEADER_BYTES, EXTENT_CHUNK_METADATA_BYTES,
};

use crate::artifact::durable_frame_rejection::{damaged, field_damage, DurableFrameFieldRange};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::validation::{
    IntegrityValidatedExtentMembership, PhysicalArtifactScope, PhysicalIntegrityRejection,
};

const FORMAT_DECLARATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
const CHUNK_ORDINAL_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
const RECORD_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 24);
const EXTENT_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(72, 8);
const EXTENT_GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(80, 8);
const LOGICAL_BYTES_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(88, 8);
const LOGICAL_OFFSET_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(96, 8);
const CHUNK_LENGTH_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(104, 4);

pub(super) fn validate_manifest_membership(
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    chunk_bytes: usize,
    manifest: IntegrityValidatedExtentMembership,
) -> Option<PhysicalIntegrityRejection> {
    let coordinate = scope
        .extent_chunk_coordinate()
        .expect("extent-chunk scope carries a coordinate");
    if scope.store_identity() != manifest.scope().store_identity() {
        return Some(damaged(
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    if record_format != manifest.record_format() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::FormatMismatch,
            FORMAT_DECLARATION_FIELD,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    if coordinate.record() != manifest.record() {
        return Some(identity_damage(
            scope,
            RECORD_IDENTITY_FIELD,
            PhysicalFormatField::RecordIdentity,
        ));
    }
    if coordinate.extent_cell().extent_id() != manifest.extent_cell().extent_id() {
        return Some(identity_damage(
            scope,
            EXTENT_IDENTITY_FIELD,
            PhysicalFormatField::ExtentIdentity,
        ));
    }
    if coordinate.extent_cell().generation() != manifest.extent_cell().generation() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            EXTENT_GENERATION_FIELD,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    if coordinate.logical_bytes() != manifest.logical_bytes() {
        return Some(membership_damage(
            scope,
            LOGICAL_BYTES_FIELD,
            PhysicalFormatField::Payload,
        ));
    }
    let Some(membership) = manifest.chunk_membership(coordinate.ordinal()) else {
        return Some(membership_damage(
            scope,
            CHUNK_ORDINAL_FIELD,
            PhysicalFormatField::ChunkOrdinal,
        ));
    };
    if coordinate.logical_offset() != membership.coordinate().logical_offset() {
        return Some(membership_damage(
            scope,
            LOGICAL_OFFSET_FIELD,
            PhysicalFormatField::ChildReference,
        ));
    }
    let expected_frame_bytes =
        u64::try_from(DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES)
            .expect("extent framing width fits u64")
            .checked_add(membership.payload_bytes())
            .expect("canonical extent frame length is bounded");
    if scope.byte_range().length() != expected_frame_bytes
        || u64::try_from(chunk_bytes).ok() != Some(membership.payload_bytes())
    {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            CHUNK_LENGTH_FIELD,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ));
    }
    None
}

fn identity_damage(
    scope: PhysicalArtifactScope,
    field: DurableFrameFieldRange,
    format_field: PhysicalFormatField,
) -> PhysicalIntegrityRejection {
    field_damage(
        scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        field,
        format_field,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn membership_damage(
    scope: PhysicalArtifactScope,
    field: DurableFrameFieldRange,
    format_field: PhysicalFormatField,
) -> PhysicalIntegrityRejection {
    field_damage(
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        field,
        format_field,
        PhysicalBlastRadius::CompleteArtifact,
    )
}
