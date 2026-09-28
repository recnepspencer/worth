use super::super::RetainedExtentCopyObligation;
use crate::physical_runtime::durability::{
    PersistedPhysicalMutationAttemptBinding, PhysicalBindingDecodingContext,
};
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    PersistedPhysicalRecoveryPayload, PersistedPhysicalRecoveryProjection,
    PhysicalRecordFormatDeclaration, PhysicalRecoveryProjectionDecodeLimits,
};

/// Called only on a verified WAL frame, including frames below checkpoint.
pub(super) fn observe(
    payload: &[u8],
    range: worth_store_wal::WalLsnRange,
    format: PhysicalRecordFormatDeclaration,
    context: PhysicalBindingDecodingContext,
    obligations: &mut [RetainedExtentCopyObligation],
    checkpoint: u64,
) -> Result<(), ()> {
    let mut payload = payload;
    let binding_bytes = field(&mut payload)?;
    let redo = field(&mut payload)?;
    if !payload.is_empty() {
        return Err(());
    }
    let mut body = redo;
    if field(&mut body).ok() != Some(b"store.physical.extent-copy-publication.v1".as_slice()) {
        return Ok(());
    }
    let binding = PersistedPhysicalMutationAttemptBinding::decode_from_wal_member(
        binding_bytes,
        context,
        range,
        Sha256::digest(redo).into(),
    )
    .map_err(|_| ())?;
    observe_bound(redo, &binding, format, obligations, checkpoint)
}

pub(in crate::physical_runtime::durability::wal) fn observe_bound(
    redo: &[u8],
    binding: &PersistedPhysicalMutationAttemptBinding,
    format: PhysicalRecordFormatDeclaration,
    obligations: &mut [RetainedExtentCopyObligation],
    checkpoint: u64,
) -> Result<(), ()> {
    let mut body = redo;
    if field(&mut body).ok() != Some(b"store.physical.extent-copy-publication.v1".as_slice()) {
        return Ok(());
    }
    if Sha256::digest(redo).as_slice() != binding.redo_digest() {
        return Err(());
    }
    let range = binding.member().lsn_range();
    let lsn = number(&mut body)?;
    if lsn != range.start().get() || range.end_exclusive().get() != lsn.checked_add(1).ok_or(())? {
        return Err(());
    }
    let encoded = field(&mut body)?;
    if !body.is_empty() {
        return Err(());
    }
    let limit = encoded.len() as u64;
    let projection = PersistedPhysicalRecoveryProjection::decode(
        encoded,
        PhysicalRecoveryProjectionDecodeLimits {
            frames: 0,
            record_identities: 1,
            placements: 1,
            segment_updates: 0,
            manifests: 0,
            total_entries: limit,
            inline_allocations: limit,
        },
        format,
    )
    .map_err(|_| ())?;
    let PersistedPhysicalRecoveryPayload::SourceCopy(recipe) = projection.payload() else {
        return Err(());
    };
    if recipe.intent().operation() != binding.idempotency_identity().bytes()
        || recipe.intent_lsn() >= lsn
    {
        return Err(());
    }
    let Some(entry) = obligations
        .iter_mut()
        .find(|entry| entry.intent().operation() == recipe.intent().operation())
    else {
        return if range.end_exclusive().get() <= checkpoint {
            Ok(())
        } else {
            Err(())
        };
    };
    if entry.intent() != recipe.intent()
        || entry.intent_lsn() != recipe.intent_lsn()
        || entry.intent_digest() != recipe.intent_digest()
        || entry.publication.is_some()
        || entry.resolution().is_some()
    {
        return Err(());
    }
    entry.publication = Some((
        projection
            .source_root_generation()
            .checked_add(1)
            .ok_or(())?,
        lsn,
    ));
    Ok(())
}

fn number(bytes: &mut &[u8]) -> Result<u64, ()> {
    let (value, remaining) = bytes.split_at_checked(8).ok_or(())?;
    *bytes = remaining;
    Ok(u64::from_le_bytes(value.try_into().map_err(|_| ())?))
}
fn field<'a>(bytes: &mut &'a [u8]) -> Result<&'a [u8], ()> {
    let size = usize::try_from(number(bytes)?).map_err(|_| ())?;
    let (value, remaining) = bytes.split_at_checked(size).ok_or(())?;
    *bytes = remaining;
    Ok(value)
}
