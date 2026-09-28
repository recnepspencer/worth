//! A copy is a bounded regeneration recipe, never an empty frame-target record.
use super::*;
use worth_store_physical_format::{PersistedExtentCopyRecipe, PersistedPhysicalRecoveryPayload};

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
    if !is_copy(member.canonical_redo()) {
        return Ok(None);
    }
    let mut bytes = member.canonical_redo();
    field(&mut bytes)?;
    let publication_lsn = number(&mut bytes)?;
    if publication_lsn != member.lsn_range().start().get()
        || publication_lsn.checked_add(1) != Some(member.lsn_range().end_exclusive().get())
    {
        return Err(PhysicalRedoPlanningDenial::LsnRangeMismatch);
    }
    let encoded = field(&mut bytes)?;
    if !bytes.is_empty() {
        return Err(PhysicalRedoPlanningDenial::MalformedMember);
    }
    let projection = PersistedPhysicalRecoveryProjection::decode(encoded, limits, format)
        .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    let PersistedPhysicalRecoveryPayload::SourceCopy(recipe) = projection.payload() else {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    };
    if recipe.intent().operation() != member.operation()
        || recipe.intent_lsn() >= publication_lsn
        || !projection.root_state().inline_allocations().is_empty()
        || projection.root_state().last_inline_record().is_some()
        || projection.root_state().last_inline_segment().is_some()
    {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    if member.fate() == RecoveryOperationFate::ProvenNoEffect {
        return Err(PhysicalRedoPlanningDenial::ProvenNoEffectHasWalAttempt);
    }
    Ok(Some(PhysicalExtentCopyAdmission {
        operation: member.operation(),
        group: member.group(),
        fate: member.fate(),
        publication_lsn,
        recipe: *recipe,
        projection,
    }))
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
