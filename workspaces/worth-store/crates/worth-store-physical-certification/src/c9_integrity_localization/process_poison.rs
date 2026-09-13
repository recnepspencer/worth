use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{ClosedStoreProcessManifest, RootArtifactRole};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessRootCase {
    CleanControl,
    PoisonCurrentSelector,
    PoisonAddressedRoot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct DeclaredProcessPoison {
    role: RootArtifactRole,
    offset: u64,
    xor_mask: u8,
    manifest_identity: [u8; 32],
    identity: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessEditorAudit {
    declaration_identity: [u8; 32],
    before_sha256: [u8; 32],
    after_sha256: [u8; 32],
    changed_offset: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProcessPoisonDenial {
    CleanCaseHasNoPoison,
    MissingArtifact,
    ManifestSubstitution,
    BaselineChanged,
    InvalidOffset,
    Read,
    Write,
    IdentityEncoding,
}

impl ProcessRootCase {
    pub(crate) const fn role(self) -> Option<RootArtifactRole> {
        match self {
            Self::CleanControl => None,
            Self::PoisonCurrentSelector => Some(RootArtifactRole::CurrentSelector),
            Self::PoisonAddressedRoot => Some(RootArtifactRole::AddressedRootManifest),
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::CleanControl => "clean-control",
            Self::PoisonCurrentSelector => "poison-current-selector",
            Self::PoisonAddressedRoot => "poison-addressed-root",
        }
    }
}

impl DeclaredProcessPoison {
    pub(crate) fn for_case(
        manifest: &ClosedStoreProcessManifest,
        case: ProcessRootCase,
    ) -> Result<Self, ProcessPoisonDenial> {
        let role = case
            .role()
            .ok_or(ProcessPoisonDenial::CleanCaseHasNoPoison)?;
        let artifact = manifest
            .artifact(role)
            .ok_or(ProcessPoisonDenial::MissingArtifact)?;
        let offset = artifact.covered_edit_offset();
        let xor_mask = 1;
        let manifest_identity = manifest.identity();
        let identity = bincode::serialize(&(
            "worth-store-c9-format-aware-covered-byte-poison-v1",
            role,
            offset,
            xor_mask,
            manifest_identity,
        ))
        .map(|bytes| Sha256::digest(bytes).into())
        .map_err(|_| ProcessPoisonDenial::IdentityEncoding)?;
        Ok(Self {
            role,
            offset,
            xor_mask,
            manifest_identity,
            identity,
        })
    }

    pub(crate) const fn role(&self) -> RootArtifactRole {
        self.role
    }
    pub(crate) const fn offset(&self) -> u64 {
        self.offset
    }
    pub(crate) const fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub(crate) const fn xor_mask(&self) -> u8 {
        self.xor_mask
    }
}

pub(crate) fn apply_process_poison(
    root: &Path,
    manifest: &ClosedStoreProcessManifest,
    declaration: &DeclaredProcessPoison,
) -> Result<ProcessEditorAudit, ProcessPoisonDenial> {
    if declaration.manifest_identity != manifest.identity() {
        return Err(ProcessPoisonDenial::ManifestSubstitution);
    }
    manifest
        .require_unchanged(root)
        .map_err(|_| ProcessPoisonDenial::BaselineChanged)?;
    let artifact = manifest
        .artifact(declaration.role)
        .ok_or(ProcessPoisonDenial::MissingArtifact)?;
    let path = root.join(artifact.relative_path());
    let mut bytes = std::fs::read(&path).map_err(|_| ProcessPoisonDenial::Read)?;
    if Sha256::digest(&bytes).as_slice() != artifact.content_sha256()
        || bytes.len() as u64 != artifact.exact_length()
    {
        return Err(ProcessPoisonDenial::BaselineChanged);
    }
    let offset = usize::try_from(declaration.offset)
        .ok()
        .filter(|offset| *offset < bytes.len())
        .ok_or(ProcessPoisonDenial::InvalidOffset)?;
    bytes[offset] ^= declaration.xor_mask;
    std::fs::write(path, &bytes).map_err(|_| ProcessPoisonDenial::Write)?;
    Ok(ProcessEditorAudit {
        declaration_identity: declaration.identity,
        before_sha256: artifact.content_sha256(),
        after_sha256: Sha256::digest(bytes).into(),
        changed_offset: declaration.offset,
    })
}

impl ProcessEditorAudit {
    pub(crate) const fn declaration_identity(&self) -> [u8; 32] {
        self.declaration_identity
    }
    pub(crate) const fn before_sha256(&self) -> [u8; 32] {
        self.before_sha256
    }
    pub(crate) const fn after_sha256(&self) -> [u8; 32] {
        self.after_sha256
    }
    pub(crate) const fn changed_offset(&self) -> u64 {
        self.changed_offset
    }
}
