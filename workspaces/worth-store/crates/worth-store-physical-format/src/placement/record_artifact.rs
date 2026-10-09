#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecordArtifactFile {
    BootstrapCatalog,
    CurrentRootSelector,
    PreviousRootSelector,
    RootSelectorCandidate {
        role: crate::RootSelectorRole,
        publication: u64,
    },
    CatalogCandidate {
        publication: u64,
    },
    RootManifest {
        generation: u64,
    },
    RootRoutingBlock {
        generation: u64,
        block: u64,
    },
    ReleaseCustodyHeadBlock {
        generation: u64,
        block: u64,
    },
    Segment {
        segment: u64,
        generation: u64,
    },
    SegmentManifest {
        segment: u64,
        generation: u64,
    },
    SegmentMembershipBlock {
        generation: u64,
        block: u64,
    },
    ExtentArena {
        arena: u64,
    },
    FreeSpaceManifest {
        generation: u64,
    },
    FreeSpaceMembershipBlock {
        generation: u64,
        block: u64,
    },
}

/// Canonical artifact spelling held inline; identity hashing needs no heap allocation.
pub struct RecordArtifactFileName {
    bytes: [u8; 64],
    length: usize,
}

impl RecordArtifactFileName {
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.length]).expect("artifact names are ASCII")
    }
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}

impl std::fmt::Write for RecordArtifactFileName {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        let end = self
            .length
            .checked_add(value.len())
            .ok_or(std::fmt::Error)?;
        let destination = self
            .bytes
            .get_mut(self.length..end)
            .ok_or(std::fmt::Error)?;
        destination.copy_from_slice(value.as_bytes());
        self.length = end;
        Ok(())
    }
}

impl RecordArtifactFile {
    pub fn file_name(self) -> String {
        self.canonical_file_name().as_str().to_owned()
    }

    pub fn canonical_file_name(self) -> RecordArtifactFileName {
        use std::fmt::Write;
        let mut name = RecordArtifactFileName {
            bytes: [0; 64],
            length: 0,
        };
        match self {
            Self::BootstrapCatalog => name.write_str("bootstrap.catalog"),
            Self::CurrentRootSelector => name.write_str("root-current.selector"),
            Self::PreviousRootSelector => name.write_str("root-previous.selector"),
            Self::RootSelectorCandidate { role, publication } => match role {
                crate::RootSelectorRole::Current => {
                    write!(name, "root-current-{publication:016x}.candidate")
                }
                crate::RootSelectorRole::Previous => {
                    write!(name, "root-previous-{publication:016x}.candidate")
                }
            },
            Self::CatalogCandidate { publication } => {
                write!(name, "bootstrap-{publication:016x}.candidate")
            }
            Self::RootManifest { generation } => write!(name, "root-{generation:016x}.manifest"),
            Self::RootRoutingBlock { generation, block } => {
                write!(name, "root-{generation:016x}-block-{block:016x}.manifest")
            }
            Self::ReleaseCustodyHeadBlock { generation, block } => {
                write!(
                    name,
                    "release-head-{generation:016x}-block-{block:016x}.manifest"
                )
            }
            Self::Segment {
                segment,
                generation,
            } => {
                write!(name, "segment-{segment:016x}-{generation:016x}.pages")
            }
            Self::SegmentManifest {
                segment,
                generation,
            } => {
                write!(name, "segment-{segment:016x}-{generation:016x}.manifest")
            }
            Self::SegmentMembershipBlock { generation, block } => {
                write!(
                    name,
                    "segments-{generation:016x}-block-{block:016x}.manifest"
                )
            }
            Self::ExtentArena { arena } => {
                write!(name, "arena-{arena:016x}.data")
            }
            Self::FreeSpaceManifest { generation } => {
                write!(name, "free-space-{generation:016x}.manifest")
            }
            Self::FreeSpaceMembershipBlock { generation, block } => {
                write!(
                    name,
                    "free-space-{generation:016x}-block-{block:016x}.manifest"
                )
            }
        }
        .expect("the longest artifact name fits its canonical inline buffer");
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_name_preserves_protocol_spelling_at_maximum_identity_width() {
        let head = RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: u64::MAX,
            block: u64::MAX,
        };
        assert_eq!(
            head.canonical_file_name().as_str(),
            "release-head-ffffffffffffffff-block-ffffffffffffffff.manifest"
        );
        let extent = RecordArtifactFile::ExtentArena { arena: u64::MAX };
        assert_eq!(
            extent.canonical_file_name().as_str(),
            "arena-ffffffffffffffff.data"
        );
        let selector = RecordArtifactFile::RootSelectorCandidate {
            role: crate::RootSelectorRole::Previous,
            publication: u64::MAX,
        };
        assert_eq!(
            selector.canonical_file_name().as_str(),
            "root-previous-ffffffffffffffff.candidate"
        );
        for artifact in [
            head,
            extent,
            selector,
            RecordArtifactFile::BootstrapCatalog,
            RecordArtifactFile::CurrentRootSelector,
            RecordArtifactFile::PreviousRootSelector,
        ] {
            assert_eq!(
                artifact.file_name().as_bytes(),
                artifact.canonical_file_name().as_bytes()
            );
        }
    }
}
