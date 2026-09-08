use worth_store_physical_format::{DurableFrameDenial, PhysicalRecordFormatDenial};

use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization,
    PhysicalFormatField,
};
use crate::validation::{
    PhysicalArtifactScope, PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
    UnsupportedPhysicalIntegrityVersion,
};

const MAGIC_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(0, 8);
const ARTIFACT_FAMILY_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(8, 1);
const FORMAT_DECLARATION_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(10, 10);
const ENCODED_LENGTH_FIELDS: DurableFrameFieldRange = DurableFrameFieldRange::new(20, 8);
const HEADER_RESERVED_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(22, 2);
const NON_DATA_PAGE_LSN_FIELD: DurableFrameFieldRange = DurableFrameFieldRange::new(36, 8);

#[derive(Debug, Clone, Copy)]
pub(crate) struct DurableFrameFieldRange {
    offset: u64,
    length: u64,
}

impl DurableFrameFieldRange {
    pub(crate) const fn new(offset: u64, length: u64) -> Self {
        Self { offset, length }
    }

    pub(crate) fn bytes(self, artifact: &[u8]) -> &[u8] {
        let start = usize::try_from(self.offset).expect("field offset fits platform usize");
        let length = usize::try_from(self.length).expect("field length fits platform usize");
        &artifact[start..start + length]
    }
}

pub(crate) fn wrong_scope(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    damaged(
        scope,
        PhysicalDamageCause::FamilyMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

pub(crate) fn input_length(
    scope: PhysicalArtifactScope,
    observed_length: u64,
) -> Option<PhysicalIntegrityRejection> {
    let expected_length = scope.byte_range().length();
    if observed_length < expected_length {
        let missing = PhysicalByteRange::new(
            scope.byte_range().offset() + observed_length,
            expected_length - observed_length,
        )
        .expect("missing tail remains inside an admitted scope range");
        return Some(damaged(
            scope,
            PhysicalDamageCause::Truncated,
            missing,
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ));
    }
    (observed_length > expected_length).then(|| {
        damaged(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::EncodedLength),
            PhysicalBlastRadius::CanonicalFrame,
        )
    })
}

pub(crate) fn from_frame_denial(
    scope: PhysicalArtifactScope,
    denial: DurableFrameDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        DurableFrameDenial::Truncated => damaged(
            scope,
            PhysicalDamageCause::Truncated,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        DurableFrameDenial::WrongMagic => field_damage(
            scope,
            PhysicalDamageCause::WrongMagic,
            MAGIC_FIELD,
            PhysicalFormatField::Magic,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        DurableFrameDenial::IllegalKind(_) => field_damage(
            scope,
            PhysicalDamageCause::FamilyMismatch,
            ARTIFACT_FAMILY_FIELD,
            PhysicalFormatField::ArtifactFamily,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        DurableFrameDenial::UnsupportedSchema(observed) => {
            PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
                scope,
                PhysicalIntegrityVersionAxis::EnvelopeSchema,
                u32::from(observed),
            ))
        }
        DurableFrameDenial::UnsupportedFormat(PhysicalRecordFormatDenial::UnsupportedVersion(
            observed,
        )) => PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
            scope,
            PhysicalIntegrityVersionAxis::PhysicalFormat,
            u32::from(observed),
        )),
        DurableFrameDenial::UnsupportedFormat(_) => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            FORMAT_DECLARATION_FIELD,
            PhysicalFormatField::FormatDeclaration,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        DurableFrameDenial::ReservedFieldNonZero => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            HEADER_RESERVED_FIELD,
            PhysicalFormatField::Reserved,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        DurableFrameDenial::NonDataPageLsnNonZero => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            NON_DATA_PAGE_LSN_FIELD,
            PhysicalFormatField::Reserved,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        DurableFrameDenial::LengthMismatch => field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            ENCODED_LENGTH_FIELDS,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        // The canonical CRC covers both the stored checksum and covered bytes. Once it differs,
        // this layer has no independent evidence that distinguishes a K mutation from a B mutation.
        DurableFrameDenial::IntegrityMismatch => damaged(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
    }
}

pub(crate) fn field_damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: DurableFrameFieldRange,
    field: PhysicalFormatField,
    blast_radius: PhysicalBlastRadius,
) -> PhysicalIntegrityRejection {
    let damaged_range =
        PhysicalByteRange::new(scope.byte_range().offset() + range.offset, range.length)
            .expect("format field remains inside an admitted scope range");
    damaged(scope, cause, damaged_range, Some(field), blast_radius)
}

pub(crate) const fn damaged(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) -> PhysicalIntegrityRejection {
    PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
        scope,
        cause,
        damaged_range,
        field,
        blast_radius,
    ))
}
