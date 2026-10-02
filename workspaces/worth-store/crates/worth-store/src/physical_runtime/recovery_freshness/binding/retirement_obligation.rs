use crate::physical_runtime::durability::{RetiredArtifact, RetirementRecord};

use super::StoreRecoveryBindingSampleDenial;

/// An unresolved `store.physical.retirement.v2` intent reconstructed from WAL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreRecoveryRetirementObligation {
    source_root: u64,
    artifact: StoreRecoveryRetiredArtifact,
    bytes: u64,
    release: Option<crate::physical_runtime::durability::RetirementReleaseProjection>,
}

/// The exact displaced generation a retirement intent claimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreRecoveryRetiredArtifact {
    Segment {
        segment: u64,
        generation: u64,
    },
    Extent {
        extent: u64,
        generation: u64,
        range: worth_store_physical_format::ExtentArenaRange,
    },
    Arena {
        arena: u64,
        generation: u64,
    },
}

impl StoreRecoveryRetirementObligation {
    pub const fn release(
        &self,
    ) -> Option<crate::physical_runtime::durability::RetirementReleaseProjection> {
        self.release
    }
    pub const fn source_root(&self) -> u64 {
        self.source_root
    }
    pub const fn artifact(&self) -> StoreRecoveryRetiredArtifact {
        self.artifact
    }
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl StoreRecoveryRetiredArtifact {
    /// The segment id when the retired generation is an inline segment.
    pub const fn segment(&self) -> Option<u64> {
        match *self {
            Self::Segment { segment, .. } => Some(segment),
            Self::Extent { .. } | Self::Arena { .. } => None,
        }
    }
    /// The extent id when the retired generation is an extent.
    pub const fn extent(&self) -> Option<u64> {
        match *self {
            Self::Extent { extent, .. } => Some(extent),
            Self::Segment { .. } | Self::Arena { .. } => None,
        }
    }
    pub const fn generation(&self) -> u64 {
        match *self {
            Self::Segment { generation, .. }
            | Self::Extent { generation, .. }
            | Self::Arena { generation, .. } => generation,
        }
    }
}

/// Arrival order remains authoritative within each retired artifact. The caller
/// funds both prepared buffers; this fold neither grows nor boxes either one.
pub(super) fn fold_retirement_records(
    mut records: Vec<(usize, RetirementRecord)>,
    mut output: Vec<StoreRecoveryRetirementObligation>,
) -> Result<Vec<StoreRecoveryRetirementObligation>, StoreRecoveryBindingSampleDenial> {
    if !output.is_empty() {
        return Err(StoreRecoveryBindingSampleDenial::InvalidWalMember);
    }
    if output.capacity() < records.len() {
        return Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit);
    }
    records.sort_unstable_by_key(|(ordinal, record)| (record.artifact, *ordinal));
    let mut position = 0;
    while position < records.len() {
        let artifact = records[position].1.artifact;
        let mut intent: Option<RetirementRecord> = None;
        while position < records.len() && records[position].1.artifact == artifact {
            let record = records[position].1;
            if !record.completion {
                if intent.is_none() {
                    intent = Some(record);
                }
            } else if intent.is_some_and(|intent| {
                intent.source_root == record.source_root
                    && intent.bytes == record.bytes
                    && intent.release == record.release
            }) {
                intent = None;
            }
            position += 1;
        }
        if let Some(record) = intent {
            if output.len() == output.capacity() {
                return Err(StoreRecoveryBindingSampleDenial::RecoveryMemoryLimit);
            }
            output.push(project_retirement(record));
        }
    }
    drop(records);
    Ok(output)
}

fn project_retirement(record: RetirementRecord) -> StoreRecoveryRetirementObligation {
    StoreRecoveryRetirementObligation {
        source_root: record.source_root,
        artifact: match record.artifact {
            RetiredArtifact::Segment {
                segment,
                generation,
            } => StoreRecoveryRetiredArtifact::Segment {
                segment,
                generation,
            },
            RetiredArtifact::Extent {
                extent,
                generation,
                range,
            } => StoreRecoveryRetiredArtifact::Extent {
                extent,
                generation,
                range,
            },
            RetiredArtifact::Arena { arena, generation } => {
                StoreRecoveryRetiredArtifact::Arena { arena, generation }
            }
        },
        bytes: record.bytes,
        release: record.release,
    }
}

#[cfg(test)]
mod tests;
