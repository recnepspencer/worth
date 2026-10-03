mod append;
#[cfg(feature = "certification-test-authority")]
mod certification;
mod tree;

use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobRecordDenial, DerivedFamilyDirectoryDenial,
    DerivedFamilyRootDirectoryBinding, DerivedFamilyRootDirectoryV1, DerivedFamilyRootEntry,
    IndexedThroughBlobPublication, PersistedRecordIdentity,
};

use crate::physical_runtime::{
    blob::{source_publication_unrouted, verify_source, DedupeIndexKey, DedupeIndexValue},
    layout::{
        rebuild::{
            insert_cell, point_temporary, traverse_publication_direct, LayoutRebuildLimits,
            SelectedBlobPublication,
        },
        rebuild_blob_derived_indexes, LayoutRebuildFailure, PhysicalIndexPointKey,
        PhysicalIndexPointKeyDenial, PhysicalLayoutPagePort, PhysicalLayoutPageReadFailure,
    },
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, PhysicalReadProtectionDenial,
    PhysicalScopedAllocationFailure, ServingPhysicalRuntime,
};

pub use append::PhysicalLayoutAppendFailure;
use append::{append_layout_record, LayoutAppendKind};
pub use tree::DeferredDerivedRetirementCause;
pub(in crate::physical_runtime) use tree::{
    admit_directory_retirement, insert_registered_node, inspect_selected_tree_retirement,
    retire_selected_tree, write_registered_cell, AdmittedDirectoryRetirement, InsertedLayoutTree,
    InsertionSource, LayoutCellWrite, SelectedTreeRetirement,
};

pub(in crate::physical_runtime) fn publish_derived_directory(
    runtime: &ServingPhysicalRuntime,
    directory: DerivedFamilyRootDirectoryV1,
    expected_previous: Option<DerivedFamilyRootDirectoryBinding>,
    replaced_nodes: AdmittedDirectoryRetirement<'_>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<PersistedRecordIdentity, PhysicalLayoutAppendFailure> {
    append_layout_record(
        runtime,
        LayoutAppendKind::DerivedDirectory {
            previous: expected_previous,
            replaced_nodes,
        },
        directory.encode(),
        placement,
        deadline,
    )
}

#[derive(Debug)]
pub enum PhysicalLayoutMaintenanceFailure {
    RootProtection(PhysicalReadProtectionDenial),
    SourceNotSelected,
    PriorWatermarkMismatch,
    SourceRead(PhysicalLayoutPageReadFailure),
    SourceDigestMismatch,
    SourceFormat(BlobRecordDenial),
    SourceIdentityMismatch,
    PointKey(PhysicalIndexPointKeyDenial),
    DirectoryRead(PhysicalLayoutPageReadFailure),
    DirectoryFormat(DerivedFamilyDirectoryDenial),
    NodeRead(PhysicalLayoutPageReadFailure),
    NodeIntegrity(worth_store_physical_integrity::PhysicalIntegrityRejection),
    NodeFormat(worth_store_physical_format::BTreeNodeDenial),
    WriterAllocation(PhysicalScopedAllocationFailure),
    TreeTopologyDamaged,
    TreeHeightLimit,
    RetirementLimit,
    ConflictingKey,
    InvalidFamilyCell,
    UnregisteredFamily(DurableArtifactFamilyId),
    DedupeRequiresRebuild,
    FullAuthority(Box<LayoutRebuildFailure>),
    Append(PhysicalLayoutAppendFailure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalLayoutMaintenanceReceipt {
    source: IndexedThroughBlobPublication,
    catalog_root: PersistedRecordIdentity,
    dedupe_root: Option<PersistedRecordIdentity>,
    directory_record: PersistedRecordIdentity,
}

impl PhysicalLayoutMaintenanceReceipt {
    pub const fn source(self) -> IndexedThroughBlobPublication {
        self.source
    }

    pub const fn catalog_root(self) -> PersistedRecordIdentity {
        self.catalog_root
    }

    pub const fn dedupe_root(self) -> Option<PersistedRecordIdentity> {
        self.dedupe_root
    }

    pub const fn directory_record(self) -> PersistedRecordIdentity {
        self.directory_record
    }
}

impl ServingPhysicalRuntime {
    /// Advances the blob catalog from one *trusted selected* publication
    /// watermark. Node appends may leave unreachable residue on failure; only
    /// the final classified directory append makes the new root serveable.
    pub(in crate::physical_runtime) fn maintain_blob_catalog_publication(
        &self,
        source: IndexedThroughBlobPublication,
        previous_source: Option<IndexedThroughBlobPublication>,
        object: [u8; 16],
        generation: u64,
        placement: AdmittedRecordPlacementPolicy,
        deadline: PhysicalMutationDeadline,
    ) -> Result<PhysicalLayoutMaintenanceReceipt, PhysicalLayoutMaintenanceFailure> {
        let reader = self
            .records()
            .map_err(PhysicalLayoutMaintenanceFailure::RootProtection)?;
        if reader.selected_latest_blob_publication() != Some(source) {
            return Err(PhysicalLayoutMaintenanceFailure::SourceNotSelected);
        }
        let binding = reader.selected_derived_family_directory();
        if binding.and_then(|selected| selected.indexed_through_blob_publication())
            != previous_source
        {
            return Err(PhysicalLayoutMaintenanceFailure::PriorWatermarkMismatch);
        }
        if previous_source.is_some() && binding.is_none() {
            return Err(PhysicalLayoutMaintenanceFailure::PriorWatermarkMismatch);
        }
        let port = PhysicalLayoutPagePort::from_protected_reader(
            self,
            reader,
            self.maximum_inline_record_bytes(),
        )
        .map_err(PhysicalLayoutMaintenanceFailure::SourceRead)?;
        let source_read = port
            .read_node(source.record())
            .map_err(PhysicalLayoutMaintenanceFailure::SourceRead)?;
        let digest: [u8; 32] = Sha256::digest(source_read.bytes()).into();
        if digest != source.encoded_digest() {
            return Err(PhysicalLayoutMaintenanceFailure::SourceDigestMismatch);
        }
        let publication = BlobGenerationPublicationV1::decode(source_read.bytes())
            .map_err(PhysicalLayoutMaintenanceFailure::SourceFormat)?;
        if publication.store() != self.store_identity().bytes()
            || publication.object() != object
            || publication.generation() != generation
        {
            return Err(PhysicalLayoutMaintenanceFailure::SourceIdentityMismatch);
        }
        // Once a quarantine exists, incremental insertion cannot infer from
        // an absent derived leaf whether the basis was suppressed. Rebuild
        // from selected facts so every subsequent publication preserves the
        // no-reuse set. This rare path has an explicit selected-root bound.
        if port.reader().selected_latest_blob_quarantine().is_some() {
            let bound = port
                .reader()
                .selected_record_count()
                .checked_add(1)
                .and_then(NonZeroU64::new)
                .ok_or_else(|| {
                    PhysicalLayoutMaintenanceFailure::FullAuthority(Box::new(
                        LayoutRebuildFailure::SelectedBoundExhausted,
                    ))
                })?;
            let receipt = rebuild_blob_derived_indexes(
                self,
                LayoutRebuildLimits::new(bound, bound),
                placement,
                deadline,
            )
            .map_err(|failure| {
                PhysicalLayoutMaintenanceFailure::FullAuthority(Box::new(failure))
            })?;
            if receipt.source() != source {
                return Err(PhysicalLayoutMaintenanceFailure::SourceNotSelected);
            }
            return Ok(PhysicalLayoutMaintenanceReceipt {
                source,
                catalog_root: receipt.catalog_root(),
                dedupe_root: receipt.dedupe_root(),
                directory_record: receipt.directory_record(),
            });
        }
        let selected = SelectedBlobPublication {
            record: source.record(),
            frame_digest: source.encoded_digest(),
            publication,
        };
        traverse_publication_direct(self, port.reader(), selected, |_| Ok(())).map_err(
            |failure| PhysicalLayoutMaintenanceFailure::FullAuthority(Box::new(failure)),
        )?;
        let key =
            PhysicalIndexPointKey::selected_blob_catalog(self.store_identity(), object, generation)
                .map_err(PhysicalLayoutMaintenanceFailure::PointKey)?;
        let previous_directory = if let Some(binding) = binding {
            let directory_read = port
                .read_node(binding.directory_record())
                .map_err(PhysicalLayoutMaintenanceFailure::DirectoryRead)?;
            let directory = DerivedFamilyRootDirectoryV1::decode(directory_read.bytes())
                .map_err(PhysicalLayoutMaintenanceFailure::DirectoryFormat)?;
            if directory.indexed_through_blob_publication() != previous_source {
                return Err(PhysicalLayoutMaintenanceFailure::PriorWatermarkMismatch);
            }
            Some(directory)
        } else {
            None
        };
        let mut entries = previous_directory
            .as_ref()
            .map(|directory| directory.entries().to_vec())
            .unwrap_or_default();
        let old_root = entries
            .iter()
            .find(|entry| entry.family() == DurableArtifactFamilyId::BlobCatalog)
            .map(|entry| entry.root_record());
        let inserted_catalog = insert_registered_node(
            self,
            &port,
            DurableArtifactFamilyId::BlobCatalog,
            InsertionSource::Root(old_root),
            key.canonical_bytes().to_vec(),
            encode_record(source.record()),
            placement,
            deadline,
        )?;
        let catalog_root = inserted_catalog.root();
        let new_entry =
            DerivedFamilyRootEntry::new(DurableArtifactFamilyId::BlobCatalog, catalog_root)
                .ok_or(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?;
        if let Some(existing) = entries
            .iter_mut()
            .find(|entry| entry.family() == DurableArtifactFamilyId::BlobCatalog)
        {
            *existing = new_entry;
        } else {
            entries.insert(0, new_entry);
        }
        let mut dedupe_root = entries
            .iter()
            .find(|entry| entry.family() == DurableArtifactFamilyId::DedupeIndex)
            .map(|entry| entry.root_record());
        if previous_source.is_some() && dedupe_root.is_none() {
            return Err(PhysicalLayoutMaintenanceFailure::DedupeRequiresRebuild);
        }
        let mut dedupe_chain: Option<InsertedLayoutTree<'_>> = None;
        traverse_publication_direct(self, port.reader(), selected, |chunk| {
            if !chunk.original_occurrence {
                return Ok(());
            }
            let key = DedupeIndexKey::from_digest(publication.key_scope(), chunk.digest);
            let existing = point_temporary(
                self,
                DurableArtifactFamilyId::DedupeIndex,
                dedupe_root,
                &key.bytes(),
            )?;
            if let Some(existing) = &existing {
                let locator = DedupeIndexValue::decode(existing)
                    .ok_or(LayoutRebuildFailure::InvalidDerivedLocator)?;
                if !source_publication_unrouted(port.reader(), locator)
                    .map_err(LayoutRebuildFailure::ReuseAuthority)?
                {
                    verify_source(
                        port.reader(),
                        locator,
                        publication.key_scope(),
                        chunk.digest,
                        chunk.bytes,
                        publication.chunk_size(),
                    )
                    .map_err(LayoutRebuildFailure::ReuseAuthority)?;
                    return Ok(());
                }
            }
            // Absent, or stale: a retained cell whose source the selected
            // root no longer routes is replaced by this publication's locator.
            let value = DedupeIndexValue::new(source.record(), chunk.ordinal, chunk.record);
            let inserted = insert_cell(
                self,
                DurableArtifactFamilyId::DedupeIndex,
                match dedupe_chain.take() {
                    Some(chain) => InsertionSource::Continue(chain),
                    None => InsertionSource::Root(dedupe_root),
                },
                LayoutCellWrite {
                    key: key.bytes().to_vec(),
                    value: value.encode().to_vec(),
                    superseded: existing,
                },
                placement,
                deadline,
            )?;
            dedupe_root = Some(inserted.root());
            dedupe_chain = Some(inserted);
            Ok(())
        })
        .map_err(|failure| PhysicalLayoutMaintenanceFailure::FullAuthority(Box::new(failure)))?;
        if let Some(root) = dedupe_root {
            let entry = DerivedFamilyRootEntry::new(DurableArtifactFamilyId::DedupeIndex, root)
                .ok_or(PhysicalLayoutMaintenanceFailure::TreeTopologyDamaged)?;
            if let Some(existing) = entries
                .iter_mut()
                .find(|entry| entry.family() == DurableArtifactFamilyId::DedupeIndex)
            {
                *existing = entry;
            } else {
                entries.push(entry);
            }
        }
        let directory = DerivedFamilyRootDirectoryV1::new(entries)
            .map_err(PhysicalLayoutMaintenanceFailure::DirectoryFormat)?
            .with_indexed_through(source)
            .with_indexed_through_quarantine(port.reader().selected_latest_blob_quarantine());
        let mut chains = vec![inserted_catalog];
        if let Some(dedupe_chain) = dedupe_chain {
            chains.push(dedupe_chain);
        }
        let fresh = self
            .records()
            .map_err(PhysicalLayoutMaintenanceFailure::RootProtection)?;
        let fresh_port = PhysicalLayoutPagePort::from_protected_reader(
            self,
            fresh,
            self.maximum_inline_record_bytes(),
        )
        .map_err(PhysicalLayoutMaintenanceFailure::SourceRead)?;
        let retirement = admit_directory_retirement(
            self,
            &fresh_port,
            placement,
            deadline,
            previous_directory.as_ref(),
            &directory,
            chains,
            Vec::new(),
            Vec::new(),
        )?;
        let directory_record =
            publish_derived_directory(self, directory, binding, retirement, placement, deadline)
                .map_err(PhysicalLayoutMaintenanceFailure::Append)?;
        Ok(PhysicalLayoutMaintenanceReceipt {
            source,
            catalog_root,
            dedupe_root,
            directory_record,
        })
    }
}

fn encode_record(record: PersistedRecordIdentity) -> Vec<u8> {
    let mut bytes = vec![0; 24];
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    bytes
}
