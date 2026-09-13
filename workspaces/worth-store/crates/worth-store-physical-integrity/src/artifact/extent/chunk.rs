use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    decode_extent_chunk, durable_artifact_checksum, ExtentFrameDenial,
};

use crate::artifact::durable_frame_rejection::{
    field_damage, from_frame_denial, input_length, wrong_scope, DurableFrameFieldRange,
};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentManifest,
    IntegrityValidatedExtentMembership, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::membership::validate_manifest_membership;

const CHUNK_ORDINAL_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
const RECORD_EPOCH_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 16);
const RECORD_ORDINAL_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(64, 8);
const RECORD_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 24);
const EXTENT_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(72, 8);
const EXTENT_GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(80, 8);
const LOGICAL_BYTES_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(88, 8);
const LOGICAL_OFFSET_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(96, 8);
const CHUNK_LENGTH_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(104, 4);
const RESERVED_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(108, 4);
const ENCODED_LENGTH_FIELDS: DurableFrameFieldRange = DurableFrameFieldRange::new(20, 8);

#[derive(Debug)]
pub enum ExtentChunkIntegrityValidation<'media> {
    Intact(IntegrityValidatedExtentChunkFrame<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_extent_chunk<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
    manifest: &IntegrityValidatedExtentManifest<'_>,
) -> (
    ExtentChunkIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    validate_extent_chunk_membership(artifact, scope, manifest.membership())
}

pub fn validate_extent_chunk_membership<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
    manifest: IntegrityValidatedExtentMembership,
) -> (
    ExtentChunkIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::ExtentChunk {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let coordinate = scope
        .extent_chunk_coordinate()
        .expect("extent-chunk scope carries a coordinate");
    let (chunk_bytes, record_format) = match decode_extent_chunk(artifact.bytes(), coordinate) {
        Ok(decoded) => decoded,
        Err(denial) => {
            return rejected(
                chunk_denial(scope, artifact.bytes(), coordinate, denial),
                byte_count,
            )
        }
    };
    if record_format != scope.record_format() {
        return rejected(
            field_damage(
                scope,
                PhysicalDamageCause::FormatMismatch,
                DurableFrameFieldRange::new(10, 10),
                PhysicalFormatField::FormatDeclaration,
                PhysicalBlastRadius::CompleteArtifact,
            ),
            byte_count,
        );
    }
    if let Some(rejection) =
        validate_manifest_membership(scope, record_format, chunk_bytes.len(), manifest)
    {
        return rejected(rejection, byte_count);
    }

    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let validated = IntegrityValidatedExtentChunkFrame::new(
        scope,
        record_format,
        chunk_bytes,
        byte_range_checksum,
        artifact,
    )
    .expect("validated extent chunk satisfies the sealed-view contract");
    (
        ExtentChunkIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::ExtentChunk,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    ExtentChunkIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        ExtentChunkIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::ExtentChunk,
            byte_count,
            rejection,
        ),
    )
}

fn chunk_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    expected: worth_store_physical_format::ExtentChunkCoordinate,
    denial: ExtentFrameDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        ExtentFrameDenial::Frame(denial) => from_frame_denial(scope, denial),
        ExtentFrameDenial::MalformedLength => malformed_chunk(scope, bytes, expected),
        ExtentFrameDenial::InvalidRecordIdentity => invalid_record(scope, bytes),
        ExtentFrameDenial::RecordIdentityMismatch => identity_damage(
            scope,
            RECORD_IDENTITY_FIELD,
            PhysicalFormatField::RecordIdentity,
        ),
        ExtentFrameDenial::GenerationMismatch => field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            EXTENT_GENERATION_FIELD,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        ExtentFrameDenial::PayloadTooLarge => field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            CHUNK_LENGTH_FIELD,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
    }
}

fn malformed_chunk(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    expected: worth_store_physical_format::ExtentChunkCoordinate,
) -> PhysicalIntegrityRejection {
    if bytes.len() < 112 {
        return field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            ENCODED_LENGTH_FIELDS,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        );
    }
    let (field, format_field, cause) = if read_u64(bytes, CHUNK_ORDINAL_FIELD)
        != u64::from(expected.ordinal())
    {
        (
            CHUNK_ORDINAL_FIELD,
            PhysicalFormatField::ChunkOrdinal,
            PhysicalDamageCause::SequenceMismatch,
        )
    } else if bytes[108..112] != [0; 4] {
        (
            RESERVED_FIELD,
            PhysicalFormatField::Reserved,
            PhysicalDamageCause::MalformedStructure,
        )
    } else if read_u64(bytes, EXTENT_IDENTITY_FIELD) != expected.extent_cell().extent_id().get() {
        (
            EXTENT_IDENTITY_FIELD,
            PhysicalFormatField::ExtentIdentity,
            PhysicalDamageCause::ArtifactIdentityMismatch,
        )
    } else if read_u64(bytes, LOGICAL_BYTES_FIELD) != expected.logical_bytes() {
        (
            LOGICAL_BYTES_FIELD,
            PhysicalFormatField::Payload,
            PhysicalDamageCause::ChildReferenceMismatch,
        )
    } else if read_u64(bytes, LOGICAL_OFFSET_FIELD) != expected.logical_offset() {
        (
            LOGICAL_OFFSET_FIELD,
            PhysicalFormatField::ChildReference,
            PhysicalDamageCause::ChildReferenceMismatch,
        )
    } else {
        (
            CHUNK_LENGTH_FIELD,
            PhysicalFormatField::EncodedLength,
            PhysicalDamageCause::FramingLengthMismatch,
        )
    };
    field_damage(
        scope,
        cause,
        field,
        format_field,
        PhysicalBlastRadius::CanonicalFrame,
    )
}

fn invalid_record(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let field = if bytes.get(48..64) == Some(&[0; 16]) {
        RECORD_EPOCH_FIELD
    } else {
        RECORD_ORDINAL_FIELD
    };
    identity_damage(scope, field, PhysicalFormatField::RecordIdentity)
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

fn read_u64(bytes: &[u8], field: DurableFrameFieldRange) -> u64 {
    u64::from_le_bytes(field.bytes(bytes).try_into().expect("fixed u64 field"))
}
