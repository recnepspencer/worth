use std::path::Path;

pub(crate) fn assert_candidate_prefix(
    root: &Path,
    snapshot: &super::snapshot::DirectorySnapshot,
    expected: &[u8],
) {
    let path = find_candidate(snapshot).expect("candidate creation must leave a candidate file");
    assert_eq!(record_kinds(&read_artifact(root, path)), expected);
}

pub(crate) fn assert_candidate_ending(
    root: &Path,
    snapshot: &super::snapshot::DirectorySnapshot,
    ending: u8,
    complete: bool,
) {
    let path = find_candidate(snapshot).expect("checkpoint stage must leave a candidate file");
    let kinds = record_kinds(&read_artifact(root, path));
    assert_eq!(kinds.last().copied(), Some(ending));
    assert_eq!(kinds.first().copied(), Some(1));
    assert!(
        kinds.contains(&2),
        "checkpoint frontier omitted dirty records"
    );
    if ending >= 3 {
        assert!(
            kinds.contains(&3),
            "checkpoint frontier omitted binding header"
        );
    }
    if ending >= 4 {
        assert!(
            kinds.contains(&4),
            "checkpoint frontier omitted binding records"
        );
    }
    assert_eq!(kinds.contains(&5), complete);
}

pub(crate) fn assert_complete_frontier(kinds: &[u8]) {
    assert_eq!(kinds.first().copied(), Some(1));
    assert!(
        kinds.contains(&2),
        "published checkpoint omitted dirty records"
    );
    assert!(
        kinds.contains(&3),
        "published checkpoint omitted binding header"
    );
    assert!(
        kinds.contains(&4),
        "published checkpoint omitted binding records"
    );
    assert_eq!(kinds.last().copied(), Some(5));
    assert!(
        kinds.iter().all(|kind| (1..=5).contains(kind)),
        "published checkpoint contained an unknown record kind"
    );
}

pub(crate) fn read_artifact(root: &Path, relative: &str) -> Vec<u8> {
    std::fs::read(root.join(relative)).expect("read checkpoint frontier artifact")
}

pub(crate) fn record_kinds(bytes: &[u8]) -> Vec<u8> {
    let mut kinds = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        assert!(bytes.len() - offset >= 20, "checkpoint record is truncated");
        assert_eq!(&bytes[offset..offset + 8], b"WCP7REC\0");
        assert_eq!(bytes[offset + 8], 1);
        let payload_bytes = u32::from_le_bytes(
            bytes[offset + 12..offset + 16]
                .try_into()
                .expect("checkpoint record length"),
        ) as usize;
        let frame_bytes = 20 + payload_bytes;
        assert!(
            frame_bytes <= bytes.len() - offset,
            "checkpoint record exceeds artifact"
        );
        kinds.push(bytes[offset + 9]);
        offset += frame_bytes;
    }
    kinds
}

fn find_candidate(snapshot: &super::snapshot::DirectorySnapshot) -> Option<&str> {
    let candidates = snapshot
        .keys()
        .filter(|path| path.starts_with("staging/") && path.ends_with(".candidate"))
        .collect::<Vec<_>>();
    assert!(
        candidates.len() <= 1,
        "checkpoint frontier exposed multiple candidate files"
    );
    candidates.first().map(|path| path.as_str())
}
