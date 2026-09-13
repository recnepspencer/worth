use worth_foundational::PhysicalByteRange;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityFormatDeclaration;

use super::super::{
    crc32c::crc32c, OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization,
    OfflinePhysicalFormatField, OfflineUnsupportedPhysicalVersion, OfflineUnsupportedVersionAxis,
};

const HEADER_BYTES: usize = 48;
const CHECKSUM_OFFSET: usize = 44;
const MAGIC: &[u8; 8] = b"WRC5FRM\0";

pub(crate) struct DurableFrameFacts<'a> {
    pub(crate) identity: u64,
    pub(crate) format: [u8; 10],
    pub(crate) payload: &'a [u8],
}

pub(crate) fn read_durable_frame<'a>(
    bytes: &'a [u8],
    expected_bytes: usize,
    expected_kind: u8,
    declaration: PhysicalIntegrityFormatDeclaration,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<DurableFrameFacts<'a>, OfflineIntegrityOutcome> {
    if bytes.len() < HEADER_BYTES {
        return Err(truncation(bytes.len(), expected_bytes));
    }
    if &bytes[..8] != MAGIC {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::Framing,
            0,
            8,
            OfflinePhysicalFormatField::Magic,
        ));
    }
    if bytes[8] != expected_kind {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::ScopeMismatch,
            8,
            1,
            OfflinePhysicalFormatField::FamilyKind,
        ));
    }
    if bytes[9] != declaration.version().envelope_schema().unwrap_or_default() as u8 {
        return Err(unsupported(
            OfflineUnsupportedVersionAxis::EnvelopeSchema,
            u64::from(bytes[9]),
            "2",
            9,
            1,
        ));
    }
    validate_format(&bytes[10..20], declaration)?;
    if read_u16(bytes, 20) as usize != HEADER_BYTES {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::Framing,
            20,
            2,
            OfflinePhysicalFormatField::HeaderLength,
        ));
    }
    if bytes[22..24] != [0, 0] {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::MalformedPayload,
            22,
            2,
            OfflinePhysicalFormatField::Reserved,
        ));
    }
    let declared_total = HEADER_BYTES + read_u32(bytes, 24) as usize;
    if bytes.len() != declared_total {
        if bytes.len() < expected_bytes && declared_total == expected_bytes {
            return Err(truncation(bytes.len(), expected_bytes));
        }
        return Err(damaged_field(
            OfflinePhysicalDamageCause::Framing,
            24,
            4,
            OfflinePhysicalFormatField::PayloadLength,
        ));
    }
    if bytes.len() != expected_bytes {
        return Err(damaged_field(
            OfflinePhysicalDamageCause::Framing,
            24,
            4,
            OfflinePhysicalFormatField::PayloadLength,
        ));
    }
    counters.checksum_calculations += 1;
    let stored = read_u32(bytes, CHECKSUM_OFFSET);
    let calculated = crc32c(&[&bytes[..CHECKSUM_OFFSET], &bytes[HEADER_BYTES..]]);
    if stored != calculated {
        return Err(OfflineIntegrityOutcome::Damaged(
            OfflinePhysicalDamageLocalization::new(
                OfflinePhysicalDamageCause::ChecksumMismatch,
                Some((0, bytes.len() as u64)),
                // A mismatch proves only that the stored value differs from the CRC32C over the
                // declared ranges; it cannot identify which covered or stored bytes are corrupt.
                None,
                OfflinePhysicalBlastRadius::Frame,
            ),
        ));
    }
    counters.checksum_validated_durable_frames += 1;
    Ok(DurableFrameFacts {
        identity: read_u64(bytes, 28),
        format: bytes[10..20].try_into().expect("fixed format declaration"),
        payload: &bytes[HEADER_BYTES..],
    })
}

fn validate_format(
    bytes: &[u8],
    declaration: PhysicalIntegrityFormatDeclaration,
) -> Result<(), OfflineIntegrityOutcome> {
    let expected_version = declaration.version().format_version();
    let version = read_u16(bytes, 0);
    if version != expected_version {
        return Err(unsupported(
            OfflineUnsupportedVersionAxis::PhysicalRecordFormat,
            u64::from(version),
            "1",
            10,
            2,
        ));
    }
    let page_bytes = read_u32(bytes, 2);
    if !matches!(page_bytes, 16_384 | 32_768 | 65_536) {
        return Err(unsupported(
            OfflineUnsupportedVersionAxis::PageSize,
            u64::from(page_bytes),
            "16384|32768|65536",
            12,
            4,
        ));
    }
    for (offset, observed, expected, axis, supported) in [
        (
            6,
            bytes[6],
            1,
            OfflineUnsupportedVersionAxis::ByteOrder,
            "1",
        ),
        (
            7,
            bytes[7],
            1,
            OfflineUnsupportedVersionAxis::RootProtocol,
            "1",
        ),
        (
            8,
            bytes[8],
            1,
            OfflineUnsupportedVersionAxis::IntegrityAlgorithm,
            "1",
        ),
        (
            9,
            bytes[9],
            24,
            OfflineUnsupportedVersionAxis::RecordIdentityWidth,
            "24",
        ),
    ] {
        if observed != expected {
            return Err(unsupported(
                axis,
                u64::from(observed),
                supported,
                10 + offset,
                1,
            ));
        }
    }
    Ok(())
}

fn truncation(observed: usize, expected: usize) -> OfflineIntegrityOutcome {
    let missing = expected.saturating_sub(observed).max(1);
    OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
        OfflinePhysicalDamageCause::Truncation,
        Some((observed as u64, missing as u64)),
        None,
        OfflinePhysicalBlastRadius::Artifact,
    ))
}

pub(crate) fn damaged_field(
    cause: OfflinePhysicalDamageCause,
    offset: u64,
    length: u64,
    field: OfflinePhysicalFormatField,
) -> OfflineIntegrityOutcome {
    OfflineIntegrityOutcome::Damaged(OfflinePhysicalDamageLocalization::new(
        cause,
        Some((offset, length)),
        Some(field),
        OfflinePhysicalBlastRadius::Field,
    ))
}

fn unsupported(
    axis: OfflineUnsupportedVersionAxis,
    observed: u64,
    supported: &'static str,
    offset: u64,
    length: u64,
) -> OfflineIntegrityOutcome {
    OfflineIntegrityOutcome::Unsupported(OfflineUnsupportedPhysicalVersion::new(
        axis,
        observed,
        supported,
        PhysicalByteRange::new(offset, length).expect("unsupported field range"),
    ))
}

pub(crate) fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("bounded field"))
}
pub(crate) fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("bounded field"))
}
pub(crate) fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("bounded field"))
}
