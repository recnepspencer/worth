//! Independent persisted-media witness for a released publication's native
//! extent and the root protocol that makes Serving reconstruct displacement.

pub(crate) use {std::fs, std::path::Path};

pub(crate) use {
    worth_store_physical_format::BlobRecordKind,
    worth_store_physical_format::CurrentPhysicalRecordPlacement,
    worth_store_physical_format::DurableExtentRecordPlacement,
    worth_store_physical_format::DurableFreeSpaceManifestHeader,
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::DurableRootSelector,
    worth_store_physical_format::ExtentArenaRange,
    worth_store_physical_format::PersistedRecordIdentity,
    worth_store_physical_format::PhysicalFreeSpaceMembershipBlock,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
    worth_store_physical_format::PhysicalRootRoutingBlock,
    worth_store_physical_format::RecordArtifactFile,
    worth_store_physical_format::SelectedRecordContentClass,
};

pub(crate) fn selected_root(
    root: &Path,
) -> (DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration) {
    let records = root.join("families/records");
    let selector = worth_store_physical_format::DurableRootSelector::decode(
        &std::fs::read(records.join(
            worth_store_physical_format::RecordArtifactFile::CurrentRootSelector.file_name(),
        ))
        .unwrap(),
    )
    .unwrap();
    let bytes = std::fs::read(
        records.join("roots").join(
            worth_store_physical_format::RecordArtifactFile::RootManifest {
                generation: selector.root_generation(),
            }
            .file_name(),
        ),
    )
    .unwrap();
    let (manifest, format) =
        worth_store_physical_format::DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
    assert_eq!(format, selector.format());
    assert_eq!(manifest.generation(), selector.root_generation());
    assert_eq!(manifest.encode(format), bytes);
    (manifest, format)
}

pub(crate) fn selected_publication_extent(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
) -> (PersistedRecordIdentity, DurableExtentRecordPlacement) {
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    while let Some(reference) = pending.pop() {
        let path = root.join("families/records/roots").join(
            worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }
            .file_name(),
        );
        let (block, _) = worth_store_physical_format::PhysicalRootRoutingBlock::decode(
            &std::fs::read(path).unwrap(),
            manifest.node_capacity(),
        )
        .unwrap();
        match block {
            worth_store_physical_format::PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                for placement in entries {
                    if placement.content_class()
                        == worth_store_physical_format::SelectedRecordContentClass::Blob(
                            worth_store_physical_format::BlobRecordKind::GenerationPublished,
                        )
                    {
                        let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(
                            extent,
                        ) = placement
                        else {
                            panic!("released publication must occupy a native extent")
                        };
                        return (placement.record(), extent);
                    }
                }
            }
            worth_store_physical_format::PhysicalRootRoutingBlock::Branch { children, .. } => {
                pending.extend(children)
            }
        }
    }
    panic!("released publication missing from selected pre-redo root")
}

pub(crate) fn selected_placements(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
) -> Vec<CurrentPhysicalRecordPlacement> {
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    let mut placements = Vec::new();
    while let Some(reference) = pending.pop() {
        let path = root.join("families/records/roots").join(
            worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                generation: reference.generation(),
                block: reference.block(),
            }
            .file_name(),
        );
        let (block, _) = worth_store_physical_format::PhysicalRootRoutingBlock::decode(
            &std::fs::read(path).unwrap(),
            manifest.node_capacity(),
        )
        .unwrap();
        match block {
            worth_store_physical_format::PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                placements.extend(entries)
            }
            worth_store_physical_format::PhysicalRootRoutingBlock::Branch { children, .. } => {
                pending.extend(children)
            }
        }
    }
    placements.sort_unstable_by_key(|placement| placement.record());
    assert_eq!(placements.len(), manifest.record_count() as usize);
    assert!(placements
        .windows(2)
        .all(|pair| pair[0].record() != pair[1].record()));
    placements
}

pub(crate) fn range_published_free(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    range: ExtentArenaRange,
    source_generation: u64,
) -> bool {
    let free = root.join("families/records/free-space");
    let (header, _) = worth_store_physical_format::DurableFreeSpaceManifestHeader::decode(
        &std::fs::read(
            free.join(
                worth_store_physical_format::RecordArtifactFile::FreeSpaceManifest {
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
        let (block, _) = worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::decode(
            &std::fs::read(
                free.join(
                    worth_store_physical_format::RecordArtifactFile::FreeSpaceMembershipBlock {
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
            worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::Leaf {
                entries, ..
            } => {
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
            worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::Branch {
                children,
                ..
            } => pending.extend(children),
        }
    }
    false
}
