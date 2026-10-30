use super::protected_owner::{
    allocation_from_tag, allocation_tag, domain_from_tag, domain_tag, BackupProtectedPhysicalOwner,
};
use super::BackupReachabilityLeaseRecoveryDenial;

const MAGIC: &[u8; 4] = b"WBL1";
pub(super) const OWNER_BYTES: usize = 51;

pub(super) fn canonicalize_protected_owners(
    protected: &mut [BackupProtectedPhysicalOwner],
) -> Result<(), BackupReachabilityLeaseRecoveryDenial> {
    if protected.is_empty() {
        return Err(BackupReachabilityLeaseRecoveryDenial::EmptyProtection);
    }
    if protected.len() > u32::MAX as usize {
        return Err(BackupReachabilityLeaseRecoveryDenial::ProtectionCountOverflow);
    }
    if protected.iter().any(|owner| !owner.is_valid()) {
        return Err(BackupReachabilityLeaseRecoveryDenial::InvalidOwnerCoordinate);
    }
    protected.sort_unstable();
    if protected.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(BackupReachabilityLeaseRecoveryDenial::DuplicateProtection);
    }
    Ok(())
}

pub(super) fn encode(
    cut_identity: [u8; 32],
    protected: &[BackupProtectedPhysicalOwner],
) -> Result<Vec<u8>, BackupReachabilityLeaseRecoveryDenial> {
    let count = u32::try_from(protected.len())
        .map_err(|_| BackupReachabilityLeaseRecoveryDenial::ProtectionCountOverflow)?;
    let encoded_bytes = 40usize
        .checked_add(
            protected
                .len()
                .checked_mul(OWNER_BYTES)
                .ok_or(BackupReachabilityLeaseRecoveryDenial::RecordLengthOverflow)?,
        )
        .ok_or(BackupReachabilityLeaseRecoveryDenial::RecordLengthOverflow)?;
    let mut encoded = Vec::new();
    encoded
        .try_reserve_exact(encoded_bytes)
        .map_err(|_| BackupReachabilityLeaseRecoveryDenial::AllocationFailed)?;
    encoded.extend_from_slice(MAGIC);
    encoded.extend_from_slice(&cut_identity);
    encoded.extend_from_slice(&count.to_le_bytes());
    for owner in protected {
        encode_owner(&mut encoded, *owner);
    }
    Ok(encoded)
}

pub(super) fn decode(
    encoded: &[u8],
) -> Result<([u8; 32], Vec<BackupProtectedPhysicalOwner>), BackupReachabilityLeaseRecoveryDenial> {
    if encoded.len() < 40 || &encoded[..4] != MAGIC {
        return Err(BackupReachabilityLeaseRecoveryDenial::InvalidEncoding);
    }
    let cut_identity = encoded[4..36]
        .try_into()
        .map_err(|_| BackupReachabilityLeaseRecoveryDenial::InvalidEncoding)?;
    let count = u32::from_le_bytes(
        encoded[36..40]
            .try_into()
            .map_err(|_| BackupReachabilityLeaseRecoveryDenial::InvalidEncoding)?,
    ) as usize;
    if count == 0 {
        return Err(BackupReachabilityLeaseRecoveryDenial::EmptyProtection);
    }
    enforce_exact_record_length(encoded.len(), count)?;
    decode_canonical_owners(&encoded[40..], count).map(|protected| (cut_identity, protected))
}

fn enforce_exact_record_length(
    encoded_bytes: usize,
    count: usize,
) -> Result<(), BackupReachabilityLeaseRecoveryDenial> {
    let expected = 40usize
        .checked_add(
            count
                .checked_mul(OWNER_BYTES)
                .ok_or(BackupReachabilityLeaseRecoveryDenial::InvalidEncoding)?,
        )
        .ok_or(BackupReachabilityLeaseRecoveryDenial::InvalidEncoding)?;
    if encoded_bytes == expected {
        Ok(())
    } else {
        Err(BackupReachabilityLeaseRecoveryDenial::InvalidEncoding)
    }
}

fn decode_canonical_owners(
    encoded_owners: &[u8],
    count: usize,
) -> Result<Vec<BackupProtectedPhysicalOwner>, BackupReachabilityLeaseRecoveryDenial> {
    let mut protected = Vec::new();
    protected
        .try_reserve_exact(count)
        .map_err(|_| BackupReachabilityLeaseRecoveryDenial::AllocationFailed)?;
    let mut previous = None;
    for row in encoded_owners.as_chunks::<OWNER_BYTES>().0.iter() {
        let owner = decode_owner(row)?;
        enforce_canonical_successor(previous, owner)?;
        previous = Some(owner);
        protected.push(owner);
    }
    Ok(protected)
}

fn enforce_canonical_successor(
    previous: Option<BackupProtectedPhysicalOwner>,
    owner: BackupProtectedPhysicalOwner,
) -> Result<(), BackupReachabilityLeaseRecoveryDenial> {
    match previous {
        Some(previous) if owner == previous => {
            Err(BackupReachabilityLeaseRecoveryDenial::DuplicateProtection)
        }
        Some(previous) if owner < previous => {
            Err(BackupReachabilityLeaseRecoveryDenial::NonCanonicalProtectionOrder)
        }
        _ => Ok(()),
    }
}

fn encode_owner(encoded: &mut Vec<u8>, owner: BackupProtectedPhysicalOwner) {
    encoded.push(domain_tag(owner.domain));
    encoded.extend_from_slice(&owner.generation.to_le_bytes());
    for value in [
        owner.segment,
        owner.page,
        owner.extent,
        owner.slot,
        owner.root,
    ] {
        encoded.extend_from_slice(&value.unwrap_or(0).to_le_bytes());
    }
    encoded.push(allocation_tag(owner.allocation));
    encoded.push(0);
}

fn decode_owner(
    row: &[u8],
) -> Result<BackupProtectedPhysicalOwner, BackupReachabilityLeaseRecoveryDenial> {
    let owner = BackupProtectedPhysicalOwner {
        domain: domain_from_tag(row[0])
            .ok_or(BackupReachabilityLeaseRecoveryDenial::InvalidOwnerCoordinate)?,
        generation: read_u64(row, 1)?,
        segment: optional(read_u64(row, 9)?),
        page: optional(read_u64(row, 17)?),
        extent: optional(read_u64(row, 25)?),
        slot: optional(read_u64(row, 33)?),
        root: optional(read_u64(row, 41)?),
        allocation: allocation_from_tag(row[49])
            .ok_or(BackupReachabilityLeaseRecoveryDenial::InvalidOwnerCoordinate)?,
    };
    if row[50] != 0 || !owner.is_valid() {
        return Err(BackupReachabilityLeaseRecoveryDenial::InvalidOwnerCoordinate);
    }
    Ok(owner)
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, BackupReachabilityLeaseRecoveryDenial> {
    bytes[offset..offset + 8]
        .try_into()
        .map(u64::from_le_bytes)
        .map_err(|_| BackupReachabilityLeaseRecoveryDenial::InvalidEncoding)
}

const fn optional(value: u64) -> Option<u64> {
    if value == 0 {
        None
    } else {
        Some(value)
    }
}
