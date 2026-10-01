use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobRecordKind, PersistedBlobSemanticRecordBinding, PersistedPhysicalRecoveryBlobSemantic,
};

use crate::physical_runtime::PreparedPhysicalRootProjection;

pub(super) fn blob_semantic(
    kind: Option<BlobRecordKind>,
    prepared_bytes: &[Vec<u8>],
    root: &PreparedPhysicalRootProjection,
) -> PersistedPhysicalRecoveryBlobSemantic {
    let Some(
        kind @ (BlobRecordKind::SessionDeclared
        | BlobRecordKind::GenerationPublished
        | BlobRecordKind::SessionFrontier
        | BlobRecordKind::SessionAbandoned
        | BlobRecordKind::ChunkReuseClaim
        | BlobRecordKind::ChunkReuseClaimV2
        | BlobRecordKind::DedupeQuarantine
        | BlobRecordKind::ReclaimDescriptor
        | BlobRecordKind::ReclaimDescriptorV2
        | BlobRecordKind::ReclaimDescriptorV3),
    ) = kind
    else {
        return PersistedPhysicalRecoveryBlobSemantic::None;
    };
    let [bytes] = prepared_bytes else {
        unreachable!("typed blob append prepares exactly one record")
    };
    let records: Vec<_> = root.recovery_record_identities().collect();
    let [record] = records.as_slice() else {
        unreachable!("typed blob append plans exactly one record identity")
    };
    let binding = PersistedBlobSemanticRecordBinding::new(
        *record,
        Sha256::digest(bytes).into(),
        root.source_root_generation()
            .checked_add(1)
            .expect("planned successor root generation is available"),
    )
    .expect("planned successor root generation is nonzero");
    match kind {
        BlobRecordKind::SessionDeclared => {
            PersistedPhysicalRecoveryBlobSemantic::SessionDeclared(binding)
        }
        BlobRecordKind::GenerationPublished => {
            PersistedPhysicalRecoveryBlobSemantic::GenerationPublished(binding)
        }
        BlobRecordKind::SessionFrontier => {
            PersistedPhysicalRecoveryBlobSemantic::SessionFrontier(binding)
        }
        BlobRecordKind::SessionAbandoned => {
            PersistedPhysicalRecoveryBlobSemantic::SessionAbandoned(binding)
        }
        BlobRecordKind::ChunkReuseClaim | BlobRecordKind::ChunkReuseClaimV2 => {
            PersistedPhysicalRecoveryBlobSemantic::ChunkReused(binding)
        }
        BlobRecordKind::DedupeQuarantine => {
            PersistedPhysicalRecoveryBlobSemantic::DedupeQuarantined(binding)
        }
        BlobRecordKind::ReclaimDescriptor
        | BlobRecordKind::ReclaimDescriptorV2
        | BlobRecordKind::ReclaimDescriptorV3 => {
            PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(binding)
        }
        _ => unreachable!(),
    }
}
