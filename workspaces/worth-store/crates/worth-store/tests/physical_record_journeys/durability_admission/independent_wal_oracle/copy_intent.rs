use std::path::{Path, PathBuf};

const HEADER: usize = 116;
const FOOTER: usize = 32;
const DOMAIN: &[u8] = b"store.physical.extent-copy.v1";

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
            if let Some(body) = payload
                .strip_prefix(DOMAIN)
                .and_then(|rest| rest.strip_prefix(&[1]))
            {
                assert_eq!(body.len(), 200, "malformed framed copy intent");
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
