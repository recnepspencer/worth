pub(crate) use {std::io::Read, std::io::Seek, std::io::SeekFrom, std::path::Path};

pub(crate) use {
    worth_store::physical_runtime::PhysicalExtentCopyPhase,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRecordId,
};
pub(crate) use {
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::decode_extent_chunk, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::BootstrapCatalog,
    worth_store_physical_format::CurrentPhysicalRecordPlacement,
    worth_store_physical_format::DurableExtentManifest,
    worth_store_physical_format::DurableExtentRecordPlacement,
    worth_store_physical_format::DurablePhysicalRootManifest,
    worth_store_physical_format::ExtentArenaFrameLayout,
    worth_store_physical_format::ExtentChunkCoordinate,
    worth_store_physical_format::PersistedRecordIdentity,
    worth_store_physical_format::PhysicalRootRoutingBlock,
    worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES,
    worth_store_physical_format::EXTENT_ARENA_MANIFEST_FRAME_BYTES,
    worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES,
};

pub(crate) use {
    crate::physical_blob_journeys::blob_abort::unfinished,
    crate::physical_blob_journeys::blob_ingest_process::observe_closed_store_named,
    worth_store_testkit_store::fixture::admitted_blob_scope,
    worth_store_testkit_store::fixture::placement,
    worth_store_testkit_store::fixture::serving_from_initialization,
    worth_store_testkit_store::physical_blob_journeys::blob_frontier::selected_blob_records,
};

pub(crate) fn current_extent_route(
    root: &Path,
    record: PhysicalRecordId,
) -> DurableExtentRecordPlacement {
    let catalog = worth_store_physical_format::BootstrapCatalog::decode(
        &std::fs::read(root.join("families/records/bootstrap.catalog")).unwrap(),
    )
    .unwrap();
    let generation = catalog.current_root().generation().get();
    let manifest = std::fs::read(root.join(format!(
        "families/records/roots/root-{generation:016x}.manifest"
    )))
    .unwrap();
    let (manifest, _) =
        worth_store_physical_format::DurablePhysicalRootManifest::decode(&manifest, 64).unwrap();
    let identity = worth_store_physical_format::PersistedRecordIdentity::new(
        record.allocation_epoch(),
        record.ordinal(),
    )
    .unwrap();
    let mut pending = vec![manifest.routing_root().unwrap()];
    let mut found = None;
    while let Some(reference) = pending.pop() {
        if !reference.contains(identity) {
            continue;
        }
        let block = std::fs::read(root.join(format!(
            "families/records/roots/root-{:016x}-block-{:016x}.manifest",
            reference.generation(),
            reference.block()
        )))
        .unwrap();
        let (block, _) = worth_store_physical_format::PhysicalRootRoutingBlock::decode(
            &block,
            manifest.node_capacity(),
        )
        .unwrap();
        if let Some(entries) = block.entries() {
            for entry in entries {
                if entry.record() == identity {
                    let worth_store_physical_format::CurrentPhysicalRecordPlacement::Extent(extent) =
                        entry
                    else {
                        panic!("blob chunk must have an extent arena route");
                    };
                    assert!(found.replace(*extent).is_none());
                }
            }
        } else {
            pending.extend(block.children().unwrap().iter().copied());
        }
    }
    found.expect("exact selected blob extent route")
}

pub(crate) fn hex_record(record: PhysicalRecordId) -> String {
    record
        .allocation_epoch()
        .into_iter()
        .chain(record.ordinal().to_le_bytes())
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn extent_frame(root: &Path, placement: DurableExtentRecordPlacement) -> Vec<u8> {
    let range = placement.arena_range();
    let path = root.join(format!(
        "families/records/arenas/arena-{:016x}.data",
        range.arena().get()
    ));
    let mut file = std::fs::File::open(path).unwrap();
    file.seek(std::io::SeekFrom::Start(range.offset())).unwrap();
    let mut manifest_frame = vec![0; EXTENT_ARENA_MANIFEST_FRAME_BYTES];
    file.read_exact(&mut manifest_frame).unwrap();
    let (manifest, format) =
        worth_store_physical_format::DurableExtentManifest::decode(&manifest_frame).unwrap();
    assert_eq!(manifest.record(), placement.record());
    assert_eq!(manifest.extent_cell(), placement.extent_cell());
    assert_eq!(manifest.logical_bytes(), placement.payload_bytes());
    let layout =
        worth_store_physical_format::ExtentArenaFrameLayout::new(format, manifest.alignment())
            .unwrap();
    assert!(layout.admits(range, manifest.chunk_count()));
    let mut encoded = manifest_frame;
    let capacity = u64::from(manifest.chunk_payload_capacity());
    for ordinal in 1..=manifest.chunk_count() {
        let logical_offset = u64::from(ordinal - 1) * capacity;
        let logical_length = (manifest.logical_bytes() - logical_offset).min(capacity);
        let frame_length = DURABLE_EXTENT_FRAME_HEADER_BYTES as u64
            + EXTENT_CHUNK_METADATA_BYTES as u64
            + logical_length;
        let frame_offset = range.offset() + layout.chunk_offset(ordinal).unwrap();
        assert!(frame_offset + frame_length <= range.end());
        file.seek(std::io::SeekFrom::Start(frame_offset)).unwrap();
        let mut frame = vec![0; usize::try_from(frame_length).unwrap()];
        file.read_exact(&mut frame).unwrap();
        let coordinate = worth_store_physical_format::ExtentChunkCoordinate::new(
            placement.record(),
            placement.extent_cell(),
            manifest.logical_bytes(),
            logical_offset,
            ordinal,
        )
        .unwrap();
        let (payload, found_format) = decode_extent_chunk(&frame, coordinate).unwrap();
        assert_eq!(found_format, format);
        assert_eq!(payload.len() as u64, logical_length);
        encoded.extend_from_slice(&frame);
    }
    encoded
}
