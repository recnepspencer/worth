use crate::manifest_fixture::IndependentExtentRoute;

pub(super) fn extent_payload() -> Vec<u8> {
    (0..super::EXTENT_PAYLOAD_BYTES)
        .map(|index| (index * 31 % 251) as u8)
        .collect()
}
use std::path::{Path, PathBuf};

pub(super) fn arena_file(root: &Path, arena: u64) -> PathBuf {
    root.join(format!("families/records/arenas/arena-{arena:016x}.data"))
}

pub(super) fn arena_range_bytes(root: &Path, route: IndependentExtentRoute) -> Vec<u8> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(arena_file(root, route.arena)).unwrap();
    file.seek(SeekFrom::Start(route.offset)).unwrap();
    let mut bytes = Vec::new();
    file.take(route.length).read_to_end(&mut bytes).unwrap();
    bytes
}
