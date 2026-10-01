use worth_store_physical_format::{
    DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadDenial, ReleaseCustodyHeadEntryV1,
};

mod legacy;
mod seen;
mod walk;

/// Whole-root traversal and resident limits. `max_resident_bytes` covers
/// walker-owned scratch; a carried port may impose a stricter shared ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCustodyHeadWalkLimitsV1 {
    pub(super) max_nodes: u64,
    pub(super) max_entries: u64,
    pub(super) max_total_frame_bytes: u64,
    pub(super) max_resident_bytes: u64,
    pub(super) max_depth: u16,
}

impl ReleaseCustodyHeadWalkLimitsV1 {
    pub fn new(
        max_nodes: u64,
        max_entries: u64,
        max_total_frame_bytes: u64,
        max_resident_bytes: u64,
        max_depth: u16,
    ) -> Option<Self> {
        (max_nodes > 0 && max_total_frame_bytes > 0 && max_resident_bytes > 0 && max_depth > 0)
            .then_some(Self {
                max_nodes,
                max_entries,
                max_total_frame_bytes,
                max_resident_bytes,
                max_depth,
            })
    }

    pub const fn max_resident_bytes(self) -> u64 {
        self.max_resident_bytes
    }

    pub fn with_max_resident_bytes(mut self, ceiling: u64) -> Option<Self> {
        self.max_resident_bytes = self.max_resident_bytes.min(ceiling);
        (self.max_resident_bytes > 0).then_some(self)
    }

    /// Minimum for one rooted read. Callers with a larger node limit should
    /// use `root_resident_preflight_bytes` for the exact initial flat table.
    pub fn minimum_root_resident_bytes(format: PhysicalRecordFormatDeclaration) -> Option<u64> {
        Self::root_resident_preflight_bytes(format, 1)
    }

    pub fn root_resident_preflight_bytes(
        format: PhysicalRecordFormatDeclaration,
        max_nodes: u64,
    ) -> Option<u64> {
        seen::requested_bytes(max_nodes)?
            .checked_add(std::mem::size_of::<(ReleaseCustodyHeadBlockReferenceV1, u16)>() as u64)?
            .checked_add(u64::from(format.page_size().bytes()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReleaseCustodyHeadWalkV1 {
    pub(super) entry_count: u64,
    pub(super) roster_digest: [u8; 32],
    pub(super) node_count: u64,
    pub(super) frame_bytes_read: u64,
    pub(super) peak_resident_bytes: u64,
}

impl ReleaseCustodyHeadWalkV1 {
    pub const fn entry_count(self) -> u64 {
        self.entry_count
    }
    pub const fn roster_digest(self) -> [u8; 32] {
        self.roster_digest
    }
    pub const fn node_count(self) -> u64 {
        self.node_count
    }
    pub const fn frame_bytes_read(self) -> u64 {
        self.frame_bytes_read
    }
    pub const fn peak_resident_bytes(self) -> u64 {
        self.peak_resident_bytes
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReleaseCustodyHeadWalkDenial<ReadError, VisitError> {
    Read(ReadError),
    Visit(VisitError),
    Storage(ReadError),
    Format(ReleaseCustodyHeadDenial),
    Root,
    DuplicateNode,
    BoundExceeded,
    Allocation {
        requested: u64,
        cause: std::collections::TryReserveError,
    },
    ResidentBoundExceeded {
        required: u64,
        admitted: u64,
    },
}

/// Mechanical allocation and observation port. This creates no selected-media
/// authority; the caller still supplies rooted bytes and receives typed checks.
/// `read_node` returns a Vec already charged by this same port. The walker
/// calls `discard_vec` after its borrowed block view is no longer live.
pub trait ReleaseCustodyHeadWalkPort {
    type Error;

    fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Self::Error>;
    fn grow_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Self::Error>;
    fn discard_vec<T>(&mut self, values: Vec<T>);
    fn read_node(
        &mut self,
        reference: ReleaseCustodyHeadBlockReferenceV1,
        remaining: u64,
    ) -> Result<Vec<u8>, Self::Error>;
    fn visit_node(
        &mut self,
        reference: ReleaseCustodyHeadBlockReferenceV1,
        frame: &[u8],
    ) -> Result<(), Self::Error>;
    fn visit_entry(&mut self, entry: ReleaseCustodyHeadEntryV1) -> Result<(), Self::Error>;
}

pub fn walk_release_custody_head_with_port<P: ReleaseCustodyHeadWalkPort>(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    limits: ReleaseCustodyHeadWalkLimitsV1,
    port: &mut P,
) -> Result<ReleaseCustodyHeadWalkV1, ReleaseCustodyHeadWalkDenial<P::Error, P::Error>> {
    walk::walk(root, format, limits, port)
}

/// Existing callback entry delegates to the same parser and walker. New Store
/// V2 rejoin uses the port entry so its resident ledger cannot be reset here.
pub fn walk_release_custody_head<Read, Visit, ReadError, VisitError>(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    limits: ReleaseCustodyHeadWalkLimitsV1,
    read: Read,
    visit: Visit,
) -> Result<ReleaseCustodyHeadWalkV1, ReleaseCustodyHeadWalkDenial<ReadError, VisitError>>
where
    Read: FnMut(ReleaseCustodyHeadBlockReferenceV1, u64) -> Result<Vec<u8>, ReadError>,
    Visit: FnMut(ReleaseCustodyHeadEntryV1) -> Result<(), VisitError>,
{
    legacy::walk(root, format, limits, read, visit)
}

#[cfg(test)]
mod tests;
