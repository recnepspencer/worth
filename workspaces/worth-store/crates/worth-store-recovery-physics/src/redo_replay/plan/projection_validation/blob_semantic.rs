use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    validate_blob_record_header, BlobRecordKind, DerivedFamilyRootDirectoryV1,
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
    let payload_kind = validate_blob_record_header(bytes)
        .ok()
        .map(|header| (header.kind(), header.store()));
    if payload_kind.is_some_and(|(_, embedded_store)| embedded_store != store) {
        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
    }
    let payload_kind = payload_kind.map(|(kind, _)| kind);
    let binding = match projection.operation() {
        PersistedPhysicalRecoveryOperation::RecordAnchorAdvanced(anchored) => {
            let advance = anchored.advance();
            if bytes.starts_with(b"WRC11IDX")
                || payload_kind.is_some()
                || (identity == advance.record()
                    && advance.record_payload_sha256() != <[u8; 32]>::from(Sha256::digest(bytes)))
            {
                return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
            }
            return Ok(());
        }
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
        PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            directory_replacement,
            ..
        } => {
            if let Some(replacement) = directory_replacement {
                let next = replacement.next();
                if identity == next.record().record() {
                    if payload_kind.is_some() {
                        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
                    }
                    let decoded = DerivedFamilyRootDirectoryV1::validate_header(bytes)
                        .map_err(|_| PhysicalRedoPlanningDenial::InvalidRecoveryProjection)?;
                    if decoded.indexed_through_blob_publication().is_some()
                        || next.indexed_through_quarantine().is_some_and(|expected| {
                            decoded.indexed_through_quarantine() != expected
                        })
                    {
                        return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
                    }
                    return validate_binding(next.record(), identity, bytes, projection);
                }
            }
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
            let decoded = DerivedFamilyRootDirectoryV1::validate_header(bytes)
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
        // A terminal head retirement member carries no data record at all.
        PersistedPhysicalRecoveryOperation::TerminalReleaseHeadRetired(_) => {
            return Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection);
        }
    };
    validate_binding(binding, identity, bytes, projection)
}

fn validate_binding(
    binding: worth_store_physical_format::PersistedBlobSemanticRecordBinding,
    identity: PersistedRecordIdentity,
    bytes: &[u8],
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<(), PhysicalRedoPlanningDenial> {
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
