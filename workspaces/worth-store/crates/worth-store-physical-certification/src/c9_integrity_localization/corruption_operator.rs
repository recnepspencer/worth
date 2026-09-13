use std::ops::Range;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{CleanRootArtifactManifest, RootArtifactIdentity};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum RootCorruptionCode {
    B,
    K,
    L,
    S,
    P,
    T,
    R,
    D,
    U,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum RootCorruptionOperation {
    CoveredByteFlip { offset: u64, mask: u8 },
    ChecksumFieldFlip { offset: u64, mask: u8 },
    FramingLengthLie { encoded_payload_length: u32 },
    ScopeSubstitution { source_path: PathBuf },
    PointerCorruption { range: Range<u64>, replacement: u64 },
    StrictPrefixTruncation { retained_length: u64 },
    ArtifactRemoval,
    ArtifactDuplication { destination: PathBuf },
    UnsupportedFormatVersion { range: Range<u64>, value: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct DeclaredRootCorruption {
    target: RootArtifactIdentity,
    code: RootCorruptionCode,
    operation: RootCorruptionOperation,
    declaration_identity: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RootCorruptionDeclarationDenial {
    UnknownTarget,
    IdentityEncoding,
}

impl DeclaredRootCorruption {
    pub(crate) fn for_code(
        manifest: &CleanRootArtifactManifest,
        target: RootArtifactIdentity,
        code: RootCorruptionCode,
    ) -> Result<Self, RootCorruptionDeclarationDenial> {
        let record = manifest
            .record(target)
            .ok_or(RootCorruptionDeclarationDenial::UnknownTarget)?;
        let operation = match code {
            RootCorruptionCode::B => RootCorruptionOperation::CoveredByteFlip {
                offset: record.covered_edit_offset(),
                mask: 0x01,
            },
            RootCorruptionCode::K => RootCorruptionOperation::ChecksumFieldFlip {
                offset: record.checksum_range().start,
                mask: 0x01,
            },
            RootCorruptionCode::L => RootCorruptionOperation::FramingLengthLie {
                encoded_payload_length: (record.exact_length() as u32)
                    .saturating_sub(48)
                    .saturating_add(1),
            },
            RootCorruptionCode::S => RootCorruptionOperation::ScopeSubstitution {
                source_path: record.substitution_source_path().to_path_buf(),
            },
            RootCorruptionCode::P => RootCorruptionOperation::PointerCorruption {
                range: record.pointer_range(),
                replacement: target.root_generation().saturating_add(101),
            },
            RootCorruptionCode::T => RootCorruptionOperation::StrictPrefixTruncation {
                retained_length: record.exact_length() - 1,
            },
            RootCorruptionCode::R => RootCorruptionOperation::ArtifactRemoval,
            RootCorruptionCode::D => RootCorruptionOperation::ArtifactDuplication {
                destination: record.duplicate_path().to_path_buf(),
            },
            RootCorruptionCode::U => RootCorruptionOperation::UnsupportedFormatVersion {
                range: record.version_range(),
                value: u16::MAX,
            },
        };
        let declaration_identity = bincode::serialize(&(
            "worth-store-c9-root-corruption-v1",
            manifest.identity(),
            target,
            code,
            &operation,
        ))
        .map(|bytes| Sha256::digest(bytes).into())
        .map_err(|_| RootCorruptionDeclarationDenial::IdentityEncoding)?;
        Ok(Self {
            target,
            code,
            operation,
            declaration_identity,
        })
    }

    pub(crate) const fn target(&self) -> RootArtifactIdentity {
        self.target
    }
    pub(crate) const fn code(&self) -> RootCorruptionCode {
        self.code
    }
    pub(crate) const fn operation(&self) -> &RootCorruptionOperation {
        &self.operation
    }
    pub(crate) const fn identity(&self) -> [u8; 32] {
        self.declaration_identity
    }

    pub(crate) fn is_exact_for(&self, manifest: &CleanRootArtifactManifest) -> bool {
        Self::for_code(manifest, self.target, self.code).as_ref() == Ok(self)
    }
}
