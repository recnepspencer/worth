use super::*;

pub(super) fn extent_payload(root: &Path, placement: DurableExtentRecordPlacement) -> Vec<u8> {
    let range = placement.arena_range();
    let bytes = std::fs::read(arena_path(root, range)).expect("selected extent arena");
    let range_start = range.offset() as usize;
    let (manifest, format) = DurableExtentManifest::decode(
        &bytes[range_start..range_start + EXTENT_ARENA_MANIFEST_FRAME_BYTES],
    )
    .unwrap();
    assert_eq!(manifest.record(), placement.record());
    assert_eq!(manifest.generation(), placement.extent_generation());
    assert_eq!(manifest.logical_bytes(), placement.payload_bytes());
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    assert!(layout.admits(range, manifest.chunk_count()));
    let overhead = DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES;
    let capacity = PAGE_BYTES - overhead;
    let logical = placement.payload_bytes();
    let mut payload = Vec::new();
    let mut ordinal = 1;
    while (payload.len() as u64) < logical {
        let offset = range_start + layout.chunk_offset(ordinal).unwrap() as usize;
        let length = (logical as usize - payload.len()).min(capacity);
        let coordinate = ExtentChunkCoordinate::new(
            placement.record(),
            placement.extent_cell(),
            logical,
            payload.len() as u64,
            ordinal,
        )
        .unwrap();
        let frame = &bytes[offset..offset + overhead + length];
        let (chunk, _) = decode_extent_chunk(frame, coordinate).unwrap();
        payload.extend_from_slice(chunk);
        ordinal += 1;
    }
    assert_eq!(ordinal - 1, manifest.chunk_count());
    payload
}

pub(super) fn arena_path(root: &Path, range: ExtentArenaRange) -> PathBuf {
    root.join(ARENAS)
        .join(format!("arena-{:016x}.data", range.arena().get()))
}

pub(super) fn flip_arena_byte(root: &Path, range: ExtentArenaRange) {
    let path = arena_path(root, range);
    let arena = std::fs::read(&path).unwrap();
    let start = range.offset() as usize;
    let (manifest, format) =
        DurableExtentManifest::decode(&arena[start..start + EXTENT_ARENA_MANIFEST_FRAME_BYTES])
            .unwrap();
    let layout = ExtentArenaFrameLayout::new(format, manifest.alignment()).unwrap();
    assert!(layout.admits(range, manifest.chunk_count()));
    let within = layout.chunk_offset(1).unwrap() + 128;
    assert!(within < range.length());
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    file.seek(SeekFrom::Start(range.offset() + within)).unwrap();
    let mut byte = [0];
    std::io::Read::read_exact(&mut file, &mut byte).unwrap();
    byte[0] ^= 0xff;
    file.seek(SeekFrom::Current(-1)).unwrap();
    file.write_all(&byte).unwrap();
    file.sync_all().unwrap();
}

pub(super) fn payload() -> Vec<u8> {
    (0..PAYLOAD_BYTES)
        .map(|index| (index * 31 % 251) as u8)
        .collect()
}
