use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_physical_format::{
    DerivedFamilyRootDirectoryV1, DerivedFamilyRootEntry, IndexedThroughBlobPublication,
    PersistedRecordIdentity,
};

use crate::physical_runtime::{
    blob::{verify_source, DedupeIndexKey, DedupeIndexValue},
    layout::{
        admit_directory_retirement, insert_registered_node, inspect_selected_tree_retirement,
        publish_derived_directory, DeferredDerivedRetirementCause, InsertedLayoutTree,
        PhysicalBTreeIndex, PhysicalIndexPointKey, PhysicalLayoutPagePort, SelectedTreeRetirement,
    },
    AdmittedRecordPlacementPolicy, PhysicalMutationDeadline, ServingPhysicalRuntime,
};

use super::{
    basis::{collect_selected_blob_authority, LayoutRebuildFailure, LayoutRebuildLimits},
    traversal::traverse_publication,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutRebuildReceipt {
    source: IndexedThroughBlobPublication,
    catalog_root: PersistedRecordIdentity,
    dedupe_root: Option<PersistedRecordIdentity>,
    directory_record: PersistedRecordIdentity,
    scanned_records: u64,
    validated_chunks: u64,
    deferred_catalog_root: Option<PersistedRecordIdentity>,
    deferred_dedupe_root: Option<PersistedRecordIdentity>,
    deferred_catalog_cause: Option<DeferredDerivedRetirementCause>,
    deferred_dedupe_cause: Option<DeferredDerivedRetirementCause>,
}

impl LayoutRebuildReceipt {
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
    pub const fn scanned_records(self) -> u64 {
        self.scanned_records
    }
    pub const fn validated_chunks(self) -> u64 {
        self.validated_chunks
    }
    /// A damaged old derived catalog root left selected but unreachable. Its
    /// closure was not proved, so no nodes under it were authorized for drop.
    pub const fn deferred_catalog_root(self) -> Option<PersistedRecordIdentity> {
        self.deferred_catalog_root
    }
    /// A damaged old derived dedupe root left selected but unreachable.
    pub const fn deferred_dedupe_root(self) -> Option<PersistedRecordIdentity> {
        self.deferred_dedupe_root
    }
    pub const fn deferred_catalog_cause(self) -> Option<DeferredDerivedRetirementCause> {
        self.deferred_catalog_cause
    }
    pub const fn deferred_dedupe_cause(self) -> Option<DeferredDerivedRetirementCause> {
        self.deferred_dedupe_cause
    }
    pub const fn deferred_derived_root_count(self) -> u8 {
        self.deferred_catalog_root.is_some() as u8 + self.deferred_dedupe_root.is_some() as u8
    }
}

/// Rebuilds both derived blob families from one bounded selected C.5 snapshot.
/// Partial COW node appends remain unreachable if any authority or mutation
/// check fails; only the final classified directory append publishes roots.
pub(in crate::physical_runtime) fn rebuild_blob_derived_indexes(
    runtime: &ServingPhysicalRuntime,
    limits: LayoutRebuildLimits,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<LayoutRebuildReceipt, LayoutRebuildFailure> {
    let authority = collect_selected_blob_authority(runtime, limits)?;
    let source = authority
        .latest
        .ok_or(LayoutRebuildFailure::NoSelectedPublication)?;
    // Do not write even unreachable derived nodes until every selected
    // publication's source tree, claims and content have passed closure.
    let mut validated_chunks = 0_u64;
    for selected in &authority.publications {
        validated_chunks = validated_chunks
            .checked_add(traverse_publication(
                runtime,
                &authority,
                *selected,
                |_| Ok(()),
            )?)
            .ok_or(LayoutRebuildFailure::TraversalBoundExhausted)?;
    }
    let mut catalog_root = None;
    let mut dedupe_root = None;
    let mut catalog_chain: Option<InsertedLayoutTree> = None;
    let mut dedupe_chain: Option<InsertedLayoutTree> = None;
    for selected in &authority.publications {
        let publication = selected.publication;
        let key = PhysicalIndexPointKey::selected_blob_catalog(
            runtime.store_identity(),
            publication.object(),
            publication.generation(),
        )
        .map_err(LayoutRebuildFailure::InvalidCatalogKey)?;
        let inserted_catalog = insert_cell(
            runtime,
            DurableArtifactFamilyId::BlobCatalog,
            catalog_root,
            key.canonical_bytes().to_vec(),
            encode_record(selected.record).to_vec(),
            placement,
            deadline,
        )?;
        catalog_root = Some(inserted_catalog.root());
        catalog_chain = Some(match catalog_chain.take() {
            Some(chain) => chain
                .chain(inserted_catalog)
                .map_err(LayoutRebuildFailure::LayoutMutation)?,
            None => inserted_catalog,
        });
        traverse_publication(runtime, &authority, *selected, |chunk| {
            if !chunk.original_occurrence {
                return Ok(());
            }
            let key = DedupeIndexKey::from_digest(publication.key_scope(), chunk.digest);
            if authority.quarantined_keys.contains(&key.bytes()) {
                return Ok(());
            }
            let value = DedupeIndexValue::new(selected.record, chunk.ordinal, chunk.record);
            if let Some(existing) = point_temporary(
                runtime,
                DurableArtifactFamilyId::DedupeIndex,
                dedupe_root,
                &key.bytes(),
            )? {
                let locator = DedupeIndexValue::decode(&existing)
                    .ok_or(LayoutRebuildFailure::InvalidDerivedLocator)?;
                verify_source(
                    &authority.reader,
                    locator,
                    publication.key_scope(),
                    chunk.digest,
                    chunk.bytes,
                    publication.chunk_size(),
                )
                .map_err(LayoutRebuildFailure::ReuseAuthority)?;
            } else {
                let inserted_dedupe = insert_cell(
                    runtime,
                    DurableArtifactFamilyId::DedupeIndex,
                    dedupe_root,
                    key.bytes().to_vec(),
                    value.encode().to_vec(),
                    placement,
                    deadline,
                )?;
                dedupe_root = Some(inserted_dedupe.root());
                dedupe_chain = Some(match dedupe_chain.take() {
                    Some(chain) => chain
                        .chain(inserted_dedupe)
                        .map_err(LayoutRebuildFailure::LayoutMutation)?,
                    None => inserted_dedupe,
                });
            }
            Ok(())
        })?;
    }
    let catalog_root = catalog_root.ok_or(LayoutRebuildFailure::NoSelectedPublication)?;
    let current = runtime
        .records()
        .map_err(LayoutRebuildFailure::RootProtection)?;
    if current.selected_latest_blob_publication() != Some(source) {
        return Err(LayoutRebuildFailure::ConcurrentAuthorityAdvance);
    }
    if current.selected_latest_blob_quarantine()
        != authority.reader.selected_latest_blob_quarantine()
    {
        return Err(LayoutRebuildFailure::ConcurrentAuthorityAdvance);
    }
    let selected_directory = current.selected_derived_family_directory();
    let selected_quarantine = current.selected_latest_blob_quarantine();
    let port = PhysicalLayoutPagePort::from_protected_reader(
        runtime,
        current,
        runtime.maximum_inline_record_bytes(),
    )
    .map_err(LayoutRebuildFailure::LayoutRead)?;
    let previous_directory = if let Some(binding) = selected_directory {
        let frame = port
            .read_node(binding.directory_record())
            .map_err(LayoutRebuildFailure::LayoutRead)?;
        Some(
            DerivedFamilyRootDirectoryV1::decode(frame.bytes())
                .map_err(LayoutRebuildFailure::DirectoryFormat)?,
        )
    } else {
        None
    };
    let mut entries =
        vec![
            DerivedFamilyRootEntry::new(DurableArtifactFamilyId::BlobCatalog, catalog_root)
                .ok_or(LayoutRebuildFailure::TreeDamaged)?,
        ];
    if let Some(root) = dedupe_root {
        entries.push(
            DerivedFamilyRootEntry::new(DurableArtifactFamilyId::DedupeIndex, root)
                .ok_or(LayoutRebuildFailure::TreeDamaged)?,
        );
    }
    let directory = DerivedFamilyRootDirectoryV1::new(entries)
        .map_err(LayoutRebuildFailure::DirectoryFormat)?
        .with_indexed_through(source)
        .with_indexed_through_quarantine(selected_quarantine);
    let mut retired_selected = Vec::new();
    let mut deferred_selected = Vec::new();
    let mut deferred_catalog_root = None;
    let mut deferred_dedupe_root = None;
    let mut deferred_catalog_cause = None;
    let mut deferred_dedupe_cause = None;
    if let Some(previous) = &previous_directory {
        for old in previous.entries() {
            if old.family() == DurableArtifactFamilyId::BlobCatalog
                || old.family() == DurableArtifactFamilyId::DedupeIndex
            {
                match inspect_selected_tree_retirement(
                    runtime,
                    &port,
                    old.family(),
                    old.root_record(),
                    placement,
                    deadline,
                )
                .map_err(LayoutRebuildFailure::LayoutMutation)?
                {
                    SelectedTreeRetirement::Proven(proven) => retired_selected.push(proven),
                    SelectedTreeRetirement::Deferred(deferred) => {
                        match deferred.family() {
                            DurableArtifactFamilyId::BlobCatalog => {
                                deferred_catalog_root = Some(deferred.root());
                                deferred_catalog_cause = Some(deferred.cause());
                            }
                            DurableArtifactFamilyId::DedupeIndex => {
                                deferred_dedupe_root = Some(deferred.root());
                                deferred_dedupe_cause = Some(deferred.cause());
                            }
                            _ => return Err(LayoutRebuildFailure::TreeDamaged),
                        }
                        deferred_selected.push(deferred);
                    }
                }
            }
        }
    }
    let mut chains = Vec::new();
    if let Some(chain) = catalog_chain {
        chains.push(chain);
    }
    if let Some(chain) = dedupe_chain {
        chains.push(chain);
    }
    let retirement = admit_directory_retirement(
        runtime,
        &port,
        placement,
        deadline,
        previous_directory.as_ref(),
        &directory,
        chains,
        retired_selected,
        deferred_selected,
    )
    .map_err(LayoutRebuildFailure::LayoutMutation)?;
    let directory_record = publish_derived_directory(
        runtime,
        directory,
        selected_directory,
        retirement,
        placement,
        deadline,
    )
    .map_err(LayoutRebuildFailure::DirectoryAppend)?;
    Ok(LayoutRebuildReceipt {
        source,
        catalog_root,
        dedupe_root,
        directory_record,
        scanned_records: authority.scanned_records,
        validated_chunks,
        deferred_catalog_root,
        deferred_dedupe_root,
        deferred_catalog_cause,
        deferred_dedupe_cause,
    })
}

pub(in crate::physical_runtime) fn insert_cell(
    runtime: &ServingPhysicalRuntime,
    family: DurableArtifactFamilyId,
    root: Option<PersistedRecordIdentity>,
    key: Vec<u8>,
    value: Vec<u8>,
    placement: AdmittedRecordPlacementPolicy,
    deadline: PhysicalMutationDeadline,
) -> Result<InsertedLayoutTree, LayoutRebuildFailure> {
    let reader = runtime
        .records()
        .map_err(LayoutRebuildFailure::RootProtection)?;
    let port = PhysicalLayoutPagePort::from_protected_reader(
        runtime,
        reader,
        runtime.maximum_inline_record_bytes(),
    )
    .map_err(LayoutRebuildFailure::LayoutRead)?;
    let inserted = insert_registered_node(
        runtime, &port, family, root, key, value, placement, deadline,
    )
    .map_err(LayoutRebuildFailure::LayoutMutation)?;
    Ok(inserted)
}

pub(in crate::physical_runtime) fn point_temporary(
    runtime: &ServingPhysicalRuntime,
    family: DurableArtifactFamilyId,
    root: Option<PersistedRecordIdentity>,
    key: &[u8],
) -> Result<Option<Vec<u8>>, LayoutRebuildFailure> {
    if root.is_none() {
        return Ok(None);
    }
    let reader = runtime
        .records()
        .map_err(LayoutRebuildFailure::RootProtection)?;
    let port = PhysicalLayoutPagePort::from_protected_reader(
        runtime,
        reader,
        runtime.maximum_inline_record_bytes(),
    )
    .map_err(LayoutRebuildFailure::LayoutRead)?;
    let registered = runtime
        .registered_btree_family(family)
        .map_err(|_| LayoutRebuildFailure::InvalidDerivedLocator)?;
    PhysicalBTreeIndex::from_selected_root(&port, registered, root)
        .point_raw(key)
        .map(|(value, _)| value)
        .map_err(LayoutRebuildFailure::LayoutPoint)
}

fn encode_record(record: PersistedRecordIdentity) -> [u8; 24] {
    let mut bytes = [0_u8; 24];
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..].copy_from_slice(&record.ordinal().to_le_bytes());
    bytes
}
