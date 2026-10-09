use std::path::Path;

use worth_store_offline_verifier::OfflineDurableManifestWalk;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, FreeSpaceBlockReference, PhysicalFreeSpaceMembershipBlock,
    PhysicalRecordFormatDeclaration, RecordFreeSpaceManifestEntry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IndependentExtentRoute {
    pub(crate) arena: u64,
    pub(crate) offset: u64,
    pub(crate) length: u64,
    pub(crate) extent: u64,
    pub(crate) generation: u64,
}

/// Follows persisted root references and reads literal C11 leaf fields, without
/// the runtime placement decoder or an extent-id-to-file derivation.
pub(crate) fn current_extent_route(
    root: &Path,
    record: worth_store_physical_format::PersistedRecordIdentity,
) -> IndependentExtentRoute {
    let u64_at = |bytes: &[u8], offset: usize| {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
    };
    let catalog = std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap();
    let generation = u64_at(&catalog, 64);
    let manifest = std::fs::read(root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    )))
    .unwrap();
    assert_eq!(manifest.len(), 384);
    assert_eq!(manifest[88], 1);
    let mut pending = vec![manifest[96..168].to_vec()];
    let mut found = None;
    while let Some(reference) = pending.pop() {
        let generation = u64_at(&reference, 0);
        let block = u64_at(&reference, 8);
        let bytes = std::fs::read(root.join(format!(
            "families/records/roots/root-{generation:016x}-block-{block:016x}.manifest"
        )))
        .unwrap();
        let expected = u32::from_le_bytes(reference[20..24].try_into().unwrap());
        assert_eq!(
            super::durable_frame_oracle::independent_crc32c(&[&bytes]),
            expected
        );
        assert_eq!(&bytes[10..12], &2_u16.to_le_bytes());
        let count = u16::from_le_bytes(bytes[66..68].try_into().unwrap()) as usize;
        match bytes[68] {
            1 => {
                assert_eq!(bytes.len(), 88 + count * 88);
                for entry in bytes[88..].chunks_exact(88) {
                    if entry[..16] != record.allocation_epoch()
                        || u64_at(entry, 16) != record.ordinal()
                    {
                        continue;
                    }
                    assert_eq!(entry[24], 2);
                    assert!(found.is_none(), "record has exactly one current route");
                    found = Some(IndependentExtentRoute {
                        arena: u64_at(entry, 32),
                        offset: u64_at(entry, 56),
                        length: u64_at(entry, 64),
                        extent: u64_at(entry, 40),
                        generation: u64_at(entry, 48),
                    });
                }
            }
            2 => {
                assert_eq!(bytes.len(), 88 + count * 72);
                pending.extend(bytes[88..].chunks_exact(72).map(<[u8]>::to_vec));
            }
            _ => panic!("unknown root node kind"),
        }
    }
    found.expect("record has a current extent arena route")
}

pub(super) struct DecodedFreeSpaceTree {
    pub(super) header: DurableFreeSpaceManifestHeader,
    pub(super) entries: Vec<RecordFreeSpaceManifestEntry>,
}

pub(super) fn decode_free_space_tree(
    store_root: &Path,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    capacity: u16,
) -> DecodedFreeSpaceTree {
    let bytes = std::fs::read(store_root.join(format!(
        "families/records/free-space/free-space-{generation:016x}.manifest"
    )))
    .unwrap();
    let (header, found_format) = DurableFreeSpaceManifestHeader::decode(&bytes, capacity).unwrap();
    assert_eq!(found_format, format);
    let mut entries = Vec::new();
    if let Some(reference) = header.root() {
        walk_free_space_block(store_root, reference, format, capacity, &mut entries);
    }
    assert_eq!(entries.len() as u64, header.entry_count());
    DecodedFreeSpaceTree { header, entries }
}

fn walk_free_space_block(
    store_root: &Path,
    reference: FreeSpaceBlockReference,
    format: PhysicalRecordFormatDeclaration,
    capacity: u16,
    entries: &mut Vec<RecordFreeSpaceManifestEntry>,
) {
    let bytes = std::fs::read(store_root.join(format!(
        "families/records/free-space/free-space-{:016x}-block-{:016x}.manifest",
        reference.generation(),
        reference.block(),
    )))
    .unwrap();
    let (block, found_format) = PhysicalFreeSpaceMembershipBlock::decode(&bytes, capacity).unwrap();
    assert_eq!(found_format, format);
    assert_eq!(
        block.reference(worth_store_physical_format::durable_artifact_checksum(
            &bytes
        )),
        reference
    );
    if let Some(found) = block.entries() {
        entries.extend_from_slice(found);
    } else {
        for child in block.children().unwrap() {
            walk_free_space_block(store_root, *child, format, capacity, entries);
        }
    }
}

pub(super) fn decode_routing_tree(
    store_root: &Path,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    capacity: u16,
) -> OfflineDurableManifestWalk {
    let observation =
        worth_store_offline_verifier::walk_current_durable_record_manifest(store_root, format)
            .unwrap();
    assert_eq!(observation.root_generation(), generation);
    assert_eq!(observation.node_capacity(), capacity);
    observation
}
