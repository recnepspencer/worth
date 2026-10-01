use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    BTreeNodeV1, BlobRecordKind, DerivedFamilyRootDirectoryBinding, DerivedFamilyRootDirectoryV1,
    SelectedRecordContentClass,
};

use super::{blob_record, map_record_denial, RecordPublicationDirector};
use crate::physical_runtime::{
    durability::PhysicalMutationOperationFamily,
    record_serving::{
        publication::{
            batch::RecordAppendInput, durable_preparation::PreparedReuseDeclarationBasis,
            PhysicalManifestCapacityTransition, PhysicalMutationPreparationOutcome,
            PreparedDerivedDirectoryBasis,
        },
        AdmittedRecordPlacementPolicy, RecordAppendBatch, RecordAppendDenial,
    },
    PhysicalMutationRequest,
};

const DIRECTORY_MAGIC: &[u8; 8] = b"WRC11IDX";
const BTREE_MAGIC: &[u8; 8] = b"WRC11BTN";

#[derive(Clone)]
pub(in crate::physical_runtime::record_serving::publication::director) enum ProtectedAppendKind {
    Ordinary,
    Blob(BlobRecordKind),
    BlobReuseClaim(PreparedReuseDeclarationBasis),
    BlobDedupeQuarantine(PreparedReuseDeclarationBasis),
    Directory(PreparedDerivedDirectoryBasis),
    BTreeNode(u16),
}

impl ProtectedAppendKind {
    pub(super) const fn selected_content_class(&self) -> SelectedRecordContentClass {
        match self {
            Self::Ordinary => SelectedRecordContentClass::Opaque,
            Self::Blob(kind) => SelectedRecordContentClass::Blob(*kind),
            Self::BlobReuseClaim(_) => {
                SelectedRecordContentClass::Blob(BlobRecordKind::ChunkReuseClaimV2)
            }
            Self::BlobDedupeQuarantine(_) => {
                SelectedRecordContentClass::Blob(BlobRecordKind::DedupeQuarantine)
            }
            Self::Directory(_) => SelectedRecordContentClass::DerivedDirectory,
            Self::BTreeNode(family_code) => SelectedRecordContentClass::BTreeNode {
                family_code: *family_code,
            },
        }
    }

    pub(super) const fn blob_kind(&self) -> Option<BlobRecordKind> {
        match self {
            Self::Blob(kind) => Some(*kind),
            Self::BlobReuseClaim(_) => Some(BlobRecordKind::ChunkReuseClaimV2),
            Self::BlobDedupeQuarantine(_) => Some(BlobRecordKind::DedupeQuarantine),
            _ => None,
        }
    }

    pub(super) const fn inline_only(&self) -> bool {
        matches!(self, Self::BTreeNode(_))
    }

    pub(super) fn directory_basis(&self) -> Option<PreparedDerivedDirectoryBasis> {
        match self {
            Self::Directory(basis) => Some(basis.clone()),
            _ => None,
        }
    }

    pub(super) const fn reuse_declaration_basis(&self) -> Option<PreparedReuseDeclarationBasis> {
        match self {
            Self::BlobReuseClaim(basis) | Self::BlobDedupeQuarantine(basis) => Some(*basis),
            _ => None,
        }
    }

    pub(super) const fn operation_family(&self) -> PhysicalMutationOperationFamily {
        match self {
            Self::Ordinary => PhysicalMutationOperationFamily::RecordAppend,
            Self::Blob(_) => PhysicalMutationOperationFamily::BlobRecordAppend,
            Self::BlobReuseClaim(_) => PhysicalMutationOperationFamily::BlobRecordAppend,
            Self::BlobDedupeQuarantine(_) => PhysicalMutationOperationFamily::BlobRecordAppend,
            Self::Directory(_) => PhysicalMutationOperationFamily::DerivedDirectoryAppend,
            Self::BTreeNode(_) => PhysicalMutationOperationFamily::BTreeNodeAppend,
        }
    }
}

pub(super) fn validate_prepared_payload(
    batch: &RecordAppendBatch,
    classified: &ProtectedAppendKind,
) -> Result<(), RecordAppendDenial> {
    match classified {
        ProtectedAppendKind::Ordinary => {
            blob_record::validate_prepared_payload(batch, None)?;
            for input in &batch.records {
                let RecordAppendInput::Bytes(bytes) = input else {
                    unreachable!()
                };
                if bytes.starts_with(DIRECTORY_MAGIC) {
                    return Err(RecordAppendDenial::DerivedDirectoryRequiresStoreOwner);
                }
                if bytes.starts_with(BTREE_MAGIC) {
                    return Err(RecordAppendDenial::BTreeNodeRequiresStoreOwner);
                }
            }
            Ok(())
        }
        ProtectedAppendKind::Blob(kind) => {
            if matches!(
                kind,
                BlobRecordKind::ChunkReuseClaim
                    | BlobRecordKind::ChunkReuseClaimV2
                    | BlobRecordKind::DedupeQuarantine
            ) {
                return Err(RecordAppendDenial::BlobRecordRequiresStoreOwner);
            }
            blob_record::validate_prepared_payload(batch, Some(*kind))
        }
        ProtectedAppendKind::BlobReuseClaim(_) => {
            blob_record::validate_prepared_payload(batch, Some(BlobRecordKind::ChunkReuseClaimV2))
        }
        ProtectedAppendKind::BlobDedupeQuarantine(_) => {
            blob_record::validate_prepared_payload(batch, Some(BlobRecordKind::DedupeQuarantine))
        }
        ProtectedAppendKind::Directory(basis) => {
            let [RecordAppendInput::Bytes(bytes)] = batch.records.as_slice() else {
                return Err(RecordAppendDenial::InvalidDerivedDirectory);
            };
            let directory = DerivedFamilyRootDirectoryV1::decode(bytes)
                .map_err(|_| RecordAppendDenial::InvalidDerivedDirectory)?;
            if directory.indexed_through_blob_publication() != basis.indexed_through
                || directory.indexed_through_quarantine() != basis.indexed_through_quarantine
            {
                return Err(RecordAppendDenial::InvalidDerivedDirectory);
            }
            Ok(())
        }
        ProtectedAppendKind::BTreeNode(family_code) => {
            let [RecordAppendInput::Bytes(bytes)] = batch.records.as_slice() else {
                return Err(RecordAppendDenial::InvalidBTreeNode);
            };
            let node =
                BTreeNodeV1::decode(bytes).map_err(|_| RecordAppendDenial::InvalidBTreeNode)?;
            (node.family_code() == *family_code)
                .then_some(())
                .ok_or(RecordAppendDenial::InvalidBTreeNode)
        }
    }
}

impl RecordPublicationDirector {
    pub(in crate::physical_runtime::record_serving::publication::director) fn prepare_blob_derived_directory_append(
        &self,
        encoded: Vec<u8>,
        expected_previous: Option<DerivedFamilyRootDirectoryBinding>,
        replaced_nodes: crate::physical_runtime::layout::AdmittedDirectoryRetirement<'_>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let directory = match DerivedFamilyRootDirectoryV1::decode(&encoded) {
            Ok(directory) => directory,
            Err(_) => return map_record_denial(RecordAppendDenial::InvalidDerivedDirectory),
        };
        let current = self.root_owner.snapshot().0;
        let previous = current.derived_family_directory();
        if current.latest_blob_publication() != directory.indexed_through_blob_publication()
            || current.latest_blob_quarantine() != directory.indexed_through_quarantine()
            || previous != expected_previous
        {
            return map_record_denial(RecordAppendDenial::DerivedDirectorySourceChanged);
        }
        let Some(basis) = PreparedDerivedDirectoryBasis::from_admitted(
            expected_previous,
            directory.indexed_through_blob_publication(),
            directory.indexed_through_quarantine(),
            replaced_nodes,
            self.durability.store_identity(),
            self.durability.runtime_identity(),
            self.generation,
        ) else {
            return map_record_denial(RecordAppendDenial::DerivedDirectorySourceChanged);
        };
        let batch = match RecordAppendBatch::builder().push_owned(encoded).build() {
            Ok(batch) => batch,
            Err(denial) => return map_record_denial(denial),
        };
        self.prepare_durable_append_classified(
            batch,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
            ProtectedAppendKind::Directory(basis),
        )
    }

    pub(in crate::physical_runtime::record_serving::publication::director) fn prepare_btree_node_append(
        &self,
        encoded: Vec<u8>,
        expected_family: DurableArtifactFamilyId,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let family_code = match expected_family {
            DurableArtifactFamilyId::BlobCatalog => 1,
            DurableArtifactFamilyId::DedupeIndex => 2,
            _ => return map_record_denial(RecordAppendDenial::InvalidBTreeNode),
        };
        let batch = match RecordAppendBatch::builder().push_owned(encoded).build() {
            Ok(batch) => batch,
            Err(denial) => return map_record_denial(denial),
        };
        self.prepare_durable_append_classified(
            batch,
            placement,
            PhysicalManifestCapacityTransition::PreserveCurrent,
            request,
            ProtectedAppendKind::BTreeNode(family_code),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::IndexedThroughBlobPublication;

    #[test]
    fn ordinary_append_cannot_poison_protected_index_namespaces() {
        for (bytes, denial) in [
            (
                b"WRC11IDX".as_slice(),
                RecordAppendDenial::DerivedDirectoryRequiresStoreOwner,
            ),
            (
                b"WRC11BTN".as_slice(),
                RecordAppendDenial::BTreeNodeRequiresStoreOwner,
            ),
        ] {
            let batch = RecordAppendBatch::builder()
                .push_owned(bytes.to_vec())
                .build()
                .unwrap();
            assert_eq!(
                validate_prepared_payload(&batch, &ProtectedAppendKind::Ordinary),
                Err(denial),
            );
        }
    }

    #[test]
    fn typed_directory_requires_exact_embedded_source_marker() {
        let directory = DerivedFamilyRootDirectoryV1::new(vec![]).unwrap();
        let batch = RecordAppendBatch::builder()
            .push_owned(directory.encode())
            .build()
            .unwrap();
        let record = worth_store_physical_format::PersistedRecordIdentity::new([7; 16], 1).unwrap();
        let marker = IndexedThroughBlobPublication::new(4, record, [8; 32]).unwrap();
        assert_eq!(
            validate_prepared_payload(
                &batch,
                &ProtectedAppendKind::Directory(
                    PreparedDerivedDirectoryBasis::empty_for_validation(None, Some(marker), None,)
                )
            ),
            Err(RecordAppendDenial::InvalidDerivedDirectory),
        );
    }
}
