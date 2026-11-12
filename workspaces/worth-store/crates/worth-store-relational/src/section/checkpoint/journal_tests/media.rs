//! Actual original frame observation and fixture-family damage.
use worth_store::physical_runtime::{RecordByteLimit, RecordReadLimits, ServingPhysicalRuntime};

pub(super) struct JournalFrame {
    pub(super) bytes: Vec<u8>,
    pub(super) first_chunk: Vec<u8>,
    pub(super) touched_extents: u64,
}

pub(super) fn journal_frame(serving: &ServingPhysicalRuntime, position: u64) -> JournalFrame {
    let entry = serving
        .canonical_commits()
        .unwrap()
        .read(position, 1 << 30)
        .unwrap()
        .entry();
    let reader = serving.records().unwrap();
    let mut session = reader
        .open(
            entry.record(),
            RecordReadLimits::new(RecordByteLimit::new(1 << 30).unwrap()),
        )
        .unwrap();
    let mut bytes = Vec::new();
    let mut first_chunk = Vec::new();
    while let Some(chunk) = session.next_chunk().unwrap() {
        assert_eq!(chunk.basis().record(), entry.record());
        assert_eq!(chunk.logical_range().start, bytes.len() as u64);
        if bytes.is_empty() {
            assert_eq!(chunk.logical_range().start, 0);
            first_chunk.extend_from_slice(chunk.bytes());
        }
        bytes.extend_from_slice(chunk.bytes());
    }
    assert!(bytes.starts_with(b"WSCJREC1"));
    JournalFrame {
        bytes,
        first_chunk,
        touched_extents: session.observation().touched_extents(),
    }
}
pub(super) fn corrupt_original(root: &std::path::Path, fragment: &[u8]) -> usize {
    fn visit(path: &std::path::Path, frame: &[u8]) -> usize {
        if path.is_dir() {
            return std::fs::read_dir(path)
                .unwrap()
                .map(|entry| visit(&entry.unwrap().path(), frame))
                .sum();
        }
        let mut bytes = std::fs::read(path).unwrap();
        let mut found = 0;
        for start in 0..=bytes.len().saturating_sub(frame.len()) {
            if bytes.get(start..start + frame.len()) == Some(frame) {
                bytes[start + frame.len() - 1] ^= 1;
                found += 1;
            }
        }
        if found != 0 {
            std::fs::write(path, bytes).unwrap();
        }
        found
    }
    std::fs::read_dir(root.join("families"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.file_name().unwrap() != "wal")
        .map(|path| visit(&path, fragment))
        .sum()
}
