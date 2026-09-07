use worth_foundational::PhysicalArtifactFamily;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChildExpectation {
    pub(crate) path: String,
    pub(crate) family: PhysicalArtifactFamily,
    pub(crate) generation: u64,
    pub(crate) format: [u8; 10],
    pub(crate) offset: u64,
    pub(crate) length: Option<u64>,
    pub(crate) checksum: Option<u32>,
    pub(crate) scope: ChildScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChildScope {
    Tree {
        tree: u64,
        block: u64,
        level: u16,
        capacity: u16,
        first: Vec<u8>,
        last: Vec<u8>,
    },
    FreeSpace {
        tree: u64,
        capacity: u16,
    },
    Page {
        segment: u64,
        page: u64,
        pages: u32,
    },
    ExtentManifest {
        extent: u64,
        record: [u8; 24],
        logical_bytes: u64,
    },
    ExtentChunk {
        extent: u64,
        record: [u8; 24],
        logical_bytes: u64,
        logical_offset: u64,
        ordinal: u32,
    },
}

impl ChildExpectation {
    pub(crate) fn identity(&self) -> String {
        match self.scope {
            ChildScope::Tree { block, .. } => format!("block:{block:016x}"),
            ChildScope::FreeSpace { .. } => format!("free-space:{:016x}", self.generation),
            ChildScope::Page { segment, page, .. } => format!("page:{segment:016x}:{page:016x}"),
            ChildScope::ExtentManifest { extent, .. } => format!("extent:{extent:016x}"),
            ChildScope::ExtentChunk {
                extent, ordinal, ..
            } => format!("extent:{extent:016x}:chunk:{ordinal}"),
        }
    }
}

pub(crate) fn tree_path(family: PhysicalArtifactFamily, generation: u64, block: u64) -> String {
    match family {
        PhysicalArtifactFamily::RootRoutingBlock => format!("families/records/roots/root-{generation:016x}-block-{block:016x}.manifest"),
        PhysicalArtifactFamily::SegmentMembershipBlock => format!("families/records/segment-manifests/segments-{generation:016x}-block-{block:016x}.manifest"),
        PhysicalArtifactFamily::FreeSpaceMembershipBlock => format!("families/records/free-space/free-space-{generation:016x}-block-{block:016x}.manifest"),
        _ => unreachable!("tree path requires a tree family"),
    }
}
