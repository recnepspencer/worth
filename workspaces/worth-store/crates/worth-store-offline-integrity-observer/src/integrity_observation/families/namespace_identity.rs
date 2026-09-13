use worth_foundational::PhysicalByteRange;
use worth_store_physical_format::integrity_declarations::{
    families::NAMESPACE_IDENTITY_INTEGRITY_DECLARATION, PhysicalIntegrityAlgorithm,
    PhysicalIntegrityCoverageBoundary,
};

use super::super::{
    sha256::sha256, OfflineIntegrityObservationCounters, OfflineIntegrityOutcome,
    OfflinePhysicalBlastRadius, OfflinePhysicalDamageCause, OfflinePhysicalDamageLocalization,
    OfflinePhysicalFormatField, OfflineUnsupportedPhysicalVersion, OfflineUnsupportedVersionAxis,
};

pub(crate) const NAMESPACE_IDENTITY_BYTES: usize = 72;
const MAGIC: &[u8; 8] = b"WSTNSID\0";

pub(crate) struct OfflineNamespaceIdentityFacts {
    pub(crate) store_identity: [u8; 16],
}

pub(crate) fn read_namespace_identity(
    bytes: &[u8],
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<OfflineNamespaceIdentityFacts, OfflineIntegrityOutcome> {
    if bytes.len() != NAMESPACE_IDENTITY_BYTES {
        return Err(length_damage(bytes.len()));
    }
    if &bytes[..8] != MAGIC {
        return Err(field_damage(
            OfflinePhysicalDamageCause::Framing,
            0,
            8,
            OfflinePhysicalFormatField::Magic,
        ));
    }
    let (covered, field) = declared_checksum_ranges()?;
    counters.checksum_calculations += 1;
    if bytes[field.clone()] != sha256(&bytes[covered.clone()]) {
        return Err(OfflineIntegrityOutcome::Damaged(
            OfflinePhysicalDamageLocalization::new(
                OfflinePhysicalDamageCause::ChecksumMismatch,
                Some((covered.start as u64, (field.end - covered.start) as u64)),
                Some(OfflinePhysicalFormatField::Checksum),
                OfflinePhysicalBlastRadius::Artifact,
            ),
        ));
    }
    let declared_version = NAMESPACE_IDENTITY_INTEGRITY_DECLARATION
        .version()
        .format_version();
    let encoding = read_u16(bytes, 8);
    if encoding != declared_version {
        return Err(unsupported(
            OfflineUnsupportedVersionAxis::NamespaceEncoding,
            encoding,
            8,
        ));
    }
    let namespace = read_u16(bytes, 10);
    if namespace != 1 {
        return Err(unsupported(
            OfflineUnsupportedVersionAxis::NamespaceSchema,
            namespace,
            10,
        ));
    }
    if read_u32(bytes, 12) != 72 {
        return Err(field_damage(
            OfflinePhysicalDamageCause::Framing,
            12,
            4,
            OfflinePhysicalFormatField::RecordLength,
        ));
    }
    if read_u16(bytes, 16) != 1 || bytes[18..20] != [0, 0] {
        return Err(field_damage(
            OfflinePhysicalDamageCause::MalformedPayload,
            16,
            4,
            OfflinePhysicalFormatField::FieldCount,
        ));
    }
    if read_u16(bytes, 20) != 1 || read_u16(bytes, 22) != 16 {
        return Err(field_damage(
            OfflinePhysicalDamageCause::MalformedPayload,
            20,
            4,
            OfflinePhysicalFormatField::IdentityField,
        ));
    }
    let mut store_identity = [0; 16];
    store_identity.copy_from_slice(&bytes[24..40]);
    if store_identity == [0; 16] {
        return Err(field_damage(
            OfflinePhysicalDamageCause::ScopeMismatch,
            24,
            16,
            OfflinePhysicalFormatField::StoreIdentity,
        ));
    }
    counters.namespace_identity_payload_decoder_entries += 1;
    Ok(OfflineNamespaceIdentityFacts { store_identity })
}

fn declared_checksum_ranges(
) -> Result<(std::ops::Range<usize>, std::ops::Range<usize>), OfflineIntegrityOutcome> {
    let checksums = NAMESPACE_IDENTITY_INTEGRITY_DECLARATION.checksums();
    let Some(checksum) = checksums.first().copied().filter(|_| checksums.len() == 1) else {
        return Err(unsupported_declaration());
    };
    if checksum.algorithm() != PhysicalIntegrityAlgorithm::Sha256
        || checksum.covered_ranges().len() != 1
    {
        return Err(unsupported_declaration());
    }
    let covered = checksum.covered_ranges()[0];
    let field = checksum.field();
    let (
        PhysicalIntegrityCoverageBoundary::Fixed(covered_start),
        PhysicalIntegrityCoverageBoundary::Fixed(covered_end),
        PhysicalIntegrityCoverageBoundary::Fixed(field_start),
        PhysicalIntegrityCoverageBoundary::Fixed(field_end),
    ) = (covered.start(), covered.end(), field.start(), field.end())
    else {
        return Err(unsupported_declaration());
    };
    let bounds = [covered_start, covered_end, field_start, field_end];
    if bounds
        .iter()
        .any(|value| *value > NAMESPACE_IDENTITY_BYTES as u64)
        || covered_start >= covered_end
        || field_start >= field_end
        || field_end - field_start != 32
    {
        return Err(unsupported_declaration());
    }
    Ok((
        covered_start as usize..covered_end as usize,
        field_start as usize..field_end as usize,
    ))
}

fn unsupported_declaration() -> OfflineIntegrityOutcome {
    OfflineIntegrityOutcome::Unsupported(OfflineUnsupportedPhysicalVersion::new(
        OfflineUnsupportedVersionAxis::IntegrityAlgorithm,
        0,
        "one sha256 checksum with fixed ranges",
        PhysicalByteRange::new(40, 32).expect("namespace checksum field"),
    ))
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("bounded field"))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("bounded field"))
}

fn length_damage(observed: usize) -> OfflineIntegrityOutcome {
    let (offset, length) = if observed < 72 {
        (observed as u64, (72 - observed) as u64)
    } else {
        (72, (observed - 72) as u64)
    };
    field_damage(
        OfflinePhysicalDamageCause::Truncation,
        offset,
        length.max(1),
        OfflinePhysicalFormatField::RecordLength,
    )
}

fn field_damage(
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
    observed: u16,
    offset: u64,
) -> OfflineIntegrityOutcome {
    OfflineIntegrityOutcome::Unsupported(OfflineUnsupportedPhysicalVersion::new(
        axis,
        u64::from(observed),
        "1",
        PhysicalByteRange::new(offset, 2).expect("version field"),
    ))
}
