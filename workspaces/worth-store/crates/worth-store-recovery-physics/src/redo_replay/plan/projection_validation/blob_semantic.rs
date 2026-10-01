use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, DerivedFamilyRootDirectoryV1,
    PersistedPhysicalRecoveryOperation, PersistedPhysicalRecoveryProjection,
    PersistedRecordIdentity,
};

use super::PhysicalRedoPlanningDenial;

pub(super) fn validate_blob_semantic(
    bytes: &[u8],
    identity: PersistedRecordIdentity,
    store: [u8; 16],
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<(), PhysicalRedoPlanningDenial> {
    let payload_kind = decode_blob_record(bytes).ok().map(|value| {
        let embedded_store = match &value {
            BlobRecordV1::SessionDeclared(frame) => frame.store(),
            BlobRecordV1::Chunk(frame) => frame.occurrence().store(),
            BlobRecordV1::TreeNode(frame) => frame.occurrence().store(),
            BlobRecordV1::GenerationPublished(frame) => frame.store(),
            BlobRecordV1::SessionFrontier(frame) => frame.store(),
            BlobRecordV1::SessionAbandoned(frame) => frame.store(),
            BlobRecordV1::DropSetManifest(frame) => frame.store(),
            BlobRecordV1::DropSetManifestV2(frame) => frame.store(),
            BlobRecordV1::DropSetManifestV3(frame) => frame.store(),
            BlobRecordV1::OriginalDropReserved(frame) => frame.store(),
            BlobRecordV1::ReclaimDescriptor(frame) => frame.store(),
            BlobRecordV1::ReclaimDescriptorV2(frame) => frame.store(),
            BlobRecordV1::ReclaimDescriptorV3(frame) => frame.base().store(),
            BlobRecordV1::ChunkReuseClaim(frame) => frame.store(),
            BlobRecordV1::ChunkReuseClaimV2(frame) => frame.claim().store(),
            BlobRecordV1::DedupeQuarantine(frame) => frame.store(),
        };
        (value.kind(), embedded_store)
    });
    if payload_kind.is_some_and(|(_, embedded_store)| embedded_store != store) {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    let payload_kind = payload_kind.map(|(kind, _)| kind);
    let binding = match projection.operation() {
        PersistedPhysicalRecoveryOperation::None => {
            if bytes.starts_with(b"WRC11IDX") {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            if matches!(
                payload_kind,
                Some(
                    BlobRecordKind::SessionDeclared
                        | BlobRecordKind::GenerationPublished
                        | BlobRecordKind::SessionFrontier
                        | BlobRecordKind::SessionAbandoned
                        | BlobRecordKind::ChunkReuseClaim
                        | BlobRecordKind::ChunkReuseClaimV2
                        | BlobRecordKind::DedupeQuarantine
                        | BlobRecordKind::ReclaimDescriptor
                        | BlobRecordKind::ReclaimDescriptorV2
                        | BlobRecordKind::ReclaimDescriptorV3
                )
            ) {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            return Ok(());
        }
        PersistedPhysicalRecoveryOperation::SessionDeclared(binding) => {
            if payload_kind != Some(BlobRecordKind::SessionDeclared) {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            *binding
        }
        PersistedPhysicalRecoveryOperation::GenerationPublished(binding) => {
            if payload_kind != Some(BlobRecordKind::GenerationPublished) {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            *binding
        }
        PersistedPhysicalRecoveryOperation::SessionFrontier(binding) => {
            if payload_kind != Some(BlobRecordKind::SessionFrontier) {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            *binding
        }
        PersistedPhysicalRecoveryOperation::SessionAbandoned(binding) => {
            if payload_kind != Some(BlobRecordKind::SessionAbandoned) {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            *binding
        }
        PersistedPhysicalRecoveryOperation::ChunkReused(binding) => {
            if !matches!(
                payload_kind,
                Some(BlobRecordKind::ChunkReuseClaim | BlobRecordKind::ChunkReuseClaimV2)
            ) {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            *binding
        }
        PersistedPhysicalRecoveryOperation::DedupeQuarantined(binding) => {
            if payload_kind != Some(BlobRecordKind::DedupeQuarantine) {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            *binding
        }
        PersistedPhysicalRecoveryOperation::RecordsDropped { binding, .. } => {
            if !matches!(
                payload_kind,
                Some(
                    BlobRecordKind::ReclaimDescriptor
                        | BlobRecordKind::ReclaimDescriptorV2
                        | BlobRecordKind::ReclaimDescriptorV3
                )
            ) {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            *binding
        }
        PersistedPhysicalRecoveryOperation::DerivedDirectory {
            binding: directory, ..
        } => {
            if payload_kind.is_some() {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            let decoded = DerivedFamilyRootDirectoryV1::decode(bytes)
                .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
            if decoded.indexed_through_blob_publication() != directory.indexed_through()
                || directory
                    .indexed_through_quarantine()
                    .is_some_and(|expected| decoded.indexed_through_quarantine() != expected)
            {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            directory.record()
        }
    };
    if binding.record() != identity
        || projection.source_root_generation().checked_add(1)
            != Some(binding.candidate_root_generation())
        || binding.record_payload_sha256() != <[u8; 32]>::from(Sha256::digest(bytes))
    {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
