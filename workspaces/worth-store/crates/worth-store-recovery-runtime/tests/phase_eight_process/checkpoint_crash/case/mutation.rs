use std::path::Path;

pub(super) const PERSISTED_WAL_FRAME_BYTE_FLIP: &str = "WalFrameHeaderByteFlip";

pub(super) fn mutate_persisted_wal_frame(root: &Path) {
    let wal_directory = root.join("families/wal");
    let mut paths = std::fs::read_dir(&wal_directory)
        .expect("read persisted WAL mutation directory")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(std::fs::FileType::is_file)
                .map(|_| entry.path())
        })
        .collect::<Vec<_>>();
    paths.sort();
    let path = paths
        .first()
        .expect("persisted WAL mutation requires a WAL frame")
        .clone();
    let mut bytes = std::fs::read(&path).expect("read persisted WAL for mutation fixture");
    assert!(
        !bytes.is_empty(),
        "{PERSISTED_WAL_FRAME_BYTE_FLIP} requires a non-empty persisted WAL frame"
    );
    bytes[12..20].copy_from_slice(&0_u64.to_le_bytes());
    std::fs::write(path, bytes).expect("persist WAL byte mutation fixture");
}
