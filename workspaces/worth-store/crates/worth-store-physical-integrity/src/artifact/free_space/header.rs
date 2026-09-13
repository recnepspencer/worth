use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, maximum_current_root_entries, DurableFreeSpaceManifestHeader,
    FreeSpaceHeaderScopeIdentity, PhysicalRecordFormatDeclaration,
};

use crate::artifact::durable_frame_rejection::{field_damage, input_length, wrong_scope};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedFreeSpaceHeader, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::header_rejection::{free_space_header_denial, HeaderField};

#[derive(Debug)]
pub enum FreeSpaceHeaderIntegrityValidation<'media> {
    Intact(IntegrityValidatedFreeSpaceHeader<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_free_space_header<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    FreeSpaceHeaderIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::FreeSpaceHeader {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let (header, record_format) =
        match DurableFreeSpaceManifestHeader::decode(artifact.bytes(), u16::MAX) {
            Ok(decoded) => decoded,
            Err(denial) => {
                return rejected(
                    free_space_header_denial(scope, artifact.bytes(), denial),
                    byte_count,
                )
            }
        };
    let expected = scope
        .free_space_header_identity()
        .expect("free-space-header family scope carries its identity");
    if let Err(rejection) = validate_header_format(scope, record_format, &header) {
        return rejected(rejection, byte_count);
    }
    if let Err(rejection) = validate_header_identity(scope, expected, &header) {
        return rejected(rejection, byte_count);
    }
    let complete_checksum =
        match validate_complete_header_checksum(scope, expected, artifact.bytes()) {
            Ok(checksum) => checksum,
            Err(rejection) => return rejected(rejection, byte_count),
        };
    let validated = IntegrityValidatedFreeSpaceHeader::new(
        scope,
        header,
        record_format,
        complete_checksum,
        artifact,
    )
    .expect("validated free-space header satisfies the sealed-view contract");
    (
        FreeSpaceHeaderIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            byte_count,
        ),
    )
}

fn validate_header_format(
    scope: PhysicalArtifactScope,
    record_format: PhysicalRecordFormatDeclaration,
    header: &DurableFreeSpaceManifestHeader,
) -> Result<(), PhysicalIntegrityRejection> {
    if record_format != scope.record_format() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::FormatMismatch,
            HeaderField::FORMAT,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    if header.node_capacity() > maximum_current_root_entries(record_format) {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            HeaderField::NODE_CAPACITY,
            PhysicalFormatField::NodeCapacity,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    Ok(())
}

fn validate_header_identity(
    scope: PhysicalArtifactScope,
    expected: FreeSpaceHeaderScopeIdentity,
    header: &DurableFreeSpaceManifestHeader,
) -> Result<(), PhysicalIntegrityRejection> {
    if header.generation() != expected.generation().get() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            HeaderField::ENVELOPE_IDENTITY,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if header.tree_identity() != expected.tree().get() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            HeaderField::TREE_IDENTITY,
            PhysicalFormatField::TreeIdentity,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    if header.root() != expected.root() {
        return Err(field_damage(
            scope,
            PhysicalDamageCause::ChildReferenceMismatch,
            HeaderField::ROOT_REFERENCE,
            PhysicalFormatField::ChildReference,
            PhysicalBlastRadius::ReachableSubtree,
        ));
    }
    Ok(())
}

fn validate_complete_header_checksum(
    scope: PhysicalArtifactScope,
    expected: FreeSpaceHeaderScopeIdentity,
    bytes: &[u8],
) -> Result<u32, PhysicalIntegrityRejection> {
    let complete_checksum = durable_artifact_checksum(bytes);
    if complete_checksum != expected.complete_child_checksum().get() {
        return Err(crate::artifact::durable_frame_rejection::damaged(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::CompleteChildChecksum),
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    Ok(complete_checksum)
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    FreeSpaceHeaderIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        FreeSpaceHeaderIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
            byte_count,
            rejection,
        ),
    )
}
