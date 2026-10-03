use sha2::{Digest, Sha256};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    BlobReclaimSourceBasisV1, BlobRecordV1, PersistedPhysicalRecoveryOperation,
    PersistedPhysicalRecoveryProjection, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadMutationV1,
};

use super::{PhysicalRedoPlanningDenial, PhysicalRedoRecord};

pub(super) fn validate_release_head_effect(
    records: &[PhysicalRedoRecord],
    projection: &PersistedPhysicalRecoveryProjection,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let PersistedPhysicalRecoveryOperation::RecordsDropped {
        head_effect: Some(effect),
        ..
    } = projection.operation()
    else {
        return Ok(());
    };
    let Some(record) = records.first() else {
        return invalid();
    };
    let Some(identity) = projection.record_identities().first() else {
        return invalid();
    };
    validate_head_descriptor(record.bytes(), *identity, projection, store, format, effect)
}

fn validate_head_descriptor(
    bytes: &[u8],
    identity: worth_store_physical_format::PersistedRecordIdentity,
    projection: &PersistedPhysicalRecoveryProjection,
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    effect: &worth_store_physical_format::PersistedReleaseCustodyHeadEffectV1,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let Ok(BlobRecordV1::ReclaimDescriptorV3(descriptor)) =
        worth_store_physical_format::decode_blob_record(bytes)
    else {
        return invalid();
    };
    let base = descriptor.base();
    let ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior,
        next,
    } = effect.mutation()
    else {
        return invalid();
    };
    let basis = effect.source_basis();
    let basis_digest = BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest(store.bytes());
    let PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } = projection.operation()
    else {
        return invalid();
    };
    if basis.publication().store() != store.bytes()
        || basis_digest != base.source_basis_digest()
        || next.source_basis_digest() != basis_digest
        || next.key().object() != basis.object()
        || next.key().generation() != basis.generation()
        || next.descriptor_record() != identity
        || next.descriptor_frame_sha256() != <[u8; 32]>::from(Sha256::digest(bytes))
        || next.manifest_record() != base.manifest_record()
        || next.manifest_frame_sha256() != base.manifest_frame_sha256()
        || next.predecessor() != base.predecessor()
        || next.cumulative_dropped() != base.cumulative_dropped()
        || next.terminal() != base.terminal()
        || next.source_root_generation() != base.source_root_generation()
        || base.source_root_generation() != projection.source_root_generation()
        || base.candidate_root_generation() != binding.candidate_root_generation()
        || binding.record() != identity
        || binding.record_payload_sha256() != <[u8; 32]>::from(Sha256::digest(bytes))
    {
        return invalid();
    }
    match (base.predecessor(), expected_prior) {
        (None, None) if next.cumulative_dropped() == u64::from(base.manifest_count()) => {}
        (Some(previous), Some(prior))
            if prior.key() == next.key()
                && !prior.terminal()
                && prior.descriptor_record() == previous.descriptor_record()
                && prior.descriptor_frame_sha256() == previous.descriptor_frame_sha256()
                && prior
                    .cumulative_dropped()
                    .checked_add(u64::from(base.manifest_count()))
                    == Some(next.cumulative_dropped()) => {}
        _ => return invalid(),
    }
    effect
        .verify_exact(projection.source_root_generation(), format)
        .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
    Ok(())
}

fn invalid<T>() -> Result<T, PhysicalRedoPlanningDenial> {
    Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
}

#[cfg(test)]
mod tests;
