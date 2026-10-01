//! A copy is a bounded regeneration recipe, never an empty frame-target record.
use super::*;
use worth_store_physical_format::{
    PersistedExtentCopyRecipe, PersistedPhysicalRecoveryPayload, PhysicalRecoveryProjectionDenial,
};

const DOMAIN: &[u8] = b"store.physical.extent-copy-publication.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalExtentCopyAdmission {
    operation: [u8; 32],
    group: PhysicalRedoGroupBinding,
    fate: RecoveryOperationFate,
    publication_lsn: u64,
    recipe: PersistedExtentCopyRecipe,
    projection: PersistedPhysicalRecoveryProjection,
}

impl PhysicalExtentCopyAdmission {
    pub const fn operation(&self) -> [u8; 32] {
        self.operation
    }
    pub const fn group(&self) -> PhysicalRedoGroupBinding {
        self.group
    }
    pub const fn fate(&self) -> RecoveryOperationFate {
        self.fate
    }
    pub const fn publication_lsn(&self) -> u64 {
        self.publication_lsn
    }
    pub const fn recipe(&self) -> PersistedExtentCopyRecipe {
        self.recipe
    }
    pub const fn projection(&self) -> &PersistedPhysicalRecoveryProjection {
        &self.projection
    }
}

impl ImmutablePhysicalRedoPlan {
    pub fn source_copies(&self) -> &[PhysicalExtentCopyAdmission] {
        &self.source_copies
    }
}

pub(super) fn is_copy(bytes: &[u8]) -> bool {
    field(&mut &*bytes).is_ok_and(|domain| domain == DOMAIN)
}

pub(super) fn admit(
    member: &PhysicalRedoMemberInput,
    format: PhysicalRecordFormatDeclaration,
    limits: PhysicalRecoveryProjectionDecodeLimits,
) -> Result<Option<PhysicalExtentCopyAdmission>, PhysicalRedoPlanningDenial> {
    let Some(projection) = admit_current_source_copy_publication(
        member.operation(),
        member.lsn_range(),
        member.canonical_redo(),
        format,
        limits,
    )?
    else {
        return Ok(None);
    };
    if member.fate() == RecoveryOperationFate::ProvenNoEffect {
        return Err(PhysicalRedoPlanningDenial::ProvenNoEffectHasWalAttempt);
    }
    let PersistedPhysicalRecoveryPayload::SourceCopy(recipe) = projection.payload() else {
        unreachable!("borrowed copy admission proved the source-copy payload")
    };
    Ok(Some(PhysicalExtentCopyAdmission {
        operation: member.operation(),
        group: member.group(),
        fate: member.fate(),
        publication_lsn: member.lsn_range().start().get(),
        recipe: *recipe,
        projection,
    }))
}

/// Pure C.9 member admission. The caller must separately bind a sampled WAL
/// member to actual media and join the earlier durable copy-intent frame.
pub fn admit_current_source_copy_publication(
    operation: [u8; 32],
    range: WalLsnRange,
    canonical_redo: &[u8],
    format: PhysicalRecordFormatDeclaration,
    limits: PhysicalRecoveryProjectionDecodeLimits,
) -> Result<Option<PersistedPhysicalRecoveryProjection>, PhysicalRedoPlanningDenial> {
    if !is_copy(canonical_redo) {
        return Ok(None);
    }
    let mut bytes = canonical_redo;
    field(&mut bytes)?;
    let publication_lsn = number(&mut bytes)?;
    if publication_lsn != range.start().get()
        || publication_lsn.checked_add(1) != Some(range.end_exclusive().get())
    {
        return Err(PhysicalRedoPlanningDenial::LsnRangeMismatch);
    }
    let encoded = field(&mut bytes)?;
    if !bytes.is_empty() {
        return Err(PhysicalRedoPlanningDenial::MalformedMember);
    }
    let copy_limits = PhysicalRecoveryProjectionDecodeLimits {
        frames: 0,
        record_identities: limits.record_identities.min(1),
        placements: limits.placements.min(1),
        segment_updates: 0,
        manifests: 0,
        total_entries: limits.total_entries.min(1),
        inline_allocations: 0,
    };
    let projection = PersistedPhysicalRecoveryProjection::decode(encoded, copy_limits, format)
        .map_err(|denial| match denial {
            PhysicalRecoveryProjectionDenial::UnsupportedVersion(version) => {
                PhysicalRedoPlanningDenial::UnsupportedRecoveryProjectionVersion(version)
            }
            _ => PhysicalRedoPlanningDenial::InvalidRecoveryProjection,
        })?;
    let PersistedPhysicalRecoveryPayload::SourceCopy(recipe) = projection.payload() else {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    };
    if recipe.intent().operation() != operation
        || recipe.intent_lsn() >= publication_lsn
        || !projection.root_state().inline_allocations().is_empty()
        || projection.root_state().last_inline_record().is_some()
        || projection.root_state().last_inline_segment().is_some()
    {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    Ok(Some(projection))
}

pub(super) fn charge(
    retained: u64,
    copy: &PhysicalExtentCopyAdmission,
    limit: u64,
) -> Result<u64, PhysicalRedoPlanningDenial> {
    // Three page-sized buffers cover source admission, final encoding and the
    // executor's owned write. Descriptor charge is independent of extent size.
    let observed = u64::from(copy.recipe.intent().maximum_frame_bytes())
        .checked_mul(3)
        .and_then(|bytes| bytes.checked_add(4096))
        .and_then(|bytes| retained.checked_add(bytes))
        .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
    if observed > limit {
        return Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
            observed,
            admitted: limit,
        });
    }
    Ok(observed)
}

fn number(bytes: &mut &[u8]) -> Result<u64, PhysicalRedoPlanningDenial> {
    let (value, rest) = bytes
        .split_at_checked(8)
        .ok_or(PhysicalRedoPlanningDenial::MalformedMember)?;
    *bytes = rest;
    Ok(u64::from_le_bytes(value.try_into().map_err(|_| {
        PhysicalRedoPlanningDenial::MalformedMember
    })?))
}
fn field<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], PhysicalRedoPlanningDenial> {
    let length =
        usize::try_from(number(bytes)?).map_err(|_| PhysicalRedoPlanningDenial::MalformedMember)?;
    let (value, rest) = bytes
        .split_at_checked(length)
        .ok_or(PhysicalRedoPlanningDenial::MalformedMember)?;
    *bytes = rest;
    Ok(value)
}

#[cfg(test)]
mod tests;
