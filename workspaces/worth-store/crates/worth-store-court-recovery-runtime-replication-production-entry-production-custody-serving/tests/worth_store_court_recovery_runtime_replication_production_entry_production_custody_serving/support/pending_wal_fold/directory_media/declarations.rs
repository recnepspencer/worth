//! A released V3 edge's replacement directory frame is reread from selected
//! media by Store. A structurally valid substitute frame keeps every C.8/C.9
//! proof intact, so only Store's independent media rejoin can deny it.

pub(crate) use {std::fs, std::io::Read, std::io::Seek, std::io::SeekFrom, std::io::Write, std::path::Path};

pub(crate) use {worth_store::physical_runtime::PhysicalRecoverySelectedRejoinMismatch, worth_store::physical_runtime::RecoveredPhysicalRuntimeConstructionDenial};
pub(crate) use {worth_store_physical_format::decode_extent_chunk, worth_store_physical_format::durable_artifact_checksum, worth_store_physical_format::encode_extent_chunk, worth_store_physical_format::CurrentPhysicalRecordPlacement, worth_store_physical_format::DerivedFamilyRootDirectoryV1, worth_store_physical_format::DerivedFamilyRootEntry, worth_store_physical_format::DurableExtentManifest, worth_store_physical_format::DurablePhysicalRootManifest, worth_store_physical_format::ExtentArenaFrameLayout, worth_store_physical_format::ExtentChunkCoordinate, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalRootRoutingBlock, worth_store_physical_format::RecordArtifactFile, worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES, worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES, worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES};
pub(crate) use {worth_store_recovery_runtime::PhysicalRecoveryOutcome, worth_store_recovery_runtime::WorthStoreRecovery};

pub(crate) fn root_manifests(root: &Path) -> Vec<DurablePhysicalRootManifest> {
    let roots = root.join("families/records/roots");
    let mut found = std::fs::read_dir(&roots)
        .unwrap()
        .filter_map(|entry| {
            let bytes = std::fs::read(entry.unwrap().path()).ok()?;
            Some(
                worth_store_physical_format::DurablePhysicalRootManifest::decode(&bytes, u16::MAX)
                    .ok()?
                    .0,
            )
        })
        .collect::<Vec<_>>();
    found.sort_unstable_by_key(worth_store_physical_format::DurablePhysicalRootManifest::generation);
    found
}

pub(crate) fn routes(
    root: &Path,
    manifest: &DurablePhysicalRootManifest,
) -> Vec<CurrentPhysicalRecordPlacement> {
    let roots = root.join("families/records/roots");
    let mut pending = manifest.routing_root().into_iter().collect::<Vec<_>>();
    let mut routes = Vec::new();
    while let Some(reference) = pending.pop() {
        let bytes = std::fs::read(
            roots.join(
                worth_store_physical_format::RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                }
                .file_name(),
            ),
        )
        .unwrap();
        let (block, _) =
            worth_store_physical_format::PhysicalRootRoutingBlock::decode(&bytes, manifest.node_capacity()).unwrap();
        assert_eq!(
            block.reference(durable_artifact_checksum(&bytes)),
            reference
        );
        match block {
            worth_store_physical_format::PhysicalRootRoutingBlock::Leaf { entries, .. } => routes.extend(entries),
            worth_store_physical_format::PhysicalRootRoutingBlock::Branch { children, .. } => pending.extend(children),
        }
    }
    routes
}

/// Rewrites the directory's single extent chunk with a same-length, validly
/// framed directory naming a different family root record.
pub(crate) fn substitute_directory(root: &Path, route: CurrentPhysicalRecordPlacement) {
    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) = route else {
        panic!("replacement directory must have an extent route")
    };
    let path = root.join("families/records/arenas").join(
        worth_store_physical_format::RecordArtifactFile::ExtentArena {
            arena: extent.arena_range().arena().get(),
        }
        .file_name(),
    );
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    file.seek(std::io::SeekFrom::Start(extent.arena_range().offset()))
        .unwrap();
    let mut manifest_bytes = [0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_bytes).unwrap();
    let (manifest, format) = worth_store_physical_format::DurableExtentManifest::decode(&manifest_bytes).unwrap();
    assert_eq!(manifest.record(), extent.record());
    assert_eq!(
        manifest.chunk_count(),
        1,
        "a directory is one bounded chunk"
    );
    let layout = worth_store_physical_format::ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    let coordinate = worth_store_physical_format::ExtentChunkCoordinate::new(
        extent.record(),
        extent.extent_cell(),
        manifest.logical_bytes(),
        0,
        1,
    )
    .unwrap();
    let chunk_offset = extent.arena_range().offset() + layout.chunk_offset(1).unwrap();
    let mut original = vec![
        0;
        DURABLE_EXTENT_FRAME_HEADER_BYTES
            + EXTENT_CHUNK_METADATA_BYTES
            + manifest.logical_bytes() as usize
    ];
    file.seek(std::io::SeekFrom::Start(chunk_offset)).unwrap();
    file.read_exact(&mut original).unwrap();
    let (payload, _) = decode_extent_chunk(&original, coordinate).unwrap();
    let directory = worth_store_physical_format::DerivedFamilyRootDirectoryV1::decode(payload).unwrap();
    assert!(directory.indexed_through_blob_publication().is_none());
    let mut entries = directory.entries().to_vec();
    let first = entries.first_mut().expect("replacement keeps family roots");
    let mut epoch = first.root_record().allocation_epoch();
    epoch[0] ^= 0x80;
    *first = worth_store_physical_format::DerivedFamilyRootEntry::new(
        first.family(),
        worth_store_physical_format::PersistedRecordIdentity::new(epoch, first.root_record().ordinal()).unwrap(),
    )
    .unwrap();
    let changed = worth_store_physical_format::DerivedFamilyRootDirectoryV1::new(entries)
        .unwrap()
        .with_indexed_through_quarantine(directory.indexed_through_quarantine())
        .encode();
    assert_eq!(changed.len(), payload.len());
    assert_ne!(changed.as_slice(), payload);
    let replacement = encode_extent_chunk(format, coordinate, &changed).unwrap();
    assert_eq!(replacement.len(), original.len());
    assert_eq!(
        decode_extent_chunk(&replacement, coordinate).unwrap().0,
        changed.as_slice()
    );
    file.seek(std::io::SeekFrom::Start(chunk_offset)).unwrap();
    file.write_all(&replacement).unwrap();
    file.sync_all().unwrap();
}