use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use worth_store_physical_format::RecordArtifactFile;

use super::RootArtifactRole;

mod tree_observation;
mod snapshot;
use tree_observation::observe_tree;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ClosedStoreProcessManifest {
    store_identity: [u8; 16],
    current_selector: ProcessRootArtifact,
    current_root: ProcessRootArtifact,
    directories: BTreeSet<PathBuf>,
    files: BTreeMap<PathBuf, ProcessStoreFile>,
    identity: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessRootArtifact {
    role: RootArtifactRole,
    relative_path: PathBuf,
    concrete_identity: u64,
    root_generation: u64,
    exact_length: u64,
    content_sha256: [u8; 32],
    covered_edit_offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ProcessStoreFile {
    exact_length: u64,
    content_sha256: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProcessTreeSnapshot {
    live_lease_payload_excluded: bool,
    directories: BTreeSet<PathBuf>,
    files: BTreeMap<PathBuf, ProcessStoreFile>,
    contents: BTreeMap<PathBuf, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessManifestDenial {
    TreeRead,
    NonRegularEntry(PathBuf),
    MissingCurrentSelector,
    InvalidCurrentSelector,
    MissingCurrentRoot,
    InvalidCurrentRoot,
    RootGenerationMismatch,
    DestinationExists,
    CopyFailed,
    TreeMismatch,
    MutationMismatch,
    IdentityEncoding,
}

impl ClosedStoreProcessManifest {
    pub(crate) fn observe(root: &Path) -> Result<Self, ProcessManifestDenial> {
        let snapshot = ProcessTreeSnapshot::observe(root)?;
        let directories = &snapshot.directories;
        let files = &snapshot.files;
        let selector_path = PathBuf::from("families/records/root-current.selector");
        let selector_file = files
            .get(&selector_path)
            .ok_or(ProcessManifestDenial::MissingCurrentSelector)?;
        let selector_bytes = std::fs::read(root.join(&selector_path))
            .map_err(|_| ProcessManifestDenial::MissingCurrentSelector)?;
        // The manifest records the producer's bytes. It must not ask either
        // implementation under test to classify those bytes for the oracle.
        if selector_bytes.len() != 107 || &selector_bytes[..8] != b"WRC5FRM\0" {
            return Err(ProcessManifestDenial::InvalidCurrentSelector);
        }
        let root_generation = u64::from_le_bytes(selector_bytes[65..73].try_into().unwrap());
        let root_path = PathBuf::from("families/records/roots").join(
            RecordArtifactFile::RootManifest {
                generation: root_generation,
            }
            .file_name(),
        );
        let root_file = files
            .get(&root_path)
            .ok_or(ProcessManifestDenial::MissingCurrentRoot)?;
        let root_bytes = std::fs::read(root.join(&root_path))
            .map_err(|_| ProcessManifestDenial::MissingCurrentRoot)?;
        if root_bytes.len() < 48 || &root_bytes[..8] != b"WRC5FRM\0" || root_bytes[8] != 2 {
            return Err(ProcessManifestDenial::InvalidCurrentRoot);
        }
        if u64::from_le_bytes(root_bytes[28..36].try_into().unwrap()) != root_generation {
            return Err(ProcessManifestDenial::RootGenerationMismatch);
        }
        let store_identity = selector_bytes[48..64].try_into().unwrap();
        let current_selector = ProcessRootArtifact {
            role: RootArtifactRole::CurrentSelector,
            relative_path: selector_path,
            concrete_identity: u64::from_le_bytes(selector_bytes[28..36].try_into().unwrap()),
            root_generation,
            exact_length: selector_file.exact_length,
            content_sha256: selector_file.content_sha256,
            covered_edit_offset: 48,
        };
        let current_root = ProcessRootArtifact {
            role: RootArtifactRole::AddressedRootManifest,
            relative_path: root_path,
            concrete_identity: root_generation,
            root_generation,
            exact_length: root_file.exact_length,
            content_sha256: root_file.content_sha256,
            covered_edit_offset: 56,
        };
        let identity = manifest_identity(
            store_identity,
            &current_selector,
            &current_root,
            directories,
            files,
        )?;
        Ok(Self {
            store_identity,
            current_selector,
            current_root,
            directories: snapshot.directories,
            files: snapshot.files,
            identity,
        })
    }

    pub(crate) fn copy_to(
        &self,
        source: &Path,
        destination: &Path,
    ) -> Result<(), ProcessManifestDenial> {
        if destination.exists() {
            return Err(ProcessManifestDenial::DestinationExists);
        }
        self.require_unchanged(source)?;
        std::fs::create_dir_all(destination).map_err(|_| ProcessManifestDenial::CopyFailed)?;
        for relative in &self.directories {
            std::fs::create_dir(destination.join(relative))
                .map_err(|_| ProcessManifestDenial::CopyFailed)?;
        }
        for relative in self.files.keys() {
            let target = destination.join(relative);
            let parent = target.parent().ok_or(ProcessManifestDenial::CopyFailed)?;
            std::fs::create_dir_all(parent).map_err(|_| ProcessManifestDenial::CopyFailed)?;
            std::fs::copy(source.join(relative), target)
                .map_err(|_| ProcessManifestDenial::CopyFailed)?;
        }
        self.require_unchanged(destination)
    }

    pub(crate) fn require_unchanged(&self, root: &Path) -> Result<(), ProcessManifestDenial> {
        let snapshot = ProcessTreeSnapshot::observe(root)?;
        if snapshot.directories == self.directories && snapshot.files == self.files {
            Ok(())
        } else {
            Err(ProcessManifestDenial::TreeMismatch)
        }
    }

    pub(crate) fn artifact(&self, role: RootArtifactRole) -> Option<&ProcessRootArtifact> {
        match role {
            RootArtifactRole::CurrentSelector => Some(&self.current_selector),
            RootArtifactRole::AddressedRootManifest => Some(&self.current_root),
            RootArtifactRole::PreviousSelector => None,
        }
    }

    pub(crate) const fn store_identity(&self) -> [u8; 16] {
        self.store_identity
    }
    pub(crate) const fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub(crate) fn file_count(&self) -> u64 {
        self.files.len() as u64
    }
    pub(crate) fn byte_count(&self) -> u64 {
        self.files.values().map(|file| file.exact_length).sum()
    }
    pub(crate) fn paths(&self) -> impl Iterator<Item = &Path> {
        self.files.keys().map(PathBuf::as_path)
    }
}


impl ProcessRootArtifact {
    pub(crate) const fn role(&self) -> RootArtifactRole {
        self.role
    }
    pub(crate) fn relative_path(&self) -> &Path {
        &self.relative_path
    }
    pub(crate) const fn concrete_identity(&self) -> u64 {
        self.concrete_identity
    }
    pub(crate) const fn root_generation(&self) -> u64 {
        self.root_generation
    }
    pub(crate) const fn exact_length(&self) -> u64 {
        self.exact_length
    }
    pub(crate) const fn content_sha256(&self) -> [u8; 32] {
        self.content_sha256
    }
    pub(crate) const fn covered_edit_offset(&self) -> u64 {
        self.covered_edit_offset
    }
}

fn manifest_identity(
    store_identity: [u8; 16],
    current_selector: &ProcessRootArtifact,
    current_root: &ProcessRootArtifact,
    directories: &BTreeSet<PathBuf>,
    files: &BTreeMap<PathBuf, ProcessStoreFile>,
) -> Result<[u8; 32], ProcessManifestDenial> {
    bincode::serialize(&(
        "worth-store-c9-production-root-manifest-v1",
        store_identity,
        current_selector,
        current_root,
        directories,
        files,
    ))
    .map(|bytes| Sha256::digest(bytes).into())
    .map_err(|_| ProcessManifestDenial::IdentityEncoding)
}
