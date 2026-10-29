pub(crate) use {std::collections::BTreeSet, std::collections::VecDeque};
pub(crate) use {std::path::Path, std::path::PathBuf};

pub(crate) use {worth_store_physical_format::durable_artifact_checksum, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DurableFreeSpaceManifestHeader, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::DurableRootSelector, worth_store_physical_format::PhysicalFreeSpaceMembershipBlock, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::PhysicalSegmentMembershipBlock, worth_store_physical_format::RecordArtifactFile};

pub(crate) use {crate::artifact_snapshot::process_context::collect_file, crate::artifact_snapshot::process_context::record_artifact_path};

pub(crate) fn capture(root: &Path, paths: &mut BTreeSet<PathBuf>) {
    collect_file(
        &record_artifact_path(root, worth_store_physical_format::RecordArtifactFile::BootstrapCatalog),
        paths,
    );
    let current = read_selector(root, worth_store_physical_format::RecordArtifactFile::CurrentRootSelector, paths);
    collect_selected_root(root, current, paths);
    if record_artifact_path(root, worth_store_physical_format::RecordArtifactFile::PreviousRootSelector).is_file() {
        let previous = read_selector(root, worth_store_physical_format::RecordArtifactFile::PreviousRootSelector, paths);
        collect_selected_root(root, previous, paths);
    }
}

pub(crate) fn read_selector(
    root: &Path,
    artifact: RecordArtifactFile,
    paths: &mut BTreeSet<PathBuf>,
) -> DurableRootSelector {
    let path = record_artifact_path(root, artifact);
    collect_file(&path, paths);
    worth_store_physical_format::DurableRootSelector::decode(&std::fs::read(path).expect("read selected root selector"))
        .expect("decode selected root selector")
}

pub(crate) fn collect_selected_root(
    root: &Path,
    selector: DurableRootSelector,
    paths: &mut BTreeSet<PathBuf>,
) {
    let artifact = worth_store_physical_format::RecordArtifactFile::RootManifest {
        generation: selector.root_generation(),
    };
    let path = record_artifact_path(root, artifact);
    collect_file(&path, paths);
    let (manifest, format) = worth_store_physical_format::DurablePhysicalRootManifest::decode(
        &std::fs::read(path).expect("read selected root manifest"),
        u16::MAX,
    )
    .expect("decode selected root manifest");
    assert_eq!(format, selector.format());
    collect_root_tree(root, &manifest, format, paths);
    collect_segment_tree(root, &manifest, format, paths);
    collect_free_space_tree(root, &manifest, format, paths);
}

pub(crate) fn collect_root_tree(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    paths: &mut BTreeSet<PathBuf>,
) {
    let mut pending = manifest.routing_root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = pending.pop_front() {
        let artifact = worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let path = record_artifact_path(root, artifact);
        collect_file(&path, paths);
        let bytes = std::fs::read(path).expect("read selected root routing block");
        let (block, found_format) =
            worth_store_physical_format::PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode selected root routing block");
        assert_eq!(found_format, format);
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        match block {
            worth_store_physical_format::PhysicalRootRoutingBlock::Leaf { entries, .. } => {
                for placement in entries {
                    collect_placement(root, placement, paths);
                }
            }
            worth_store_physical_format::PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
        }
    }
}

pub(crate) fn collect_placement(
    root: &Path,
    placement: CurrentPhysicalRecordPlacement,
    paths: &mut BTreeSet<PathBuf>,
) {
    match placement {
        worth_store_physical_format::CurrentPhysicalRecordPlacement::Inline(inline) => collect_file(
            &record_artifact_path(
                root,
                worth_store_physical_format::RecordArtifactFile::Segment {
                    segment: inline.segment().get(),
                    generation: inline.segment_generation(),
                },
            ),
            paths,
        ),
        worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) => {
            collect_file(
                &record_artifact_path(
                    root,
                    worth_store_physical_format::RecordArtifactFile::ExtentArena {
                        arena: extent.arena_range().arena().get(),
                    },
                ),
                paths,
            );
        }
    }
}

pub(crate) fn collect_segment_tree(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    paths: &mut BTreeSet<PathBuf>,
) {
    let mut pending = manifest.segment_root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = pending.pop_front() {
        let artifact = worth_store_physical_format::RecordArtifactFile::SegmentMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let path = record_artifact_path(root, artifact);
        collect_file(&path, paths);
        let bytes = std::fs::read(path).expect("read selected segment membership block");
        let (block, found_format) =
            worth_store_physical_format::PhysicalSegmentMembershipBlock::decode(&bytes, manifest.node_capacity())
                .expect("decode selected segment membership block");
        assert_eq!(found_format, format);
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        if let Some(children) = block.children() {
            pending.extend(children.iter().copied());
        }
    }
}

pub(crate) fn collect_free_space_tree(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    paths: &mut BTreeSet<PathBuf>,
) {
    let artifact = worth_store_physical_format::RecordArtifactFile::FreeSpaceManifest {
        generation: manifest.generation(),
    };
    let path = record_artifact_path(root, artifact);
    collect_file(&path, paths);
    let bytes = std::fs::read(path).expect("read selected free-space manifest");
    let (header, found_format) = worth_store_physical_format::DurableFreeSpaceManifestHeader::decode(&bytes, u16::MAX)
        .expect("decode selected free-space manifest");
    assert_eq!(found_format, format);
    assert_eq!(
        durable_artifact_checksum(&bytes),
        manifest.free_space_checksum()
    );
    let mut pending = header.root().into_iter().collect::<VecDeque<_>>();
    while let Some(reference) = pending.pop_front() {
        let artifact = worth_store_physical_format::RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        let path = record_artifact_path(root, artifact);
        collect_file(&path, paths);
        let bytes = std::fs::read(path).expect("read selected free-space membership block");
        let (block, block_format) =
            worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::decode(&bytes, header.node_capacity())
                .expect("decode selected free-space membership block");
        assert_eq!(block_format, format);
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        if let Some(children) = block.children() {
            pending.extend(children.iter().copied());
        }
    }
}