use worth_store_physical_format::wal_frame::{
    WalFrameV1BoundedDecodeDenial, WalFrameV1ChecksumMismatch, WalFrameV1Denial,
    WAL_FRAME_V1_FOOTER_BYTES,
};
use worth_store_physical_format::WalSegmentIdentity;

use crate::localization::{
    PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause, PhysicalDamageLocalization,
    PhysicalFormatField,
};
use crate::validation::{
    PhysicalArtifactScope, PhysicalIntegrityRejection, PhysicalIntegrityVersionAxis,
    UnsupportedPhysicalIntegrityVersion,
};

const MAGIC: WalFrameFieldRange = WalFrameFieldRange::new(0, 8);
const HEADER_LENGTH: WalFrameFieldRange = WalFrameFieldRange::new(10, 2);
const SEGMENT: WalFrameFieldRange = WalFrameFieldRange::new(12, 8);
const GENERATION: WalFrameFieldRange = WalFrameFieldRange::new(20, 8);
const LSN_RANGE: WalFrameFieldRange = WalFrameFieldRange::new(28, 16);
const PAYLOAD_LENGTH: WalFrameFieldRange = WalFrameFieldRange::new(44, 8);
const PAYLOAD_CHECKSUM: WalFrameFieldRange = WalFrameFieldRange::new(84, 32);

#[derive(Debug, Clone, Copy)]
struct WalFrameFieldRange {
    offset: u64,
    length: u64,
}

impl WalFrameFieldRange {
    const fn new(offset: u64, length: u64) -> Self {
        Self { offset, length }
    }
}

pub(super) fn wrong_scope(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    damaged(
        scope,
        PhysicalDamageCause::FamilyMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

pub(super) fn input_length(
    scope: PhysicalArtifactScope,
    observed: u64,
) -> Option<PhysicalIntegrityRejection> {
    let expected = scope.byte_range().length();
    if observed < expected {
        let missing =
            PhysicalByteRange::new(scope.byte_range().offset() + observed, expected - observed)
                .expect("missing WAL tail remains inside its exact scope");
        return Some(damaged(
            scope,
            PhysicalDamageCause::Truncated,
            missing,
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ));
    }
    (observed > expected).then(|| {
        damaged(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        )
    })
}

pub(super) fn from_bounded_denial(
    scope: PhysicalArtifactScope,
    denial: WalFrameV1BoundedDecodeDenial,
) -> PhysicalIntegrityRejection {
    match denial {
        WalFrameV1BoundedDecodeDenial::TruncatedHeader => damaged(
            scope,
            PhysicalDamageCause::Truncated,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        WalFrameV1BoundedDecodeDenial::Header(denial) => from_header_denial(scope, denial),
        WalFrameV1BoundedDecodeDenial::FrameLengthOverflow
        | WalFrameV1BoundedDecodeDenial::FrameLengthMismatch { .. } => field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            PAYLOAD_LENGTH,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        WalFrameV1BoundedDecodeDenial::ChecksumMismatch(mismatch) => {
            checksum_mismatch(scope, mismatch)
        }
    }
}

pub(super) fn scope_identity_mismatch(
    scope: PhysicalArtifactScope,
    observed: WalSegmentIdentity,
) -> PhysicalIntegrityRejection {
    let expected = scope
        .wal_segment_identity()
        .expect("WAL scope carries its segment identity");
    if observed.segment() != expected.segment() {
        return field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            SEGMENT,
            PhysicalFormatField::SegmentIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
    field_damage(
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        GENERATION,
        PhysicalFormatField::PhysicalGeneration,
        PhysicalBlastRadius::CompleteArtifact,
    )
}

fn from_header_denial(
    scope: PhysicalArtifactScope,
    denial: WalFrameV1Denial,
) -> PhysicalIntegrityRejection {
    match denial {
        WalFrameV1Denial::WrongMagic => field_damage(
            scope,
            PhysicalDamageCause::WrongMagic,
            MAGIC,
            PhysicalFormatField::Magic,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        WalFrameV1Denial::UnsupportedVersion(observed) => {
            PhysicalIntegrityRejection::Unsupported(UnsupportedPhysicalIntegrityVersion::new(
                scope,
                PhysicalIntegrityVersionAxis::WalFrame,
                u32::from(observed),
            ))
        }
        WalFrameV1Denial::HeaderLengthMismatch(_) => field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            HEADER_LENGTH,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        WalFrameV1Denial::InvalidSegmentIdentity => field_damage(
            scope,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            SEGMENT,
            PhysicalFormatField::SegmentIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        WalFrameV1Denial::InvalidGeneration => field_damage(
            scope,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            GENERATION,
            PhysicalFormatField::PhysicalGeneration,
            PhysicalBlastRadius::CompleteArtifact,
        ),
        WalFrameV1Denial::InvalidLsnRange => field_damage(
            scope,
            PhysicalDamageCause::SequenceMismatch,
            LSN_RANGE,
            PhysicalFormatField::WalLsnRange,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        WalFrameV1Denial::EmptyPayload | WalFrameV1Denial::PayloadLengthMismatch => field_damage(
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            PAYLOAD_LENGTH,
            PhysicalFormatField::EncodedLength,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        WalFrameV1Denial::ChecksumMismatch => damaged(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
    }
}

fn checksum_mismatch(
    scope: PhysicalArtifactScope,
    mismatch: WalFrameV1ChecksumMismatch,
) -> PhysicalIntegrityRejection {
    match (mismatch.payload_checksum(), mismatch.frame_checksum()) {
        (true, false) => field_damage(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            PAYLOAD_CHECKSUM,
            PhysicalFormatField::Checksum,
            PhysicalBlastRadius::DamagedRange,
        ),
        (false, true) => field_damage(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            footer_field(scope),
            PhysicalFormatField::Checksum,
            PhysicalBlastRadius::DamagedRange,
        ),
        (true, true) | (false, false) => damaged(
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
    }
}

fn footer_field(scope: PhysicalArtifactScope) -> WalFrameFieldRange {
    WalFrameFieldRange::new(
        scope.byte_range().length() - WAL_FRAME_V1_FOOTER_BYTES as u64,
        WAL_FRAME_V1_FOOTER_BYTES as u64,
    )
}

fn field_damage(
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: WalFrameFieldRange,
    field: PhysicalFormatField,
    blast_radius: PhysicalBlastRadius,
) -> PhysicalIntegrityRejection {
    let damaged_range =
        PhysicalByteRange::new(scope.byte_range().offset() + range.offset, range.length)
            .expect("WAL field remains inside exact frame scope");
    damaged(scope, cause, damaged_range, Some(field), blast_radius)
}

const fn damaged(
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
