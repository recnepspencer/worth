use worth_store_physical_format::physical_work_obligation::{
    PhysicalWorkObligationV6Denial, PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES,
};

use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization,
    PhysicalFormatField,
};
use crate::validation::{
    PhysicalArtifactScope, PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
    UnsupportedPhysicalIntegrityVersion,
};

const MAGIC: FieldRange = FieldRange::new(0, 8);
const OPERATION_FAMILY: FieldRange = FieldRange::new(9, 1);
const HEADER_RESERVED: FieldRange = FieldRange::new(10, 6);
const STORE_IDENTITY: FieldRange = FieldRange::new(16, 16);
const RUNTIME_IDENTITY: FieldRange = FieldRange::new(32, 8);
const GENERATION_IDENTITY: FieldRange = FieldRange::new(40, 8);
const OPERATION_IDENTITY: FieldRange = FieldRange::new(48, 8);
const TARGET_SHAPE: FieldRange = FieldRange::new(56, 72);
const PAYLOAD_DIGEST_PRESENCE: FieldRange = FieldRange::new(105, 1);
const TARGET_RESERVED: FieldRange = FieldRange::new(107, 5);

#[derive(Clone, Copy)]
pub(super) struct FieldRange {
    offset: u64,
    length: u64,
}

impl FieldRange {
    pub(super) const fn new(offset: u64, length: u64) -> Self {
        Self { offset, length }
    }
}

pub(super) fn wrong_scope(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    damage(
        scope,
        PhysicalDamageCause::FamilyMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

pub(super) fn invalid_scope_length(
    scope: PhysicalArtifactScope,
) -> Option<PhysicalIntegrityRejection> {
    (scope.byte_range().length() != PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES as u64).then(|| {
        damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::EncodedLength),
            PhysicalBlastRadius::CanonicalFrame,
        )
    })
}

pub(super) fn input_length(
    scope: PhysicalArtifactScope,
    observed: u64,
) -> Option<PhysicalIntegrityRejection> {
    let expected = PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES as u64;
    if observed < expected {
        return Some(damage(
            scope,
            PhysicalDamageCause::Truncated,
            PhysicalByteRange::new(scope.byte_range().offset() + observed, expected - observed)
                .expect("missing physical-work tail is bounded by the fixed v6 record"),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ));
    }
    (observed > expected).then(|| {
        damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::EncodedLength),
            PhysicalBlastRadius::CanonicalFrame,
        )
    })
}

pub(super) fn format_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: PhysicalWorkObligationV6Denial,
) -> PhysicalIntegrityRejection {
    match denial {
        PhysicalWorkObligationV6Denial::LengthMismatch => damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::EncodedLength),
            PhysicalBlastRadius::CanonicalFrame,
        ),
        PhysicalWorkObligationV6Denial::WrongMagic => field_damage(
            scope,
            PhysicalDamageCause::WrongMagic,
            MAGIC,
            PhysicalFormatField::Magic,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        PhysicalWorkObligationV6Denial::UnsupportedVersion(observed) => {
            PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
                scope,
                PhysicalIntegrityVersionAxis::PhysicalWorkObligation,
                u32::from(observed),
            ))
        }
        PhysicalWorkObligationV6Denial::ReservedFieldNonZero => reserved_damage(scope, bytes),
        PhysicalWorkObligationV6Denial::ChecksumMismatch => damage(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        PhysicalWorkObligationV6Denial::InvalidIdentity => invalid_identity(scope, bytes),
        PhysicalWorkObligationV6Denial::UnknownOperation(_) => field_damage(
            scope,
            PhysicalDamageCause::RecordKindMismatch,
            OPERATION_FAMILY,
            PhysicalFormatField::OperationFamily,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        PhysicalWorkObligationV6Denial::InvalidTarget => invalid_target(scope, bytes),
    }
}

pub(super) fn field_damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    field_range: FieldRange,
    field: PhysicalFormatField,
    blast_radius: PhysicalBlastRadius,
) -> PhysicalIntegrityRejection {
    damage(
        scope,
        cause,
        PhysicalByteRange::new(
            scope.byte_range().offset() + field_range.offset,
            field_range.length,
        )
        .expect("physical-work field remains inside the fixed v6 record"),
        Some(field),
        blast_radius,
    )
}

pub(super) const fn damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) -> PhysicalIntegrityRejection {
    PhysicalIntegrityRejection::Damaged(PhysicalDamageLocalization::new(
        scope,
        cause,
        range,
        field,
        blast_radius,
    ))
}

fn reserved_damage(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let range = if bytes[10..16].iter().any(|byte| *byte != 0) {
        HEADER_RESERVED
    } else {
        TARGET_RESERVED
    };
    field_damage(
        scope,
        PhysicalDamageCause::MalformedStructure,
        range,
        PhysicalFormatField::Reserved,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn invalid_identity(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let (range, field, cause) = if bytes[16..32] == [0; 16] {
        (
            STORE_IDENTITY,
            PhysicalFormatField::StoreIdentity,
            PhysicalDamageCause::StoreIdentityMismatch,
        )
    } else if read_u64(bytes, RUNTIME_IDENTITY) == 0 {
        (
            RUNTIME_IDENTITY,
            PhysicalFormatField::RuntimeIdentity,
            PhysicalDamageCause::ArtifactIdentityMismatch,
        )
    } else if read_u64(bytes, GENERATION_IDENTITY) == 0 {
        (
            GENERATION_IDENTITY,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalDamageCause::PhysicalGenerationMismatch,
        )
    } else {
        (
            OPERATION_IDENTITY,
            PhysicalFormatField::OperationIdentity,
            PhysicalDamageCause::ArtifactIdentityMismatch,
        )
    };
    field_damage(
        scope,
        cause,
        range,
        field,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn invalid_target(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let (range, field) = if bytes[105] > 1 {
        (
            PAYLOAD_DIGEST_PRESENCE,
            PhysicalFormatField::PayloadDigestPresence,
        )
    } else {
        (TARGET_SHAPE, PhysicalFormatField::TargetShape)
    };
    field_damage(
        scope,
        PhysicalDamageCause::MalformedStructure,
        range,
        field,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn read_u64(bytes: &[u8], range: FieldRange) -> u64 {
    let start = range.offset as usize;
    u64::from_le_bytes(
        bytes[start..start + 8]
            .try_into()
            .expect("physical-work identity field has fixed width"),
    )
}
