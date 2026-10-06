//! The most one read of a whole artifact may return, from the fact that
//! declares it. A ceiling names its artifact, so it cannot be paired with
//! another, and no caller can state a record's as a number: a fixed slot's
//! size is its format's constant, and every other whole record a recovery
//! reads is one page of its declared format. A stream is as long as the fact
//! the caller holds declares it; a stream no fact declares has no ceiling at
//! all, only the grant and the observation's bytes. An artifact longer than
//! its ceiling is damage no budget can fix.

use super::discovery::RecoveryDiscoveryArtifact;
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

/// The streams a recovery reads whole. A stream's length is declared only
/// where a fact the caller holds states it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamArtifact {
    CurrentCheckpoint,
}

/// What one read may return of the artifact it names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArtifactCeiling {
    artifact: CeilingArtifact,
    extent: CeilingExtent,
}

/// The artifact a ceiling names: a record file, or a stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CeilingArtifact {
    Record(RecordArtifactFile),
    Stream(StreamArtifact),
}

/// How a ceiling bounds its artifact: a whole file within `bytes`, or exactly
/// `length` bytes at `offset` of a larger one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CeilingExtent {
    Whole { bytes: CeilingBytes, fixed: bool },
    Frame { offset: u64, length: u32 },
}

/// How long a whole artifact may be: what a fact declares, or, for a stream
/// no fact declares, no length of its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CeilingBytes {
    Declared(u64),
    Undeclared,
}

impl CeilingBytes {
    /// The bytes a read may be asked for under this ceiling alone.
    pub(crate) const fn admitted(self) -> u64 {
        match self {
            Self::Declared(bytes) => bytes,
            Self::Undeclared => u64::MAX,
        }
    }

    /// `length` past this ceiling: the declared ceiling it passed. `None`
    /// within it, and always for a stream no fact declares.
    pub(crate) const fn passed_by(self, length: u64) -> Option<u64> {
        match self {
            Self::Declared(ceiling) if length > ceiling => Some(ceiling),
            Self::Declared(_) | Self::Undeclared => None,
        }
    }
}

impl ArtifactCeiling {
    /// One page of `format`: a whole page-sized file, or one segment frame.
    pub fn page(format: PhysicalRecordFormatDeclaration, address: PageAddress) -> Self {
        let page = format.page_size().bytes();
        let whole = |file| Self {
            artifact: CeilingArtifact::Record(file),
            extent: CeilingExtent::Whole {
                bytes: CeilingBytes::Declared(u64::from(page)),
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
                artifact: CeilingArtifact::Record(RecordArtifactFile::Segment {
                    segment,
                    generation,
                }),
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
            artifact: CeilingArtifact::Record(file),
            extent: CeilingExtent::Whole {
                bytes: CeilingBytes::Declared(bytes as u64),
                fixed: true,
            },
        }
    }

    /// A stream no fact declares a length for: only the read's grant and the
    /// observation's own bytes bound it.
    pub const fn undeclared(stream: StreamArtifact) -> Self {
        Self {
            artifact: CeilingArtifact::Stream(stream),
            extent: CeilingExtent::Whole {
                bytes: CeilingBytes::Undeclared,
                fixed: false,
            },
        }
    }

    /// A stream whose length a fact the caller holds declares, such as a
    /// checkpoint claim's encoded bytes: a longer stream is damage.
    pub const fn declared(stream: StreamArtifact, bytes: u64) -> Self {
        Self {
            artifact: CeilingArtifact::Stream(stream),
            extent: CeilingExtent::Whole {
                bytes: CeilingBytes::Declared(bytes),
                fixed: false,
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

    /// The artifact this ceiling names, as a read reports it.
    pub fn artifact(&self) -> RecoveryDiscoveryArtifact {
        match self.artifact {
            CeilingArtifact::Record(file) => RecoveryDiscoveryArtifact::Record(file),
            CeilingArtifact::Stream(StreamArtifact::CurrentCheckpoint) => {
                RecoveryDiscoveryArtifact::CurrentCheckpoint
            }
        }
    }

    pub(crate) const fn address(&self) -> CeilingArtifact {
        self.artifact
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
