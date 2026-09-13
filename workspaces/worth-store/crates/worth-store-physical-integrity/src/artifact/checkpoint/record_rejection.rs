use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::CheckpointStreamDecodeDenial;

use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization,
    PhysicalFormatField,
};
use crate::validation::{
    PhysicalArtifactScope, PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
    UnsupportedPhysicalIntegrityVersion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CheckpointRecordFieldRange {
    offset: u64,
    length: u64,
}

impl CheckpointRecordFieldRange {
    pub(super) const fn new(offset: u64, length: u64) -> Self {
        Self { offset, length }
    }
}

pub(super) fn checkpoint_record_denial(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
    denial: CheckpointStreamDecodeDenial,
) -> PhysicalIntegrityRejection {
    use CheckpointStreamDecodeDenial as Denial;
    match denial {
        Denial::Truncated => damaged(
            scope,
            PhysicalDamageCause::Truncated,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        Denial::WrongMagic => field_damage(
            scope,
            PhysicalDamageCause::WrongMagic,
            CheckpointRecordFieldRange::new(0, 8),
            PhysicalFormatField::Magic,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        Denial::UnsupportedSchema(observed) => {
            PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
                scope,
                PhysicalIntegrityVersionAxis::CheckpointRecordSchema,
                u32::from(observed),
            ))
        }
        Denial::WrongRecordKind { .. } => field_damage(
            scope,
            PhysicalDamageCause::RecordKindMismatch,
            CheckpointRecordFieldRange::new(9, 1),
            PhysicalFormatField::CheckpointRecordKind,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        Denial::ReservedFieldNonZero => reserved_field_damage(scope, bytes),
        Denial::LengthMismatch | Denial::BindingRecordTooLarge => field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            CheckpointRecordFieldRange::new(12, 4),
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        Denial::IntegrityMismatch => damaged(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        Denial::InvalidIdentity | Denial::SourceIdentityMismatch => identity_damage(scope, bytes),
        Denial::InvalidWalRange => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            CheckpointRecordFieldRange::new(40, 16),
            PhysicalFormatField::WalLsnRange,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        Denial::InvalidCapturePosture(_) => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            CheckpointRecordFieldRange::new(80, 1),
            PhysicalFormatField::Payload,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        denial @ (Denial::InvalidSecurityBindingPresence(_)
        | Denial::AbsentSecurityBindingResidue { .. }
        | Denial::InvalidSecurityPolicyIdentity
        | Denial::InvalidSecurityRetention
        | Denial::SecurityBindingDigestMismatch) => security_binding_denial(scope, denial),
        Denial::InvalidArtifactKind(_) => field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            CheckpointRecordFieldRange::new(16, 1),
            PhysicalFormatField::ArtifactIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        Denial::InvalidCoordinate => coordinate_damage(scope, bytes),
        Denial::InvalidBindingCompactionHeader => compaction_header_damage(scope, bytes),
        Denial::EmptyBindingRecord => field_damage(
            scope,
            PhysicalDamageCause::MalformedStructure,
            CheckpointRecordFieldRange::new(12, 4),
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        Denial::RecordCountMismatch => damaged(
            scope,
            PhysicalDamageCause::AggregateMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::CheckpointAggregate),
            PhysicalBlastRadius::CompleteArtifact,
        ),
        Denial::RecordByteCountMismatch => field_damage(
            scope,
            PhysicalDamageCause::AggregateMismatch,
            CheckpointRecordFieldRange::new(112, 8),
            PhysicalFormatField::CheckpointAggregate,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        Denial::BindingCompactionMismatch => field_damage(
            scope,
            PhysicalDamageCause::AggregateMismatch,
            CheckpointRecordFieldRange::new(80, 24),
            PhysicalFormatField::CheckpointAggregate,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        Denial::AggregateDigestMismatch => damaged(
            scope,
            PhysicalDamageCause::AggregateMismatch,
            scope.byte_range(),
            Some(PhysicalFormatField::CheckpointAggregate),
            PhysicalBlastRadius::CompleteArtifact,
        ),
    }
}

fn security_binding_denial(
    scope: PhysicalArtifactScope,
    denial: CheckpointStreamDecodeDenial,
) -> PhysicalIntegrityRejection {
    use CheckpointStreamDecodeDenial as Denial;
    let (cause, range, field) = match denial {
        Denial::InvalidSecurityBindingPresence(_) => (
            PhysicalDamageCause::MalformedStructure,
            CheckpointRecordFieldRange::new(81, 1),
            PhysicalFormatField::Payload,
        ),
        Denial::AbsentSecurityBindingResidue { payload_offset } => (
            PhysicalDamageCause::MalformedStructure,
            CheckpointRecordFieldRange::new(16 + u64::from(payload_offset), 1),
            PhysicalFormatField::Reserved,
        ),
        Denial::InvalidSecurityPolicyIdentity => (
            PhysicalDamageCause::MalformedStructure,
            CheckpointRecordFieldRange::new(88, 32),
            PhysicalFormatField::Payload,
        ),
        Denial::InvalidSecurityRetention => (
            PhysicalDamageCause::MalformedStructure,
            CheckpointRecordFieldRange::new(120, 8),
            PhysicalFormatField::Payload,
        ),
        Denial::SecurityBindingDigestMismatch => (
            PhysicalDamageCause::ChecksumMismatch,
            CheckpointRecordFieldRange::new(16, 144),
            PhysicalFormatField::Checksum,
        ),
        _ => unreachable!("caller admits only security-binding denials"),
    };
    field_damage(
        scope,
        cause,
        range,
        field,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

pub(super) fn field_damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    field_range: CheckpointRecordFieldRange,
    field: PhysicalFormatField,
    blast_radius: PhysicalBlastRadius,
) -> PhysicalIntegrityRejection {
    let range = PhysicalByteRange::new(
        scope.byte_range().offset() + field_range.offset,
        field_range.length,
    )
    .expect("checkpoint field remains within its certified record scope");
    damaged(scope, cause, range, Some(field), blast_radius)
}

pub(super) const fn damaged(
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

fn reserved_field_damage(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let range = if bytes.get(10..12).is_some_and(|reserved| reserved != [0; 2]) {
        CheckpointRecordFieldRange::new(10, 2)
    } else if scope.artifact_family() == PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis {
        if bytes.get(17..24).is_some_and(|reserved| reserved != [0; 7]) {
            CheckpointRecordFieldRange::new(17, 7)
        } else {
            CheckpointRecordFieldRange::new(52, 4)
        }
    } else {
        CheckpointRecordFieldRange::new(82, 6)
    };
    field_damage(
        scope,
        PhysicalDamageCause::MalformedStructure,
        range,
        PhysicalFormatField::Reserved,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn identity_damage(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let (cause, range, field) = if bytes.get(16..32).is_some_and(all_zero) {
        (
            PhysicalDamageCause::StoreIdentityMismatch,
            CheckpointRecordFieldRange::new(16, 16),
            PhysicalFormatField::StoreIdentity,
        )
    } else {
        (
            PhysicalDamageCause::SequenceMismatch,
            CheckpointRecordFieldRange::new(32, 8),
            PhysicalFormatField::CheckpointIdentity,
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

fn coordinate_damage(scope: PhysicalArtifactScope, bytes: &[u8]) -> PhysicalIntegrityRejection {
    let length_is_zero = bytes.get(48..52).is_some_and(|length| length == [0; 4]);
    let range = if length_is_zero {
        CheckpointRecordFieldRange::new(48, 4)
    } else {
        CheckpointRecordFieldRange::new(40, 12)
    };
    field_damage(
        scope,
        PhysicalDamageCause::MalformedStructure,
        range,
        PhysicalFormatField::Payload,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn compaction_header_damage(
    scope: PhysicalArtifactScope,
    bytes: &[u8],
) -> PhysicalIntegrityRejection {
    let zero_generation = bytes
        .get(16..24)
        .is_some_and(|generation| generation == [0; 8]);
    let (range, field) = if zero_generation {
        (
            CheckpointRecordFieldRange::new(16, 8),
            PhysicalFormatField::PhysicalGeneration,
        )
    } else {
        (
            CheckpointRecordFieldRange::new(24, 8),
            PhysicalFormatField::WalLsnRange,
        )
    };
    field_damage(
        scope,
        PhysicalDamageCause::MalformedStructure,
        range,
        field,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}
