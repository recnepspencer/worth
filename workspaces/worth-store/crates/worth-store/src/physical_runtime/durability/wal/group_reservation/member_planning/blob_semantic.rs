use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobRecordKind, PersistedBlobSemanticRecordBinding, PersistedDerivedDirectoryRecordBinding,
    PersistedPhysicalRecoveryOperation, PersistedReleasedDirectoryReplacementV1,
};

use crate::physical_runtime::PreparedPhysicalRootProjection;

pub(super) fn blob_semantic(
    kind: Option<BlobRecordKind>,
    prepared_bytes: &[Vec<u8>],
    root: &PreparedPhysicalRootProjection,
) -> PersistedPhysicalRecoveryOperation {
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
        return PersistedPhysicalRecoveryOperation::None;
    };
    if let Some(rebinding) = root.recovery_released_directory_rebinding() {
        let [descriptor_bytes, directory_bytes] = prepared_bytes else {
            unreachable!("released directory rebinding prepares exactly two records")
        };
        let records: Vec<_> = root.recovery_record_identities().collect();
        let [descriptor_record, directory_record] = records.as_slice() else {
            unreachable!("released directory rebinding plans exactly two records")
        };
        assert_eq!(kind, BlobRecordKind::ReclaimDescriptorV3);
        assert_eq!(
            <[u8; 32]>::from(Sha256::digest(directory_bytes)),
            rebinding.next_payload_sha256()
        );
        let generation = root
            .source_root_generation()
            .checked_add(1)
            .expect("successor generation");
        let descriptor = PersistedBlobSemanticRecordBinding::new(
            *descriptor_record,
            Sha256::digest(descriptor_bytes).into(),
            generation,
        )
        .expect("release descriptor has a nonzero successor generation");
        let directory = PersistedBlobSemanticRecordBinding::new(
            *directory_record,
            Sha256::digest(directory_bytes).into(),
            generation,
        )
        .expect("released directory has a nonzero successor generation");
        let next = PersistedDerivedDirectoryRecordBinding::new_with_quarantine(
            directory,
            None,
            rebinding.quarantined_through(),
        );
        return PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding: descriptor,
            head_effect: root.recovery_release_head_effect().cloned(),
            directory_replacement: Some(
                PersistedReleasedDirectoryReplacementV1::new(
                    rebinding.expected_previous(),
                    rebinding.expected_previous_payload_sha256(),
                    next,
                )
                .expect("selected source and admitted successor directory are distinct"),
            ),
        };
    }
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
            PersistedPhysicalRecoveryOperation::SessionDeclared(binding)
        }
        BlobRecordKind::GenerationPublished => {
            PersistedPhysicalRecoveryOperation::GenerationPublished(binding)
        }
        BlobRecordKind::SessionFrontier => {
            PersistedPhysicalRecoveryOperation::SessionFrontier(binding)
        }
        BlobRecordKind::SessionAbandoned => {
            PersistedPhysicalRecoveryOperation::SessionAbandoned(binding)
        }
        BlobRecordKind::ChunkReuseClaim | BlobRecordKind::ChunkReuseClaimV2 => {
            PersistedPhysicalRecoveryOperation::ChunkReused(binding)
        }
        BlobRecordKind::DedupeQuarantine => {
            PersistedPhysicalRecoveryOperation::DedupeQuarantined(binding)
        }
        BlobRecordKind::ReclaimDescriptor
        | BlobRecordKind::ReclaimDescriptorV2
        | BlobRecordKind::ReclaimDescriptorV3 => {
            PersistedPhysicalRecoveryOperation::RecordsDropped {
                binding,
                head_effect: root.recovery_release_head_effect().cloned(),
                directory_replacement: None,
            }
        }
        _ => unreachable!(),
    }
}
