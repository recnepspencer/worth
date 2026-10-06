//! The most one read of a whole artifact may return, from the fact that
//! declares it. A ceiling names its artifact, so it cannot be paired with
//! another, and no caller can state it as a number: a fixed slot's size is its
//! format's constant, and every other whole artifact a recovery reads is one
//! page of its declared format. An artifact longer than its ceiling is damage
//! no budget can fix.

use worth_store_physical_format::{
    PhysicalRecordFormatDeclaration, RecordArtifactFile, BOOTSTRAP_CATALOG_BYTES,
    ROOT_SELECTOR_BYTES,
};

/// The fixed-size slots a recovery reads whole.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixedArtifact {
    CurrentRootSelector,
    PreviousRootSelector,
    BootstrapCatalog,
}

/// The artifacts one page of the format bounds: the page-sized record files,
/// and one frame of a segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageAddress {
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
    FreeSpaceManifest {
        generation: u64,
    },
    FreeSpaceMembershipBlock {
        generation: u64,
        block: u64,
    },
    SegmentMembershipBlock {
        generation: u64,
        block: u64,
    },
    SegmentFrame {
        segment: u64,
        generation: u64,
        frame: u32,
    },
}

/// What one read may return of the artifact it names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactCeiling {
    file: RecordArtifactFile,
    extent: CeilingExtent,
}

/// How a ceiling bounds its artifact: a whole file no longer than `bytes`, or
/// exactly `length` bytes at `offset` of a larger one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CeilingExtent {
    Whole { bytes: u64, fixed: bool },
    Frame { offset: u64, length: u32 },
}

impl ArtifactCeiling {
    /// One page of `format`: a whole page-sized file, or one segment frame.
    pub fn page(format: PhysicalRecordFormatDeclaration, address: PageAddress) -> Self {
        let page = format.page_size().bytes();
        let whole = |file| Self {
            file,
            extent: CeilingExtent::Whole {
                bytes: u64::from(page),
                fixed: false,
            },
        };
        match address {
            PageAddress::RootManifest { generation } => {
                whole(RecordArtifactFile::RootManifest { generation })
            }
            PageAddress::RootRoutingBlock { generation, block } => {
                whole(RecordArtifactFile::RootRoutingBlock { generation, block })
            }
            PageAddress::ReleaseCustodyHeadBlock { generation, block } => {
                whole(RecordArtifactFile::ReleaseCustodyHeadBlock { generation, block })
            }
            PageAddress::FreeSpaceManifest { generation } => {
                whole(RecordArtifactFile::FreeSpaceManifest { generation })
            }
            PageAddress::FreeSpaceMembershipBlock { generation, block } => {
                whole(RecordArtifactFile::FreeSpaceMembershipBlock { generation, block })
            }
            PageAddress::SegmentMembershipBlock { generation, block } => {
                whole(RecordArtifactFile::SegmentMembershipBlock { generation, block })
            }
            PageAddress::SegmentFrame {
                segment,
                generation,
                frame,
            } => Self {
                file: RecordArtifactFile::Segment {
                    segment,
                    generation,
                },
                extent: CeilingExtent::Frame {
                    offset: u64::from(frame) * u64::from(page),
                    length: page,
                },
            },
        }
    }

    /// A fixed slot: its format's constant size.
    pub const fn fixed(slot: FixedArtifact) -> Self {
        let (file, bytes) = match slot {
            FixedArtifact::CurrentRootSelector => {
                (RecordArtifactFile::CurrentRootSelector, ROOT_SELECTOR_BYTES)
            }
            FixedArtifact::PreviousRootSelector => (
                RecordArtifactFile::PreviousRootSelector,
                ROOT_SELECTOR_BYTES,
            ),
            FixedArtifact::BootstrapCatalog => (
                RecordArtifactFile::BootstrapCatalog,
                BOOTSTRAP_CATALOG_BYTES,
            ),
        };
        Self {
            file,
            extent: CeilingExtent::Whole {
                bytes: bytes as u64,
                fixed: true,
            },
        }
    }

    /// The ceiling `format` declares for `file`, where it is a fixed slot or a
    /// page-sized file. `None` for any other artifact: nothing here declares
    /// how long it may be.
    pub fn of_record(
        format: PhysicalRecordFormatDeclaration,
        file: RecordArtifactFile,
    ) -> Option<Self> {
        if let Ok(slot) = FixedArtifact::try_from(file) {
            return Some(Self::fixed(slot));
        }
        PageAddress::try_from(file)
            .ok()
            .map(|address| Self::page(format, address))
    }

    pub const fn file(&self) -> RecordArtifactFile {
        self.file
    }

    pub(crate) const fn extent(&self) -> CeilingExtent {
        self.extent
    }
}

impl TryFrom<RecordArtifactFile> for FixedArtifact {
    type Error = RecordArtifactFile;

    fn try_from(file: RecordArtifactFile) -> Result<Self, Self::Error> {
        match file {
            RecordArtifactFile::CurrentRootSelector => Ok(Self::CurrentRootSelector),
            RecordArtifactFile::PreviousRootSelector => Ok(Self::PreviousRootSelector),
            RecordArtifactFile::BootstrapCatalog => Ok(Self::BootstrapCatalog),
            RecordArtifactFile::RootSelectorCandidate { .. }
            | RecordArtifactFile::CatalogCandidate { .. }
            | RecordArtifactFile::RootManifest { .. }
            | RecordArtifactFile::RootRoutingBlock { .. }
            | RecordArtifactFile::ReleaseCustodyHeadBlock { .. }
            | RecordArtifactFile::Segment { .. }
            | RecordArtifactFile::SegmentManifest { .. }
            | RecordArtifactFile::SegmentMembershipBlock { .. }
            | RecordArtifactFile::ExtentArena { .. }
            | RecordArtifactFile::FreeSpaceManifest { .. }
            | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => Err(file),
        }
    }
}

/// A whole page-sized file. A segment frame names a frame no file name
/// carries, so it is never converted from one.
impl TryFrom<RecordArtifactFile> for PageAddress {
    type Error = RecordArtifactFile;

    fn try_from(file: RecordArtifactFile) -> Result<Self, Self::Error> {
        match file {
            RecordArtifactFile::RootManifest { generation } => {
                Ok(Self::RootManifest { generation })
            }
            RecordArtifactFile::RootRoutingBlock { generation, block } => {
                Ok(Self::RootRoutingBlock { generation, block })
            }
            RecordArtifactFile::ReleaseCustodyHeadBlock { generation, block } => {
                Ok(Self::ReleaseCustodyHeadBlock { generation, block })
            }
            RecordArtifactFile::FreeSpaceManifest { generation } => {
                Ok(Self::FreeSpaceManifest { generation })
            }
            RecordArtifactFile::FreeSpaceMembershipBlock { generation, block } => {
                Ok(Self::FreeSpaceMembershipBlock { generation, block })
            }
            RecordArtifactFile::SegmentMembershipBlock { generation, block } => {
                Ok(Self::SegmentMembershipBlock { generation, block })
            }
            RecordArtifactFile::BootstrapCatalog
            | RecordArtifactFile::CurrentRootSelector
            | RecordArtifactFile::PreviousRootSelector
            | RecordArtifactFile::RootSelectorCandidate { .. }
            | RecordArtifactFile::CatalogCandidate { .. }
            | RecordArtifactFile::Segment { .. }
            | RecordArtifactFile::SegmentManifest { .. }
            | RecordArtifactFile::ExtentArena { .. } => Err(file),
        }
    }
}
