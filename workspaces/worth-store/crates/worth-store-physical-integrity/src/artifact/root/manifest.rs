use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, maximum_current_root_entries, DurablePhysicalRootManifest,
    RootManifestDenial,
};

use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedRootManifest, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use crate::artifact::durable_frame_rejection::{
    damaged, field_damage, from_frame_denial, input_length, wrong_scope, DurableFrameFieldRange,
};

const FORMAT_DECLARATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
const ENVELOPE_GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(28, 8);
const PAYLOAD_GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(48, 8);
const NODE_CAPACITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(64, 2);
const LAST_RECORD_IDENTITY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(176, 24);
const ROUTING_ROOT_PRESENCE_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(88, 1);
const ROUTING_ROOT_GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(96, 8);
const SEGMENT_ROOT_PRESENCE_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(208, 1);
const SEGMENT_ROOT_GENERATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(216, 8);
const FREE_SPACE_ROOT_PRESENCE_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(280, 1);
const FREE_SPACE_ROOT_GENERATION_FIELD: DurableFrameFieldRange =
    DurableFrameFieldRange::new(288, 8);

#[derive(Debug)]
pub enum RootManifestIntegrityValidation<'media> {
    Intact(IntegrityValidatedRootManifest<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_root_manifest<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    RootManifestIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::RootManifest {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let (manifest, record_format) =
        match DurablePhysicalRootManifest::decode(artifact.bytes(), u16::MAX) {
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
    if manifest.node_capacity() > maximum_current_root_entries(record_format) {
        return rejected(
            manifest_denial(
                scope,
                artifact.bytes(),
                RootManifestDenial::EntryLimitExceeded,
            ),
            byte_count,
        );
    }
    if Some(manifest.generation()) != scope.root_generation() {
        return rejected(
            field_damage(
                scope,
                PhysicalDamageCause::PhysicalGenerationMismatch,
                ENVELOPE_GENERATION_FIELD,
                PhysicalFormatField::PhysicalGeneration,
                PhysicalBlastRadius::ReachableSubtree,
            ),
            byte_count,
        );
    }
    intact(artifact, scope, manifest, record_format, byte_count)
}

fn intact<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
    manifest: DurablePhysicalRootManifest,
    record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    byte_count: u64,
) -> (
    RootManifestIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let validated = IntegrityValidatedRootManifest::new(
        scope,
        manifest,
        record_format,
        byte_range_checksum,
        artifact,
    )
    .expect("validated root manifest satisfies the sealed-view contract");
    (
        RootManifestIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::RootManifest,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    RootManifestIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        RootManifestIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::RootManifest,
            byte_count,
            rejection,
        ),
    )
}

fn manifest_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: RootManifestDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        RootManifestDenial::Frame(denial) => from_frame_denial(scope, denial),
        RootManifestDenial::MalformedPrefix
        | RootManifestDenial::MalformedEntryLength
        | RootManifestDenial::ReservedFieldNonZero => damaged(
            scope,
            PhysicalDamageCause::MalformedStructure,
            scope.byte_range(),
            Some(PhysicalFormatField::Payload),
            PhysicalBlastRadius::CompleteArtifact,
        ),
        RootManifestDenial::IdentityMismatch => generation_identity_mismatch(scope, bytes),
        RootManifestDenial::EntryLimitExceeded => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            NODE_CAPACITY_FIELD,
            PhysicalFormatField::Payload,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        RootManifestDenial::InvalidRecordIdentity => field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            LAST_RECORD_IDENTITY_FIELD,
            PhysicalFormatField::ArtifactIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        ),
        RootManifestDenial::InvalidPlacement => invalid_placement(scope, bytes),
    }
}

fn invalid_placement(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let manifest_generation = read_u64(bytes, PAYLOAD_GENERATION_FIELD);
    let child_generation_fields = [
        (ROUTING_ROOT_PRESENCE_FIELD, ROUTING_ROOT_GENERATION_FIELD),
        (SEGMENT_ROOT_PRESENCE_FIELD, SEGMENT_ROOT_GENERATION_FIELD),
        (
            FREE_SPACE_ROOT_PRESENCE_FIELD,
            FREE_SPACE_ROOT_GENERATION_FIELD,
        ),
    ];
    for (presence_field, generation_field) in child_generation_fields {
        let child_generation = read_u64(bytes, generation_field);
        if presence_field.bytes(bytes) == [1]
            && (child_generation == 0 || child_generation > manifest_generation)
        {
            return field_damage(
                scope,
                PhysicalDamageCause::ChildReferenceMismatch,
                generation_field,
                PhysicalFormatField::ChildReference,
                PhysicalBlastRadius::ReachableSubtree,
            );
        }
    }
    damaged(
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        scope.byte_range(),
        Some(PhysicalFormatField::ChildReference),
        PhysicalBlastRadius::ReachableSubtree,
    )
}

fn generation_identity_mismatch(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> PhysicalIntegrityRejection {
    let expected = scope
        .root_generation()
        .expect("root-manifest scope carries its generation");
    let envelope = read_u64(bytes, ENVELOPE_GENERATION_FIELD);
    let payload = read_u64(bytes, PAYLOAD_GENERATION_FIELD);
    let field = match (envelope == expected, payload == expected) {
        (true, false) => Some(PAYLOAD_GENERATION_FIELD),
        (false, true) => Some(ENVELOPE_GENERATION_FIELD),
        _ => None,
    };
    match field {
        Some(field) => field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            field,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::ReachableSubtree,
        ),
        None => damaged(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::PhysicalGeneration),
            PhysicalBlastRadius::ReachableSubtree,
        ),
    }
}

fn read_u64(bytes: &[u8], field: DurableFrameFieldRange) -> u64 {
    u64::from_le_bytes(
        field
            .bytes(bytes)
            .try_into()
            .expect("validated root-manifest framing fixes field widths"),
    )
}
