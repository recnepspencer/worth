use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, DurableExtentManifest, MembershipManifestDenial,
};

use crate::artifact::durable_frame_rejection::{
    damaged, field_damage, from_frame_denial, input_length, wrong_scope, DurableFrameFieldRange,
};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedExtentManifest, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

const FORMAT_DECLARATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
const ENVELOPE_GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
const RECORD_EPOCH_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 16);
const RECORD_ORDINAL_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(64, 8);
const RECORD_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 24);
const EXTENT_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(72, 8);
const LOGICAL_BYTES_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(80, 8);
const MAXIMUM_FRAME_BYTES_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(88, 4);
const CHUNK_COUNT_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(92, 4);
const RESERVED_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(96, 8);
const ENCODED_LENGTH_FIELDS: DurableFrameFieldRange = DurableFrameFieldRange::new(20, 8);

#[derive(Debug)]
pub enum ExtentManifestIntegrityValidation<'media> {
    Intact(IntegrityValidatedExtentManifest<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_extent_manifest<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    ExtentManifestIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::ExtentManifest {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let (manifest, record_format) = match DurableExtentManifest::decode(artifact.bytes()) {
        Ok(decoded) => decoded,
        Err(denial) => {
            return rejected(manifest_denial(scope, artifact.bytes(), denial), byte_count)
        }
    };
    if record_format != scope.record_format() {
        return rejected(
            field_damage(
                scope,
                PhysicalDamageCause::FormatMismatch,
                FORMAT_DECLARATION_FIELD,
                PhysicalFormatField::FormatDeclaration,
                PhysicalBlastRadius::CompleteArtifact,
            ),
            byte_count,
        );
    }
    if let Some(rejection) = placement_mismatch(scope, manifest) {
        return rejected(rejection, byte_count);
    }

    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let validated = IntegrityValidatedExtentManifest::new(
        scope,
        manifest,
        record_format,
        byte_range_checksum,
        artifact,
    )
    .expect("validated extent manifest satisfies the sealed-view contract");
    (
        ExtentManifestIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::ExtentManifest,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    ExtentManifestIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        ExtentManifestIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::ExtentManifest,
            byte_count,
            rejection,
        ),
    )
}

fn placement_mismatch(
    scope: PhysicalArtifactScope,
    manifest: DurableExtentManifest,
) -> Option<PhysicalIntegrityRejection> {
    let placement = scope
        .extent_manifest_placement()
        .expect("extent-manifest scope carries placement identity");
    if manifest.record() != placement.record() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            RECORD_IDENTITY_FIELD,
            PhysicalFormatField::RecordIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if manifest.extent() != placement.extent() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            EXTENT_IDENTITY_FIELD,
            PhysicalFormatField::ExtentIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if manifest.generation() != placement.extent_generation() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            ENVELOPE_GENERATION_FIELD,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    (manifest.logical_bytes() != placement.payload_bytes()).then(|| {
        field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            LOGICAL_BYTES_FIELD,
            PhysicalFormatField::Payload,
            PhysicalBlastRadius::ReachableSubtree,
        )
    })
}

fn manifest_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: MembershipManifestDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        MembershipManifestDenial::Frame(denial) => from_frame_denial(scope, denial),
        MembershipManifestDenial::Malformed => malformed_manifest(scope, bytes),
        MembershipManifestDenial::Limit => damaged(
            scope,
            PhysicalDamageCause::MalformedStructure,
            scope.byte_range(),
            Some(PhysicalFormatField::Payload),
            PhysicalBlastRadius::CompleteArtifact,
        ),
        MembershipManifestDenial::Reserved => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            RESERVED_FIELD,
            PhysicalFormatField::Reserved,
            PhysicalBlastRadius::CompleteArtifact,
        ),
    }
}

fn malformed_manifest(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    if bytes.len() != 104 {
        return field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            ENCODED_LENGTH_FIELDS,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        );
    }
    if FORMAT_DECLARATION_FIELD.bytes(bytes) != scope.record_format().canonical_identity_bytes() {
        return field_damage(
            scope,
            PhysicalDamageCause::FormatMismatch,
            FORMAT_DECLARATION_FIELD,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    let (field, format_field, cause, blast_radius) = if bytes[48..64] == [0; 16] {
        (
            RECORD_EPOCH_FIELD,
            PhysicalFormatField::RecordIdentity,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PhysicalBlastRadius::ReachableSubtree,
        )
    } else if read_u64(bytes, RECORD_ORDINAL_FIELD) == 0 {
        (
            RECORD_ORDINAL_FIELD,
            PhysicalFormatField::RecordIdentity,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PhysicalBlastRadius::ReachableSubtree,
        )
    } else if read_u64(bytes, EXTENT_IDENTITY_FIELD) == 0 {
        (
            EXTENT_IDENTITY_FIELD,
            PhysicalFormatField::ExtentIdentity,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            PhysicalBlastRadius::ReachableSubtree,
        )
    } else if read_u64(bytes, ENVELOPE_GENERATION_FIELD) == 0 {
        (
            ENVELOPE_GENERATION_FIELD,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            PhysicalBlastRadius::ReachableSubtree,
        )
    } else if read_u64(bytes, LOGICAL_BYTES_FIELD) == 0 {
        (
            LOGICAL_BYTES_FIELD,
            PhysicalFormatField::Payload,
            PhysicalDamageCause::MalformedStructure,
            PhysicalBlastRadius::ReachableSubtree,
        )
    } else if bytes[96..104] != [0; 8] {
        (
            RESERVED_FIELD,
            PhysicalFormatField::Reserved,
            PhysicalDamageCause::MalformedStructure,
            PhysicalBlastRadius::CompleteArtifact,
        )
    } else if read_u32(bytes, MAXIMUM_FRAME_BYTES_FIELD)
        != scope.record_format().page_size().bytes()
    {
        (
            MAXIMUM_FRAME_BYTES_FIELD,
            PhysicalFormatField::Payload,
            PhysicalDamageCause::FormatMismatch,
            PhysicalBlastRadius::ReachableSubtree,
        )
    } else {
        (
            CHUNK_COUNT_FIELD,
            PhysicalFormatField::Payload,
            PhysicalDamageCause::ChildReferenceMismatch,
            PhysicalBlastRadius::ReachableSubtree,
        )
    };
    field_damage(scope, cause, field, format_field, blast_radius)
}

fn read_u64(bytes: &[u8], field: DurableFrameFieldRange) -> u64 {
    u64::from_le_bytes(field.bytes(bytes).try_into().expect("fixed u64 field"))
}

fn read_u32(bytes: &[u8], field: DurableFrameFieldRange) -> u32 {
    u32::from_le_bytes(field.bytes(bytes).try_into().expect("fixed u32 field"))
}
