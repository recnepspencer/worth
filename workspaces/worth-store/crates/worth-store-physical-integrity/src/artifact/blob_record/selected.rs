use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::BlobRecordDenial;

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization,
    PhysicalFormatField,
};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedBlobRecord, PhysicalArtifactScope, PhysicalIntegrityRejection,
    PhysicalIntegrityVersionAxis, UnsupportedPhysicalIntegrityVersion, UntrustedPhysicalArtifact,
};

use super::{validate_blob_record_payload_only, BlobRecordPayloadValidationDenial};

#[derive(Debug)]
pub enum BlobRecordIntegrityValidation<'media> {
    Intact(IntegrityValidatedBlobRecord<'media>),
    Rejected(PhysicalIntegrityRejection),
}

/// Checks one selected-record inner payload against a descriptive exact scope.
/// The caller still owns selected C.5 routing and outer-frame authentication.
pub fn validate_blob_record<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    BlobRecordIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if !matches!(
        scope.artifact_family(),
        PhysicalIntegrityArtifactFamily::BlobChunkFrame
            | PhysicalIntegrityArtifactFamily::BlobTreeNode
            | PhysicalIntegrityArtifactFamily::BlobGenerationPublication
    ) {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let (_, expected_kind) = scope
        .blob_record_identity()
        .expect("blob artifact scope carries record identity and kind");
    let record = match validate_blob_record_payload_only(
        scope.store_identity().bytes(),
        expected_kind,
        artifact.bytes(),
    ) {
        Ok(record) => record,
        Err(denial) => {
            return rejected(payload_denial(scope, denial, artifact.bytes()), byte_count)
        }
    };
    let validated = IntegrityValidatedBlobRecord::new(scope, record, artifact)
        .expect("decoded blob record satisfies its exact scope");
    (
        BlobRecordIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(scope.artifact_family(), byte_count),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    BlobRecordIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let scope = rejection.scope();
    (
        BlobRecordIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            scope.artifact_family(),
            byte_count,
            rejection,
        ),
    )
}

fn payload_denial(
    scope: PhysicalArtifactScope,
    denial: BlobRecordPayloadValidationDenial,
    bytes: &[u8],
) -> PhysicalIntegrityRejection {
    match denial {
        BlobRecordPayloadValidationDenial::StoreScopeMismatch => damage_field(
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            PhysicalFormatField::StoreIdentity,
            48,
            16,
        ),
        BlobRecordPayloadValidationDenial::WrongFamily => damage_field(
            scope,
            PhysicalDamageCause::RecordKindMismatch,
            PhysicalFormatField::ArtifactFamily,
            8,
            1,
        ),
        BlobRecordPayloadValidationDenial::Format(BlobRecordDenial::UnsupportedVersion) => {
            PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
                scope,
                PhysicalIntegrityVersionAxis::BlobRecord,
                u32::from(bytes[9]),
            ))
        }
        BlobRecordPayloadValidationDenial::Format(BlobRecordDenial::WrongMagic) => damage_field(
            scope,
            PhysicalDamageCause::WrongMagic,
            PhysicalFormatField::Magic,
            0,
            8,
        ),
        BlobRecordPayloadValidationDenial::Format(BlobRecordDenial::UnknownKind) => damage_field(
            scope,
            PhysicalDamageCause::RecordKindMismatch,
            PhysicalFormatField::ArtifactFamily,
            8,
            1,
        ),
        BlobRecordPayloadValidationDenial::Format(BlobRecordDenial::IntegrityMismatch) => {
            whole_damage(scope, PhysicalDamageCause::ChecksumMismatch)
        }
        BlobRecordPayloadValidationDenial::Format(BlobRecordDenial::Truncated) => {
            whole_damage(scope, PhysicalDamageCause::Truncated)
        }
        BlobRecordPayloadValidationDenial::Format(
            BlobRecordDenial::LengthMismatch | BlobRecordDenial::FrameTooLarge,
        ) => whole_damage(scope, PhysicalDamageCause::FramingLengthMismatch),
        BlobRecordPayloadValidationDenial::Format(_) => {
            whole_damage(scope, PhysicalDamageCause::MalformedStructure)
        }
    }
}

fn damage_field(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    field: PhysicalFormatField,
    offset: u64,
    length: u64,
) -> PhysicalIntegrityRejection {
    let range = PhysicalByteRange::new(scope.byte_range().offset() + offset, length)
        .expect("decoded blob field has a bounded nonempty range");
    PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
        scope,
        cause,
        range,
        Some(field),
        PhysicalBlastRadius::CanonicalFrame,
    ))
}

fn whole_damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
) -> PhysicalIntegrityRejection {
    PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
        scope,
        cause,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    ))
}
