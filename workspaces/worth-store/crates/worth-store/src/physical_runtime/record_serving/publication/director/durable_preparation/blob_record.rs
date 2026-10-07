use sha2::Digest;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, PersistedRecordIdentity,
};

use super::{map_record_denial, ProtectedAppendKind, RecordPublicationDirector};
use crate::physical_runtime::{
    record_serving::{
        publication::{
            batch::RecordAppendInput, PhysicalManifestCapacityTransition,
            PhysicalMutationPreparationOutcome,
        },
        AdmittedRecordPlacementPolicy, RecordAppendBatch, RecordAppendDenial,
    },
    PhysicalMutationRequest,
};

const BLOB_MAGIC: &[u8; 8] = b"WRC11BLB";

impl RecordPublicationDirector {
    /// A V3 custody frame embeds the already-reserved drop fingerprint. The
    /// idempotency preimage must therefore remain its exact V2 base frame;
    /// the selected payload and WAL still bind the full V3 frame separately.
    pub(in crate::physical_runtime::record_serving::publication::director) fn prepare_released_descriptor_append(
        &self,
        descriptor: worth_store_physical_format::BlobReclaimDescriptorV3,
        directory_rebinding: Option<(
            crate::physical_runtime::record_serving::PreparedReleasedDirectoryRebinding,
            Vec<u8>,
        )>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let mut builder = RecordAppendBatch::builder().push_owned(descriptor.encode());
        let directory = directory_rebinding.is_some();
        if let Some((basis, bytes)) = directory_rebinding {
            let Ok(decoded) =
                worth_store_physical_format::DerivedFamilyRootDirectoryV1::decode(&bytes)
            else {
                return map_record_denial(RecordAppendDenial::InvalidDerivedDirectory);
            };
            if decoded.indexed_through_blob_publication().is_some()
                || decoded.indexed_through_quarantine() != basis.quarantined_through()
                || <[u8; 32]>::from(sha2::Sha256::digest(&bytes)) != basis.next_payload_sha256()
            {
                return map_record_denial(RecordAppendDenial::InvalidDerivedDirectory);
            }
            builder = builder.push_owned(bytes);
        }
        let batch = match builder.build() {
            Ok(batch) => batch,
            Err(denial) => return map_record_denial(denial),
        };
        self.prepare_durable_append_with_released_fingerprint(
            batch,
            placement,
            crate::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
            ProtectedAppendKind::ReleasedDrop { directory },
            Some(descriptor),
        )
    }

    pub(in crate::physical_runtime::record_serving::publication::director) fn prepare_blob_record_append(
        &self,
        encoded: Vec<u8>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let kind = match decode_blob_record(&encoded) {
            Ok(record) if blob_store(&record) == self.durability.store_identity().bytes() => {
                record.kind()
            }
            Err(_) => return map_record_denial(RecordAppendDenial::InvalidBlobRecord),
            _ => return map_record_denial(RecordAppendDenial::InvalidBlobRecord),
        };
        if matches!(
            kind,
            BlobRecordKind::DropSetManifest
                | BlobRecordKind::DropSetManifestV2
                | BlobRecordKind::DropSetManifestV3
                | BlobRecordKind::ReclaimDescriptor
                | BlobRecordKind::ReclaimDescriptorV2
                | BlobRecordKind::ReclaimDescriptorV3
                | BlobRecordKind::OriginalDropReserved
                | BlobRecordKind::ChunkReuseClaim
                | BlobRecordKind::ChunkReuseClaimV2
                | BlobRecordKind::DedupeQuarantine
        ) {
            return map_record_denial(RecordAppendDenial::BlobRecordRequiresStoreOwner);
        }
        let batch = match RecordAppendBatch::builder().push_owned(encoded).build() {
            Ok(batch) => batch,
            Err(denial) => return map_record_denial(denial),
        };
        self.prepare_durable_append_classified(
            batch,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
            super::ProtectedAppendKind::Blob(kind),
        )
    }

    pub(in crate::physical_runtime::record_serving::publication::director) fn prepare_blob_reuse_claim_append(
        &self,
        encoded: Vec<u8>,
        declaration_record: PersistedRecordIdentity,
        declaration_digest: [u8; 32],
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let claim = match decode_blob_record(&encoded) {
            Ok(BlobRecordV1::ChunkReuseClaimV2(claim))
                if claim.claim().store() == self.durability.store_identity().bytes() =>
            {
                claim.claim()
            }
            _ => return map_record_denial(RecordAppendDenial::InvalidBlobRecord),
        };
        if declaration_record == claim.selected_chunk()
            || declaration_record == claim.source_publication()
            || declaration_digest == [0; 32]
        {
            return map_record_denial(RecordAppendDenial::ReuseDestinationInvalid);
        }
        let batch = match RecordAppendBatch::builder().push_owned(encoded).build() {
            Ok(batch) => batch,
            Err(denial) => return map_record_denial(denial),
        };
        self.prepare_durable_append_classified(
            batch,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
            super::ProtectedAppendKind::BlobReuseClaim(
                crate::physical_runtime::record_serving::publication::PreparedReuseDeclarationBasis {
                    record: declaration_record,
                    digest: declaration_digest,
                },
            ),
        )
    }

    pub(in crate::physical_runtime::record_serving::publication::director) fn prepare_blob_dedupe_quarantine_append(
        &self,
        encoded: Vec<u8>,
        declaration_record: PersistedRecordIdentity,
        declaration_digest: [u8; 32],
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let claim = match decode_blob_record(&encoded) {
            Ok(BlobRecordV1::DedupeQuarantine(claim))
                if claim.store() == self.durability.store_identity().bytes() =>
            {
                claim
            }
            _ => return map_record_denial(RecordAppendDenial::InvalidBlobRecord),
        };
        if [
            claim.source_publication(),
            claim.source_chunk(),
            claim.conflicting_chunk(),
        ]
        .contains(&declaration_record)
            || declaration_digest == [0; 32]
        {
            return map_record_denial(RecordAppendDenial::ReuseDestinationInvalid);
        }
        let batch = match RecordAppendBatch::builder().push_owned(encoded).build() {
            Ok(batch) => batch,
            Err(denial) => return map_record_denial(denial),
        };
        self.prepare_durable_append_classified(
            batch,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
            super::ProtectedAppendKind::BlobDedupeQuarantine(
                crate::physical_runtime::record_serving::publication::PreparedReuseDeclarationBasis {
                    record: declaration_record,
                    digest: declaration_digest,
                },
            ),
        )
    }
}

fn blob_store(record: &BlobRecordV1<'_>) -> [u8; 16] {
    match record {
        BlobRecordV1::SessionDeclared(value) => value.store(),
        BlobRecordV1::Chunk(value) => value.occurrence().store(),
        BlobRecordV1::TreeNode(value) => value.occurrence().store(),
        BlobRecordV1::GenerationPublished(value) => value.store(),
        BlobRecordV1::SessionFrontier(value) => value.store(),
        BlobRecordV1::SessionAbandoned(value) => value.store(),
        BlobRecordV1::DropSetManifest(value) => value.store(),
        BlobRecordV1::DropSetManifestV2(value) => value.store(),
        BlobRecordV1::DropSetManifestV3(value) => value.store(),
        BlobRecordV1::OriginalDropReserved(value) => value.store(),
        BlobRecordV1::ReclaimDescriptor(value) => value.store(),
        BlobRecordV1::ReclaimDescriptorV2(value) => value.store(),
        BlobRecordV1::ReclaimDescriptorV3(value) => value.base().store(),
        BlobRecordV1::ChunkReuseClaim(value) => value.store(),
        BlobRecordV1::ChunkReuseClaimV2(value) => value.claim().store(),
        BlobRecordV1::DedupeQuarantine(value) => value.store(),
    }
}

pub(super) fn validate_prepared_payload(
    batch: &RecordAppendBatch,
    blob_record_kind: Option<BlobRecordKind>,
) -> Result<(), RecordAppendDenial> {
    match blob_record_kind {
        Some(expected) => {
            let [RecordAppendInput::Bytes(bytes)] = batch.records.as_slice() else {
                return Err(RecordAppendDenial::InvalidBlobRecord);
            };
            match decode_blob_record(bytes) {
                Ok(record) if record.kind() == expected => Ok(()),
                _ => Err(RecordAppendDenial::InvalidBlobRecord),
            }
        }
        None => {
            for input in &batch.records {
                let RecordAppendInput::Bytes(bytes) = input else {
                    unreachable!("durable preparation materializes every record source")
                };
                // Reserve the namespace even when a blob-shaped frame is
                // malformed. Otherwise an ordinary append can poison every
                // selected-root blob identity/publication scan.
                if bytes.starts_with(BLOB_MAGIC) {
                    return Err(RecordAppendDenial::BlobRecordRequiresStoreOwner);
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::BlobSessionDeclarationV1;

    #[test]
    fn canonical_blob_declaration_requires_typed_store_submission() {
        let bytes = BlobSessionDeclarationV1::new(
            [1; 16],
            [2; 16],
            [3; 16],
            [4; 32],
            64 << 10,
            100_000,
            1 << 20,
            12,
        )
        .unwrap()
        .encode();
        let batch = RecordAppendBatch::builder()
            .push_owned(bytes.clone())
            .build()
            .unwrap();
        assert_eq!(
            validate_prepared_payload(&batch, None),
            Err(RecordAppendDenial::BlobRecordRequiresStoreOwner),
        );
        assert_eq!(
            validate_prepared_payload(&batch, Some(BlobRecordKind::SessionDeclared)),
            Ok(()),
        );
        assert_eq!(
            validate_prepared_payload(&batch, Some(BlobRecordKind::Chunk)),
            Err(RecordAppendDenial::InvalidBlobRecord),
        );
        let mut damaged = bytes;
        damaged[16] ^= 1;
        let damaged = RecordAppendBatch::builder()
            .push_owned(damaged)
            .build()
            .unwrap();
        assert_eq!(
            validate_prepared_payload(&damaged, Some(BlobRecordKind::SessionDeclared)),
            Err(RecordAppendDenial::InvalidBlobRecord),
        );
    }

    #[test]
    fn malformed_blob_namespace_cannot_be_appended_as_ordinary_record() {
        for bytes in [
            b"WRC11BLB".as_slice(),
            b"WRC11BLB\x01".as_slice(),
            b"WRC11BLB\xfftruncated".as_slice(),
        ] {
            let batch = RecordAppendBatch::builder()
                .push_owned(bytes.to_vec())
                .build()
                .unwrap();
            assert_eq!(
                validate_prepared_payload(&batch, None),
                Err(RecordAppendDenial::BlobRecordRequiresStoreOwner),
            );
        }
    }
}
