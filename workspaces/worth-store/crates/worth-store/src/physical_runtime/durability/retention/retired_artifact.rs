use worth_store_physical_format::{ExtentArenaRange, RecordArtifactFile};

/// A displaced segment file or exact arena allocation. Arena range retirement
/// changes published free-space authority; it never removes the shared file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(in crate::physical_runtime) enum RetiredArtifact {
    Segment {
        segment: u64,
        generation: u64,
    },
    Extent {
        extent: u64,
        generation: u64,
        range: ExtentArenaRange,
    },
    Arena {
        arena: u64,
        generation: u64,
    },
}

impl RetiredArtifact {
    pub(in crate::physical_runtime) const fn id(self) -> u64 {
        match self {
            Self::Segment { segment, .. } => segment,
            Self::Extent { extent, .. } => extent,
            Self::Arena { arena, .. } => arena,
        }
    }
    pub(in crate::physical_runtime) const fn generation(self) -> u64 {
        match self {
            Self::Segment { generation, .. }
            | Self::Extent { generation, .. }
            | Self::Arena { generation, .. } => generation,
        }
    }
    pub(in crate::physical_runtime) const fn arena_range(self) -> Option<ExtentArenaRange> {
        match self {
            Self::Extent { range, .. } => Some(range),
            Self::Segment { .. } | Self::Arena { .. } => None,
        }
    }
    /// Only a segment generation owns an independently removable file.
    pub(in crate::physical_runtime) fn files(self) -> Vec<RecordArtifactFile> {
        match self {
            Self::Segment {
                segment,
                generation,
            } => vec![RecordArtifactFile::Segment {
                segment,
                generation,
            }],
            Self::Extent { .. } => Vec::new(),
            Self::Arena { arena, .. } => vec![RecordArtifactFile::ExtentArena { arena }],
        }
    }
    pub(in crate::physical_runtime) fn admits_removal(self, artifact: RecordArtifactFile) -> bool {
        self.files().contains(&artifact)
    }
    pub(in crate::physical_runtime) const fn action_code(self, completion: bool) -> u8 {
        match (self, completion) {
            (Self::Segment { .. }, false) => super::retirement::RETIREMENT_INTENT,
            (Self::Segment { .. }, true) => super::retirement::RETIREMENT_COMPLETION,
            (Self::Extent { .. }, false) => super::retirement::RETIREMENT_EXTENT_INTENT,
            (Self::Extent { .. }, true) => super::retirement::RETIREMENT_EXTENT_COMPLETION,
            (Self::Arena { .. }, false) => 5,
            (Self::Arena { .. }, true) => 6,
        }
    }
    pub(in crate::physical_runtime) fn from_action(
        action: u8,
        id: u64,
        generation: u64,
        range: Option<ExtentArenaRange>,
    ) -> Option<(Self, bool)> {
        if id == 0 || generation == 0 {
            return None;
        }
        match action {
            1 | 2 if range.is_none() => Some((
                Self::Segment {
                    segment: id,
                    generation,
                },
                action == 2,
            )),
            3 | 4 => Some((
                Self::Extent {
                    extent: id,
                    generation,
                    range: range?,
                },
                action == 4,
            )),
            5 | 6 if range.is_none() => Some((
                Self::Arena {
                    arena: id,
                    generation,
                },
                action == 6,
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn an_extent_allocation_never_authorizes_removing_its_shared_arena() {
        let range = ExtentArenaRange::new(
            worth_store_physical_format::ExtentArenaId::new(4).unwrap(),
            4096,
            8192,
        )
        .unwrap();
        let extent = RetiredArtifact::Extent {
            extent: 4,
            generation: 2,
            range,
        };
        assert!(extent.files().is_empty());
        assert!(!extent.admits_removal(RecordArtifactFile::ExtentArena { arena: 4 }));
        for completion in [false, true] {
            assert_eq!(
                RetiredArtifact::from_action(extent.action_code(completion), 4, 2, Some(range)),
                Some((extent, completion))
            );
        }
        assert!(RetiredArtifact::from_action(3, 4, 2, None).is_none());
        assert!(RetiredArtifact::from_action(1, 4, 2, Some(range)).is_none());
    }
}
