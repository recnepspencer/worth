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

impl RecordArtifactFile {
    pub fn file_name(self) -> String {
        match self {
            Self::BootstrapCatalog => "bootstrap.catalog".to_owned(),
            Self::CurrentRootSelector => "root-current.selector".to_owned(),
            Self::PreviousRootSelector => "root-previous.selector".to_owned(),
            Self::RootSelectorCandidate { role, publication } => match role {
                crate::RootSelectorRole::Current => {
                    format!("root-current-{publication:016x}.candidate")
                }
                crate::RootSelectorRole::Previous => {
                    format!("root-previous-{publication:016x}.candidate")
                }
            },
            Self::CatalogCandidate { publication } => {
                format!("bootstrap-{publication:016x}.candidate")
            }
            Self::RootManifest { generation } => format!("root-{generation:016x}.manifest"),
            Self::RootRoutingBlock { generation, block } => {
                format!("root-{generation:016x}-block-{block:016x}.manifest")
            }
            Self::Segment {
                segment,
                generation,
            } => {
                format!("segment-{segment:016x}-{generation:016x}.pages")
            }
            Self::SegmentManifest {
                segment,
                generation,
            } => {
                format!("segment-{segment:016x}-{generation:016x}.manifest")
            }
            Self::SegmentMembershipBlock { generation, block } => {
                format!("segments-{generation:016x}-block-{block:016x}.manifest")
            }
            Self::ExtentArena { arena } => {
                format!("arena-{arena:016x}.data")
            }
            Self::FreeSpaceManifest { generation } => {
                format!("free-space-{generation:016x}.manifest")
            }
            Self::FreeSpaceMembershipBlock { generation, block } => {
                format!("free-space-{generation:016x}-block-{block:016x}.manifest")
            }
        }
    }
}
