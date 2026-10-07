#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OfflineRecordIdentity {
    allocation_epoch: [u8; 16],
    ordinal: u64,
}

impl OfflineRecordIdentity {
    pub(super) fn decode(bytes: &[u8]) -> Option<Self> {
        let allocation_epoch: [u8; 16] = bytes.get(..16)?.try_into().ok()?;
        let ordinal = u64::from_le_bytes(bytes.get(16..24)?.try_into().ok()?);
        (allocation_epoch != [0; 16] && ordinal != 0).then_some(Self {
            allocation_epoch,
            ordinal,
        })
    }

    pub const fn allocation_epoch(self) -> [u8; 16] {
        self.allocation_epoch
    }

    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineRecordPlacement {
    Inline {
        record: OfflineRecordIdentity,
        segment: u64,
        page: u64,
        segment_generation: u64,
        page_generation: u64,
        slot_generation: u64,
        payload_bytes: u64,
        segment_page_capacity: u32,
        slot: u16,
    },
    Extent {
        record: OfflineRecordIdentity,
        extent: u64,
        generation: u64,
        payload_bytes: u64,
        arena: u64,
        arena_offset: u64,
        arena_length: u64,
    },
}

impl OfflineRecordPlacement {
    pub const fn record(self) -> OfflineRecordIdentity {
        match self {
            Self::Inline { record, .. } | Self::Extent { record, .. } => record,
        }
    }

    pub const fn payload_bytes(self) -> u64 {
        match self {
            Self::Inline { payload_bytes, .. } | Self::Extent { payload_bytes, .. } => {
                payload_bytes
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineRecordPayloadObservation {
    record: OfflineRecordIdentity,
    payload_bytes: u64,
    prefix: Box<[u8]>,
    digest: [u8; 32],
}

impl OfflineRecordPayloadObservation {
    pub(super) fn new(
        record: OfflineRecordIdentity,
        payload_bytes: u64,
        prefix: Box<[u8]>,
        digest: [u8; 32],
    ) -> Self {
        Self {
            record,
            payload_bytes,
            prefix,
            digest,
        }
    }

    pub const fn record(&self) -> OfflineRecordIdentity {
        self.record
    }

    pub const fn payload_bytes(&self) -> u64 {
        self.payload_bytes
    }

    pub fn prefix(&self) -> &[u8] {
        &self.prefix
    }

    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfflineSegmentPageMembership {
    pub(super) segment: u64,
    pub(super) page: u64,
    pub(super) page_generation: u64,
    pub(super) data_generation: u64,
    pub(super) data_page_count: u32,
    pub(super) frame_index: u32,
}

impl OfflineSegmentPageMembership {
    pub const fn segment(self) -> u64 {
        self.segment
    }
    pub const fn page(self) -> u64 {
        self.page
    }
    pub const fn page_generation(self) -> u64 {
        self.page_generation
    }
    pub const fn data_generation(self) -> u64 {
        self.data_generation
    }
    pub const fn data_page_count(self) -> u32 {
        self.data_page_count
    }
    pub const fn frame_index(self) -> u32 {
        self.frame_index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OfflineAllocationClass {
    InlinePage,
    ExtentArena,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OfflineFreeSpaceMembership {
    pub(super) class: OfflineAllocationClass,
    pub(super) owner: u64,
    pub(super) first_unallocated: u64,
    pub(super) unallocated_count: u64,
    pub(super) generation: u64,
}

impl OfflineFreeSpaceMembership {
    pub const fn class(self) -> OfflineAllocationClass {
        self.class
    }
    pub const fn owner(self) -> u64 {
        self.owner
    }
    pub const fn first_unallocated(self) -> u64 {
        self.first_unallocated
    }
    pub const fn unallocated_count(self) -> u64 {
        self.unallocated_count
    }
    pub const fn generation(self) -> u64 {
        self.generation
    }
}
