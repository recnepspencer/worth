pub use {std::collections::BTreeSet};
pub use {std::path::Path};

pub use {worth_store_physical_format::durable_artifact_checksum, worth_store_physical_format::DurableFreeSpaceManifestHeader, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::FreeSpaceBlockReference, worth_store_physical_format::ManifestBlockReference, worth_store_physical_format::PhysicalFreeSpaceMembershipBlock, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::PhysicalSegmentMembershipBlock, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::SegmentManifestBlockReference, worth_store_physical_format::BOOTSTRAP_CATALOG_BYTES, worth_store_physical_format::ROOT_SELECTOR_BYTES};

#[derive(Clone, Copy)]
pub enum Reference {
    Record(ManifestBlockReference),
    Segment(SegmentManifestBlockReference),
    Free(FreeSpaceBlockReference),
}

impl Reference {
    pub fn generation(self) -> u64 {
        match self {
            Self::Record(value) => value.generation(),
            Self::Segment(value) => value.generation(),
            Self::Free(value) => value.generation(),
        }
    }

    pub fn key(self) -> (u8, u64) {
        match self {
            Self::Record(value) => (0, value.block()),
            Self::Segment(value) => (1, value.block()),
            Self::Free(value) => (2, value.block()),
        }
    }
}

/// Uses the persisted bytes and format decoders, not Store's retained-byte
/// estimator or charged counter. A missing/extra emitted block fails the set
/// comparison even when the total byte count happens to match.
pub fn actual_publication_metadata(
    store: &Path,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
) -> u64 {
    actual_publication_metadata_checked(store, generation, format, Some(2))
}

pub fn actual_publication_metadata_from_media(
    store: &Path,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
) -> u64 {
    actual_publication_metadata_checked(store, generation, format, None)
}

pub fn actual_publication_metadata_checked(
    store: &Path,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    expected_capacity: Option<u16>,
) -> u64 {
    let roots = store.join("families/records/roots");
    let segments = store.join("families/records/segment-manifests");
    let free = store.join("families/records/free-space");
    let root_bytes =
        std::fs::read(roots.join(worth_store_physical_format::RecordArtifactFile::RootManifest { generation }.file_name()))
            .unwrap();
    let (root, found_format) =
        worth_store_physical_format::DurablePhysicalRootManifest::decode(&root_bytes, expected_capacity.unwrap_or(u16::MAX))
            .unwrap();
    assert_eq!(found_format, format);
    assert_eq!(root.generation(), generation);
    if let Some(expected) = expected_capacity {
        assert_eq!(root.node_capacity(), expected);
    }
    let capacity = root.node_capacity();
    let free_bytes =
        std::fs::read(free.join(worth_store_physical_format::RecordArtifactFile::FreeSpaceManifest { generation }.file_name()))
            .unwrap();
    let (header, found_format) =
        worth_store_physical_format::DurableFreeSpaceManifestHeader::decode(&free_bytes, capacity).unwrap();
    assert_eq!(found_format, format);
    assert_eq!(header.generation(), generation);
    assert_eq!(header.root(), root.free_space_root());
    let mut pending = Vec::new();
    pending.extend(root.routing_root().map(crate::publication_metadata_oracle::Reference::Record));
    pending.extend(root.segment_root().map(crate::publication_metadata_oracle::Reference::Segment));
    pending.extend(header.root().map(crate::publication_metadata_oracle::Reference::Free));
    let mut reached = std::collections::BTreeSet::new();
    let mut total = (root_bytes.len() + free_bytes.len()) as u64
        + (2 * ROOT_SELECTOR_BYTES + BOOTSTRAP_CATALOG_BYTES) as u64;
    while let Some(reference) = pending.pop() {
        assert!(reference.generation() <= generation);
        if reference.generation() < generation {
            continue;
        }
        assert!(reached.insert(reference.key()), "duplicate new block");
        let (path, children) = match reference {
            crate::publication_metadata_oracle::Reference::Record(value) => {
                let path = roots.join(
                    worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                        generation,
                        block: value.block(),
                    }
                    .file_name(),
                );
                let bytes = std::fs::read(&path).unwrap();
                let (block, found_format) =
                    worth_store_physical_format::PhysicalRootRoutingBlock::decode(&bytes, capacity).unwrap();
                assert_eq!(found_format, format);
                assert_eq!(block.reference(durable_artifact_checksum(&bytes)), value);
                pending.extend(
                    block
                        .children()
                        .unwrap_or(&[])
                        .iter()
                        .copied()
                        .map(crate::publication_metadata_oracle::Reference::Record),
                );
                (path, bytes.len())
            }
            crate::publication_metadata_oracle::Reference::Segment(value) => {
                let path = segments.join(
                    worth_store_physical_format::RecordArtifactFile::SegmentMembershipBlock {
                        generation,
                        block: value.block(),
                    }
                    .file_name(),
                );
                let bytes = std::fs::read(&path).unwrap();
                let (block, found_format) =
                    worth_store_physical_format::PhysicalSegmentMembershipBlock::decode(&bytes, capacity).unwrap();
                assert_eq!(found_format, format);
                assert_eq!(block.reference(durable_artifact_checksum(&bytes)), value);
                pending.extend(
                    block
                        .children()
                        .unwrap_or(&[])
                        .iter()
                        .copied()
                        .map(crate::publication_metadata_oracle::Reference::Segment),
                );
                (path, bytes.len())
            }
            crate::publication_metadata_oracle::Reference::Free(value) => {
                let path = free.join(
                    worth_store_physical_format::RecordArtifactFile::FreeSpaceMembershipBlock {
                        generation,
                        block: value.block(),
                    }
                    .file_name(),
                );
                let bytes = std::fs::read(&path).unwrap();
                let (block, found_format) =
                    worth_store_physical_format::PhysicalFreeSpaceMembershipBlock::decode(&bytes, capacity).unwrap();
                assert_eq!(found_format, format);
                assert_eq!(block.reference(durable_artifact_checksum(&bytes)), value);
                pending.extend(
                    block
                        .children()
                        .unwrap_or(&[])
                        .iter()
                        .copied()
                        .map(crate::publication_metadata_oracle::Reference::Free),
                );
                (path, bytes.len())
            }
        };
        assert!(path.exists());
        total += children as u64;
    }
    assert_eq!(
        reached,
        emitted_block_keys(&roots, &segments, &free, generation)
    );
    total
}

pub fn emitted_block_keys(
    roots: &Path,
    segments: &Path,
    free: &Path,
    generation: u64,
) -> BTreeSet<(u8, u64)> {
    let mut found = std::collections::BTreeSet::new();
    for (family, directory, prefix) in [
        (0, roots, format!("root-{generation:016x}-block-")),
        (1, segments, format!("segments-{generation:016x}-block-")),
        (2, free, format!("free-space-{generation:016x}-block-")),
    ] {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            let Some(rest) = name.strip_prefix(&prefix) else {
                continue;
            };
            let block = u64::from_str_radix(rest.strip_suffix(".manifest").unwrap(), 16).unwrap();
            assert!(found.insert((family, block)));
        }
    }
    found
}

pub fn wal_file_bytes(store: &Path) -> u64 {
    std::fs::read_dir(store.join("families/wal"))
        .unwrap()
        .map(|entry| entry.unwrap().metadata().unwrap().len())
        .sum()
}