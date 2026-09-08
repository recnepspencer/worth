use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(super) struct PendingArtifact {
    pub path: PathBuf,
    pub store: [u8; 16],
    pub runtime: u64,
    pub generation: u64,
    pub operation: u64,
    pub bytes: Vec<u8>,
    pub sha256: [u8; 32],
}

impl PendingArtifact {
    pub fn observe(root: &Path) -> Self {
        let paths = std::fs::read_dir(root.join("families/physical-work"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|value| value == "pending"))
            .collect::<Vec<_>>();
        assert_eq!(
            paths.len(),
            1,
            "one ordinary mutation paused at its first target write"
        );
        let path = paths[0].strip_prefix(root).unwrap().to_owned();
        let bytes = std::fs::read(&paths[0]).unwrap();
        assert_eq!(bytes.len(), 160);
        assert_eq!(&bytes[..8], b"WPEFFECT");
        assert_eq!(bytes[8], 6);
        assert_eq!(&bytes[128..], Sha256::digest(&bytes[..128]).as_slice());
        let runtime = u64::from_le_bytes(bytes[32..40].try_into().unwrap());
        let generation = u64::from_le_bytes(bytes[40..48].try_into().unwrap());
        let operation = u64::from_le_bytes(bytes[48..56].try_into().unwrap());
        assert!(runtime > 0 && generation > 0 && operation > 0);
        assert_eq!(
            path.file_name().unwrap().to_str().unwrap(),
            format!("effect-{runtime:016x}-{generation:016x}-{operation:016x}.pending")
        );
        Self {
            path,
            store: bytes[16..32].try_into().unwrap(),
            runtime,
            generation,
            operation,
            sha256: Sha256::digest(&bytes).into(),
            bytes,
        }
    }

    pub fn duplicate_path(&self) -> PathBuf {
        self.path.with_file_name(format!(
            "effect-{:016x}-{:016x}-{:016x}.pending",
            self.runtime,
            self.generation,
            self.operation.checked_add(1).unwrap()
        ))
    }

    pub fn require_unchanged(&self, root: &Path) {
        assert_eq!(std::fs::read(root.join(&self.path)).unwrap(), self.bytes);
    }
}
