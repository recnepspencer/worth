use worth_store_physical_format::RecordArtifactFile;

/// One displaced payload generation that retirement may delete.
///
/// A segment generation is one file. An extent generation is its chunk file
/// and its manifest, and retirement deletes both or neither completes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(in crate::physical_runtime) enum RetiredArtifact {
    Segment { segment: u64, generation: u64 },
    Extent { extent: u64, generation: u64 },
}

impl RetiredArtifact {
    pub(in crate::physical_runtime) const fn id(self) -> u64 {
        match self {
            Self::Segment { segment, .. } => segment,
            Self::Extent { extent, .. } => extent,
        }
    }

    pub(in crate::physical_runtime) const fn generation(self) -> u64 {
        match self {
            Self::Segment { generation, .. } | Self::Extent { generation, .. } => generation,
        }
    }

    /// Every file this generation owns, in deletion order.
    pub(in crate::physical_runtime) fn files(self) -> Vec<RecordArtifactFile> {
        match self {
            Self::Segment {
                segment,
                generation,
            } => vec![RecordArtifactFile::Segment {
                segment,
                generation,
            }],
            Self::Extent { extent, generation } => vec![
                RecordArtifactFile::Extent { extent, generation },
                RecordArtifactFile::ExtentManifest { extent, generation },
            ],
        }
    }

    /// Whether `artifact` is one of the files this exact generation owns.
    pub(in crate::physical_runtime) fn admits_removal(self, artifact: RecordArtifactFile) -> bool {
        self.files().contains(&artifact)
    }

    /// Retirement WAL action codes: 1 and 2 name a segment, 3 and 4 an extent.
    pub(in crate::physical_runtime) const fn action_code(self, completion: bool) -> u8 {
        match (self, completion) {
            (Self::Segment { .. }, false) => super::retirement::RETIREMENT_INTENT,
            (Self::Segment { .. }, true) => super::retirement::RETIREMENT_COMPLETION,
            (Self::Extent { .. }, false) => super::retirement::RETIREMENT_EXTENT_INTENT,
            (Self::Extent { .. }, true) => super::retirement::RETIREMENT_EXTENT_COMPLETION,
        }
    }

    /// Decodes an action code into the artifact kind and whether it completes.
    pub(in crate::physical_runtime) const fn from_action(
        action: u8,
        id: u64,
        generation: u64,
    ) -> Option<(Self, bool)> {
        match action {
            super::retirement::RETIREMENT_INTENT | super::retirement::RETIREMENT_COMPLETION => {
                Some((
                    Self::Segment {
                        segment: id,
                        generation,
                    },
                    action == super::retirement::RETIREMENT_COMPLETION,
                ))
            }
            super::retirement::RETIREMENT_EXTENT_INTENT
            | super::retirement::RETIREMENT_EXTENT_COMPLETION => Some((
                Self::Extent {
                    extent: id,
                    generation,
                },
                action == super::retirement::RETIREMENT_EXTENT_COMPLETION,
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_extent_generation_owns_its_chunks_and_manifest_only() {
        let extent = RetiredArtifact::Extent {
            extent: 4,
            generation: 2,
        };
        assert!(extent.admits_removal(RecordArtifactFile::Extent {
            extent: 4,
            generation: 2
        }));
        assert!(extent.admits_removal(RecordArtifactFile::ExtentManifest {
            extent: 4,
            generation: 2
        }));
        assert!(!extent.admits_removal(RecordArtifactFile::Extent {
            extent: 4,
            generation: 3
        }));
        assert!(!extent.admits_removal(RecordArtifactFile::Segment {
            segment: 4,
            generation: 2
        }));
        let segment = RetiredArtifact::Segment {
            segment: 4,
            generation: 2,
        };
        assert_eq!(segment.files().len(), 1);
        assert_ne!(segment, extent, "equal ids and generations stay distinct");
    }

    #[test]
    fn action_codes_round_trip_the_artifact_kind() {
        for artifact in [
            RetiredArtifact::Segment {
                segment: 1,
                generation: 2,
            },
            RetiredArtifact::Extent {
                extent: 1,
                generation: 2,
            },
        ] {
            for completion in [false, true] {
                let code = artifact.action_code(completion);
                assert_eq!(
                    RetiredArtifact::from_action(code, 1, 2),
                    Some((artifact, completion))
                );
            }
        }
        assert_eq!(RetiredArtifact::from_action(5, 1, 2), None);
        assert_eq!(RetiredArtifact::from_action(0, 1, 2), None);
    }
}
