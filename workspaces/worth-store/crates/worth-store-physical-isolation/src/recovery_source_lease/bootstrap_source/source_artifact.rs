use std::path::{Component, Path, PathBuf};

use super::BootstrapSourceResolutionDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BootstrapSourceArtifactFamily {
    Authority,
    Checkpoint,
    Wal,
    Page,
    Blob,
    Layout,
}

impl BootstrapSourceArtifactFamily {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::Authority => 1,
            Self::Checkpoint => 2,
            Self::Wal => 3,
            Self::Page => 4,
            Self::Blob => 5,
            Self::Layout => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapSourceArtifact {
    family: BootstrapSourceArtifactFamily,
    relative_path: PathBuf,
    byte_length: u64,
    content_digest: [u8; 32],
}

impl BootstrapSourceArtifact {
    pub fn declare(
        family: BootstrapSourceArtifactFamily,
        relative_path: impl Into<PathBuf>,
        byte_length: u64,
        content_digest: [u8; 32],
    ) -> Result<Self, BootstrapSourceResolutionDenial> {
        let relative_path = relative_path.into();
        if !is_safe_relative_path(&relative_path) || content_digest == [0; 32] {
            return Err(BootstrapSourceResolutionDenial::InvalidArtifact);
        }
        Ok(Self {
            family,
            relative_path,
            byte_length,
            content_digest,
        })
    }

    pub const fn family(&self) -> BootstrapSourceArtifactFamily {
        self.family
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    pub const fn content_digest(&self) -> [u8; 32] {
        self.content_digest
    }
}

pub(super) fn is_safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}
