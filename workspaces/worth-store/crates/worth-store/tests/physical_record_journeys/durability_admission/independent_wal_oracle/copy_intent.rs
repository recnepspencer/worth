use std::path::{Path, PathBuf};

const HEADER: usize = 116;
const FOOTER: usize = 32;
const DOMAIN: &[u8] = b"store.physical.extent-copy.v1";
const CLASSIFIED_DOMAIN: &[u8] = b"store.physical.extent-copy.v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IndependentCopyIntent {
    pub(crate) operation: [u8; 32],
    pub(crate) source_root: u64,
    pub(crate) extent: u64,
    pub(crate) source_generation: u64,
    pub(crate) destination_generation: u64,
    pub(crate) source: [u64; 3],
    pub(crate) destination: [u64; 3],
    pub(crate) payload_bytes: u64,
}

/// Walks framed WAL members, then parses literal copy-intent fields. It does
/// not import the writer codec or search arbitrary payload bytes for a domain.
pub(crate) fn produced_copy_intents(root: &Path) -> Vec<IndependentCopyIntent> {
    let mut files = Vec::new();
    collect(&root.join("families/wal"), &mut files);
    files.sort();
    let mut intents = Vec::new();
    for path in files {
        let bytes = std::fs::read(&path).unwrap();
        let mut offset = 0usize;
        while let Some(header) = bytes.get(offset..offset + HEADER) {
            assert_eq!(&header[..8], b"WORTHWAL", "bad WAL frame in {path:?}");
            let length = usize::try_from(number(header, 44)).unwrap();
            let start = offset + HEADER;
            let end = start.checked_add(length).unwrap();
            let Some(payload) = bytes.get(start..end) else {
                break;
            };
            if bytes.get(end..end + FOOTER).is_none() {
                break;
            }
            let intent = payload
                .strip_prefix(CLASSIFIED_DOMAIN)
                .map(|bytes| (bytes, true))
                .or_else(|| payload.strip_prefix(DOMAIN).map(|bytes| (bytes, false)));
            if let Some((body, classified)) = intent.and_then(|(bytes, classified)| {
                bytes.strip_prefix(&[1]).map(|body| (body, classified))
            }) {
                assert_eq!(
                    body.len(),
                    if classified { 214 } else { 200 },
                    "malformed framed copy intent"
                );
                if classified {
                    let source_route = &body[200..207];
                    let destination_route = &body[207..214];
                    assert!(valid_classified_route(source_route), "invalid source route");
                    assert_ne!(source_route[0], 0, "classified source cannot be unknown");
                    assert!(
                        valid_classified_route(destination_route),
                        "invalid destination route"
                    );
                    assert_eq!(
                        source_route[..4],
                        destination_route[..4],
                        "copy must preserve selected content class"
                    );
                }
                let operation = body[..32].try_into().unwrap();
                let triple = |at| {
                    [
                        number(body, at),
                        number(body, at + 8),
                        number(body, at + 16),
                    ]
                };
                let source = triple(96);
                let destination = triple(120);
                assert_ne!(operation, [0; 32]);
                assert_ne!(source[0], destination[0]);
                assert_eq!(source[2], destination[2]);
                intents.push(IndependentCopyIntent {
                    operation,
                    source_root: number(body, 32),
                    extent: number(body, 72),
                    source_generation: number(body, 80),
                    destination_generation: number(body, 88),
                    source,
                    destination,
                    payload_bytes: number(body, 144),
                });
            }
            offset = end + FOOTER;
        }
    }
    intents
}

fn valid_classified_route(bytes: &[u8]) -> bool {
    if bytes.len() != 7 || bytes[5..] != [0; 2] || bytes[4] > 2 {
        return false;
    }
    let family = u16::from_le_bytes([bytes[2], bytes[3]]);
    match (bytes[0], bytes[1], family) {
        (0, 0, 0) => bytes[4] == 0,
        (1 | 4, 0, 0) => true,
        (2, 1..=16, 0) => true,
        (3, 0, 1..) => true,
        _ => false,
    }
}

fn number(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else {
            files.push(path);
        }
    }
}
