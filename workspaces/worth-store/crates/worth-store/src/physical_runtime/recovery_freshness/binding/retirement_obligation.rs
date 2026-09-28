use crate::physical_runtime::durability::{unresolved_retirements, RetiredArtifact};

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

pub(super) fn retirement_obligations(
    records: Vec<crate::physical_runtime::durability::RetirementRecord>,
) -> Box<[StoreRecoveryRetirementObligation]> {
    unresolved_retirements(records)
        .into_iter()
        .map(|record| StoreRecoveryRetirementObligation {
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
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use crate::physical_runtime::durability::{RetiredArtifact, RetirementRecord};

    use super::{retirement_obligations, StoreRecoveryRetiredArtifact};

    #[test]
    fn completion_removes_the_reconstructed_obligation() {
        let intent = RetirementRecord {
            artifact: RetiredArtifact::Segment {
                segment: 1,
                generation: 7,
            },
            completion: false,
            source_root: 3,
            bytes: 32,
            release: None,
        };
        let completion = RetirementRecord {
            completion: true,
            ..intent
        };
        assert_eq!(retirement_obligations(vec![intent]).len(), 1);
        assert!(retirement_obligations(vec![intent, completion]).is_empty());
    }

    fn range() -> worth_store_physical_format::ExtentArenaRange {
        worth_store_physical_format::ExtentArenaRange::new(
            worth_store_physical_format::ExtentArenaId::new(1).unwrap(),
            4096,
            8192,
        )
        .unwrap()
    }

    #[test]
    fn an_extent_obligation_keeps_its_artifact_kind() {
        let intent = RetirementRecord {
            artifact: RetiredArtifact::Extent {
                extent: 1,
                generation: 7,
                range: range(),
            },
            completion: false,
            source_root: 3,
            bytes: 32,
            release: Some(
                crate::physical_runtime::durability::RetirementReleaseProjection::new(
                    4, 5, [7; 32], 4096, 9,
                )
                .unwrap(),
            ),
        };
        let obligations = retirement_obligations(vec![intent]);
        assert_eq!(
            obligations[0].artifact(),
            StoreRecoveryRetiredArtifact::Extent {
                extent: 1,
                generation: 7,
                range: range()
            }
        );
        assert_eq!(obligations[0].artifact().segment(), None);
    }
}
