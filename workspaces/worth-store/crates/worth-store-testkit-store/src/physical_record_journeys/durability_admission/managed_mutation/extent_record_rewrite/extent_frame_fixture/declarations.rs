pub use {crate::physical_record_journeys::manifest_fixture::IndependentExtentRoute};

pub fn extent_payload() -> Vec<u8> {
    (0..crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::EXTENT_PAYLOAD_BYTES)
        .map(|index| (index * 31 % 251) as u8)
        .collect()
}
pub use {std::path::Path, std::path::PathBuf};

pub fn arena_file(root: &Path, arena: u64) -> PathBuf {
    root.join(format!("families/records/arenas/arena-{arena:016x}.data"))
}

pub fn arena_range_bytes(root: &Path, route: IndependentExtentRoute) -> Vec<u8> {
    use {std::io::Read, std::io::Seek, std::io::SeekFrom};
    let mut file = std::fs::File::open(arena_file(root, route.arena)).unwrap();
    file.seek(SeekFrom::Start(route.offset)).unwrap();
    let mut bytes = Vec::new();
    file.take(route.length).read_to_end(&mut bytes).unwrap();
    bytes
}