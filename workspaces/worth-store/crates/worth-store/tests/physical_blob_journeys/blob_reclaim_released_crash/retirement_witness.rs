//! Independent persisted-media witness for a released publication's native
//! extent and the root protocol that makes Serving reconstruct displacement.

use std::{fs, path::Path};

use worth_store_physical_format::{
    BlobRecordKind, CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, DurableRootSelector,
    ExtentArenaRange, PersistedRecordIdentity, PhysicalFreeSpaceMembershipBlock,
    PhysicalRecordFormatDeclaration, PhysicalRootRoutingBlock, RecordArtifactFile,
    SelectedRecordContentClass,
};

pub(super) fn selected_root(
    root: &Path,
) -> (DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration) {
    let records = root.join("families/records");
    let selector = DurableRootSelector::decode(
        &fs::read(records.join(RecordArtifactFile::CurrentRootSelector.file_name())).unwrap(),
    )
    .unwrap();
    let bytes = fs::read(
        records.join("roots").join(
            RecordArtifactFile::RootManifest {
                generation: selector.root_generation(),
            }
            .file_name(),
        ),
    )
    .unwrap();
    let (manifest, format) = DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
    assert_eq!(format, selector.format());
    assert_eq!(manifest.generation(), selector.root_generation());
    assert_eq!(manifest.encode(format), bytes);
    (manifest, format)
}

pub(super) fn selected_publication_extent(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
) -> (PersistedRecordIdentity, DurableExtentRecordPlacement) {
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    while let Some(reference) = pending.pop() {
        let path = root.join("families/records/roots").join(
            RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }
            .file_name(),
        );
        let (block, _) =
            PhysicalRootRoutingBlock::decode(&fs::read(path).unwrap(), manifest.node_capacity())
                .unwrap();
        match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                for placement in entries {
                    if placement.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::GenerationPublished)
                    {
                        let CurrentPhysicalRecordPlacement::Extent(extent) = placement else {
                            panic!("released publication must occupy a native extent")
                        };
                        return (placement.record(), extent);
                    }
                }
            }
            PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
        }
    }
    panic!("released publication missing from selected pre-redo root")
}

pub(super) fn selected_placements(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
) -> Vec<CurrentPhysicalRecordPlacement> {
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    let mut placements = Vec::new();
    while let Some(reference) = pending.pop() {
        let path = root.join("families/records/roots").join(
            RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }
            .file_name(),
        );
        let (block, _) =
            PhysicalRootRoutingBlock::decode(&fs::read(path).unwrap(), manifest.node_capacity())
                .unwrap();
        match block {
            PhysicalRootRoutingBlock::Leaf { entries, .. } => placements.extend(entries),
            PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
        }
    }
    placements.sort_unstable_by_key(|placement| placement.record());
    assert_eq!(placements.len(), manifest.record_count() as usize);
    assert!(placements
        .windows(2)
        .all(|pair| pair[0].record() != pair[1].record()));
    placements
}

pub(super) fn range_published_free(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    range: ExtentArenaRange,
    source_generation: u64,
) -> bool {
    let free = root.join("families/records/free-space");
    let (header, _) = DurableFreeSpaceManifestHeader::decode(
        &fs::read(
            free.join(
                RecordArtifactFile::FreeSpaceManifest {
                    generation: manifest.generation(),
                }
                .file_name(),
            ),
        )
        .unwrap(),
        manifest.node_capacity(),
    )
    .unwrap();
    let mut pending = header.root().into_iter().collect::<Vec<_>>();
    while let Some(reference) = pending.pop() {
        let (block, _) = PhysicalFreeSpaceMembershipBlock::decode(
            &fs::read(
                free.join(
                    RecordArtifactFile::FreeSpaceMembershipBlock {
                        generation: reference.generation(),
                        block: reference.block(),
                    }
                    .file_name(),
                ),
            )
            .unwrap(),
            header.node_capacity(),
        )
        .unwrap();
        match block {
            PhysicalFreeSpaceMembershipBlock::Leaf { entries, .. } => {
                if entries.iter().any(|entry| {
                    entry.generation() > source_generation
                        && entry.generation() <= manifest.generation()
                        && entry.arena_free_range().is_some_and(|free| {
                            free.arena() == range.arena()
                                && free.offset() <= range.offset()
                                && free.end() >= range.end()
                        })
                }) {
                    return true;
                }
            }
            PhysicalFreeSpaceMembershipBlock::Branch { children, .. } => pending.extend(children),
        }
    }
    false
}
