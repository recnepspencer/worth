use crate::{WalSegmentGeneration, WalSegmentId};
use std::path::PathBuf;
use worth_store_physical_format::WalSegmentIdentity;

/// Canonical identity encoded by one Store-owned WAL segment artifact name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WalSegmentArtifactIdentity {
    format_identity: WalSegmentIdentity,
}

pub(in crate::artifact_store) fn wal_segment_relative_path(
    segment: u64,
    generation: u64,
) -> Result<PathBuf, crate::WalArtifactStoreDenial> {
    let identity = WalSegmentArtifactIdentity::new(
        WalSegmentId::new(segment).map_err(|_| crate::WalArtifactStoreDenial::InvalidFrame)?,
        WalSegmentGeneration::new(generation)
            .map_err(|_| crate::WalArtifactStoreDenial::InvalidFrame)?,
    );
    Ok(PathBuf::from("wal").join(identity.file_name()))
}

impl WalSegmentArtifactIdentity {
    pub const fn new(segment: WalSegmentId, generation: WalSegmentGeneration) -> Self {
        Self {
            format_identity: match WalSegmentIdentity::new(segment.get(), generation.get()) {
                Some(identity) => identity,
                None => unreachable!(),
            },
        }
    }

    pub fn parse(file_name: &str) -> Option<Self> {
        let body = file_name.strip_prefix("segment-")?.strip_suffix(".wal")?;
        let (segment, generation) = body.split_once("-generation-")?;
        let identity = Self::new(
            WalSegmentId::new(segment.parse().ok()?).ok()?,
            WalSegmentGeneration::new(generation.parse().ok()?).ok()?,
        );
        (identity.file_name() == file_name).then_some(identity)
    }

    pub fn file_name(self) -> String {
        format!(
            "segment-{}-generation-{}.wal",
            self.format_identity.segment().get(),
            self.format_identity.generation().get()
        )
    }

    pub const fn segment(self) -> WalSegmentId {
        match WalSegmentId::new(self.format_identity.segment().get()) {
            Ok(segment) => segment,
            Err(_) => unreachable!(),
        }
    }

    pub const fn generation(self) -> WalSegmentGeneration {
        match WalSegmentGeneration::new(self.format_identity.generation().get()) {
            Ok(generation) => generation,
            Err(_) => unreachable!(),
        }
    }

    pub const fn format_identity(self) -> WalSegmentIdentity {
        self.format_identity
    }
}
