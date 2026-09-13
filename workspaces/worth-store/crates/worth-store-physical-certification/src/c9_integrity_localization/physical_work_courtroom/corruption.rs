use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::artifact_manifest::PendingArtifact;
use super::process_protocol::{emit, Request};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) enum Operator {
    Clean,
    CoveredByte,
    ChecksumByte,
    WrongStore,
    Truncate,
    Duplicate,
    Unsupported,
}

impl Operator {
    pub const ALL: [Self; 7] = [
        Self::Clean,
        Self::CoveredByte,
        Self::ChecksumByte,
        Self::WrongStore,
        Self::Truncate,
        Self::Duplicate,
        Self::Unsupported,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Clean => "clean-pending",
            Self::CoveredByte => "B-covered-byte",
            Self::ChecksumByte => "K-checksum-byte",
            Self::WrongStore => "S-wrong-store",
            Self::Truncate => "T-truncated-tail",
            Self::Duplicate => "D-duplicated-identity",
            Self::Unsupported => "U-unsupported-version",
        }
    }
}

pub(super) fn run_editor(request: &Request, operator: Operator) {
    use std::io::Write;
    let artifact = PendingArtifact::observe(&request.root);
    let mut bytes = artifact.bytes.clone();
    let mut path = artifact.path.clone();
    match operator {
        Operator::Clean => {}
        Operator::CoveredByte => bytes[72] ^= 0x01,
        Operator::ChecksumByte => bytes[128] ^= 0x01,
        Operator::WrongStore => {
            bytes[16] ^= 0x01;
            refresh(&mut bytes);
        }
        Operator::Truncate => {
            bytes.pop().unwrap();
        }
        Operator::Duplicate => {
            path = artifact.duplicate_path();
        }
        Operator::Unsupported => {
            bytes[8] = 7;
            refresh(&mut bytes);
        }
    }
    if !matches!(operator, Operator::Clean) {
        let mut options = std::fs::OpenOptions::new();
        options.write(true);
        if matches!(operator, Operator::Duplicate) {
            options.create_new(true);
        } else {
            options.truncate(true);
        }
        let mut file = options.open(request.root.join(&path)).unwrap();
        file.write_all(&bytes).unwrap();
        file.sync_all().unwrap();
    }
    emit(
        request,
        json!({"operator": operator.label(), "path": path,
        "before_sha256": artifact.sha256, "after_sha256": <[u8; 32]>::from(Sha256::digest(&bytes)),
        "after_length": bytes.len()}),
    );
}

fn refresh(bytes: &mut [u8]) {
    let checksum = Sha256::digest(&bytes[..128]);
    bytes[128..160].copy_from_slice(&checksum);
}

pub(super) fn restore(root: &std::path::Path, artifact: &PendingArtifact, operator: Operator) {
    use std::io::Write;
    match operator {
        Operator::Clean => {}
        Operator::Duplicate => std::fs::remove_file(root.join(artifact.duplicate_path())).unwrap(),
        _ => {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(root.join(&artifact.path))
                .unwrap();
            file.write_all(&artifact.bytes).unwrap();
            file.sync_all().unwrap();
        }
    }
    artifact.require_unchanged(root);
}

pub(super) fn audit(
    source_root: &std::path::Path,
    root: &std::path::Path,
    manifest: &super::super::ClosedStoreProcessManifest,
    artifact: &PendingArtifact,
    operator: Operator,
) {
    let target = if matches!(operator, Operator::Duplicate) {
        artifact.duplicate_path()
    } else {
        artifact.path.clone()
    };
    let observed = std::fs::read(root.join(&target)).unwrap();
    let before = &artifact.bytes;
    match operator {
        Operator::Clean | Operator::Duplicate => assert_eq!(&observed, before),
        Operator::CoveredByte | Operator::ChecksumByte => {
            let offset = if matches!(operator, Operator::CoveredByte) {
                72
            } else {
                128
            };
            assert_eq!(observed.len(), 160);
            for index in 0..160 {
                assert_eq!(observed[index], before[index] ^ u8::from(index == offset));
            }
        }
        Operator::WrongStore | Operator::Unsupported => {
            let offset = if matches!(operator, Operator::WrongStore) {
                16
            } else {
                8
            };
            assert_eq!(observed.len(), 160);
            for index in 0..128 {
                assert_eq!(observed[index], before[index] ^ u8::from(index == offset));
            }
            assert_eq!(
                &observed[128..],
                Sha256::digest(&observed[..128]).as_slice()
            );
        }
        Operator::Truncate => assert_eq!(&observed, &before[..159]),
    }
    // Every unrelated artifact must retain the producer's bytes. The runner's
    // clone manifest supplies this inventory, not the editor's report.
    let after = super::super::ClosedStoreProcessManifest::observe(root).unwrap();
    assert_eq!(directory_paths(root), directory_paths(source_root));
    let before_paths = manifest.paths().collect::<Vec<_>>();
    let extra = usize::from(matches!(operator, Operator::Duplicate));
    assert_eq!(after.file_count(), manifest.file_count() + extra as u64);
    for path in after.paths() {
        assert!(before_paths.contains(&path) || path == target);
        if path != target {
            assert_eq!(
                std::fs::read(root.join(path)).unwrap(),
                std::fs::read(source_root.join(path)).unwrap()
            );
        }
    }
}

fn directory_paths(root: &std::path::Path) -> std::collections::BTreeSet<std::path::PathBuf> {
    let mut found = std::collections::BTreeSet::new();
    let mut remaining = vec![root.to_owned()];
    while let Some(directory) = remaining.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                assert!(found.len() < 4096, "producer directory inventory bound");
                found.insert(entry.path().strip_prefix(root).unwrap().to_owned());
                remaining.push(entry.path());
            }
        }
    }
    found
}
