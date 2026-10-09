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
        Some(Self::new(
            WalSegmentId::new(canonical_positive_decimal(segment)?).ok()?,
            WalSegmentGeneration::new(canonical_positive_decimal(generation)?).ok()?,
        ))
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

/// An artifact name is its identity only when both numbers have their unique
/// ASCII spelling; parsing must not allocate a rendered name for comparison.
fn canonical_positive_decimal(value: &str) -> Option<u64> {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes[0] == b'0' {
        return None;
    }
    bytes.iter().try_fold(0_u64, |number, byte| {
        if !byte.is_ascii_digit() {
            return None;
        }
        number.checked_mul(10)?.checked_add(u64::from(byte - b'0'))
    })
}
