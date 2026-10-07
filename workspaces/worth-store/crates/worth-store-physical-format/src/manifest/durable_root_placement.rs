use crate::{
    PageGenerationCell, PersistedRecordIdentity, PhysicalExtentId, PhysicalPageId,
    PhysicalRecordSlot, PhysicalSegmentId, RecordExtentGenerationCell, SegmentGenerationCell,
    SelectedRecordRouteMetadata, SlotGenerationCell,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurableInlineRecordPlacement {
    record: PersistedRecordIdentity,
    segment: SegmentGenerationCell,
    page: PageGenerationCell,
    slot: SlotGenerationCell,
    segment_page_capacity: u32,
    payload_bytes: u64,
    route_metadata: SelectedRecordRouteMetadata,
}

impl DurableInlineRecordPlacement {
    pub fn legacy_unknown(
        record: PersistedRecordIdentity,
        segment: SegmentGenerationCell,
        page: PageGenerationCell,
        slot: SlotGenerationCell,
        segment_page_capacity: u32,
        payload_bytes: u64,
    ) -> Option<Self> {
        (segment_page_capacity > 0
            && payload_bytes <= u64::from(u32::MAX)
            && page.segment_id() == segment.segment_id()
            && slot.segment_id() == segment.segment_id()
            && slot.page_id() == page.page_id())
        .then_some(Self {
            record,
            segment,
            page,
            slot,
            segment_page_capacity,
            payload_bytes,
            route_metadata: SelectedRecordRouteMetadata::legacy_primary(),
        })
    }

    pub fn new_selected(
        record: PersistedRecordIdentity,
        segment: SegmentGenerationCell,
        page: PageGenerationCell,
        slot: SlotGenerationCell,
        segment_page_capacity: u32,
        payload_bytes: u64,
        route_metadata: SelectedRecordRouteMetadata,
    ) -> Option<Self> {
        let mut placement = Self::legacy_unknown(
            record,
            segment,
            page,
            slot,
            segment_page_capacity,
            payload_bytes,
        )?;
        placement.route_metadata = route_metadata;
        Some(placement)
    }

    pub const fn route_metadata(self) -> SelectedRecordRouteMetadata {
        self.route_metadata
    }
    pub const fn content_class(self) -> crate::SelectedRecordContentClass {
        self.route_metadata.content_class()
    }
    pub const fn tier_class(self) -> crate::PhysicalTierClass {
        self.route_metadata.tier_class()
    }

    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn segment(self) -> PhysicalSegmentId {
        self.segment.segment_id()
    }
    pub const fn segment_cell(self) -> SegmentGenerationCell {
        self.segment
    }
    pub const fn segment_generation(self) -> u64 {
        self.segment.generation().get()
    }
    pub const fn page(self) -> PhysicalPageId {
        self.page.page_id()
    }
    pub const fn page_cell(self) -> PageGenerationCell {
        self.page
    }
    pub const fn page_generation(self) -> u64 {
        self.page.generation().get()
    }
    pub const fn slot(self) -> PhysicalRecordSlot {
        self.slot.slot()
    }
    pub const fn slot_cell(self) -> SlotGenerationCell {
        self.slot
    }
    pub const fn slot_generation(self) -> u64 {
        self.slot.generation().get()
    }
    pub const fn segment_page_capacity(self) -> u32 {
        self.segment_page_capacity
    }
    pub const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurableExtentRecordPlacement {
    record: PersistedRecordIdentity,
    extent: RecordExtentGenerationCell,
    payload_bytes: u64,
    arena_range: crate::ExtentArenaRange,
    route_metadata: SelectedRecordRouteMetadata,
}

impl DurableExtentRecordPlacement {
    pub const fn legacy_unknown(
        record: PersistedRecordIdentity,
        extent: RecordExtentGenerationCell,
        payload_bytes: u64,
        arena_range: crate::ExtentArenaRange,
    ) -> Option<Self> {
        if payload_bytes > u32::MAX as u64 {
            return None;
        }
        Some(Self {
            record,
            extent,
            payload_bytes,
            arena_range,
            route_metadata: SelectedRecordRouteMetadata::legacy_primary(),
        })
    }

    pub const fn new_selected(
        record: PersistedRecordIdentity,
        extent: RecordExtentGenerationCell,
        payload_bytes: u64,
        arena_range: crate::ExtentArenaRange,
        route_metadata: SelectedRecordRouteMetadata,
    ) -> Option<Self> {
        let mut placement = match Self::legacy_unknown(record, extent, payload_bytes, arena_range) {
            Some(placement) => placement,
            None => return None,
        };
        placement.route_metadata = route_metadata;
        Some(placement)
    }

    pub const fn route_metadata(self) -> SelectedRecordRouteMetadata {
        self.route_metadata
    }
    pub const fn content_class(self) -> crate::SelectedRecordContentClass {
        self.route_metadata.content_class()
    }
    pub const fn tier_class(self) -> crate::PhysicalTierClass {
        self.route_metadata.tier_class()
    }

    pub const fn record(self) -> PersistedRecordIdentity {
        self.record
    }
    pub const fn extent(self) -> PhysicalExtentId {
        self.extent.extent_id()
    }
    pub const fn arena_range(self) -> crate::ExtentArenaRange {
        self.arena_range
    }
    pub const fn extent_cell(self) -> RecordExtentGenerationCell {
        self.extent
    }
    pub const fn extent_generation(self) -> u64 {
        self.extent.generation().get()
    }
    pub const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurrentPhysicalRecordPlacement {
    Inline(DurableInlineRecordPlacement),
    Extent(DurableExtentRecordPlacement),
}

impl CurrentPhysicalRecordPlacement {
    pub(crate) fn with_route_metadata(self, metadata: SelectedRecordRouteMetadata) -> Self {
        match self {
            Self::Inline(mut value) => {
                value.route_metadata = metadata;
                Self::Inline(value)
            }
            Self::Extent(mut value) => {
                value.route_metadata = metadata;
                Self::Extent(value)
            }
        }
    }

    pub const fn route_metadata(self) -> SelectedRecordRouteMetadata {
        match self {
            Self::Inline(value) => value.route_metadata(),
            Self::Extent(value) => value.route_metadata(),
        }
    }

    pub const fn content_class(self) -> crate::SelectedRecordContentClass {
        self.route_metadata().content_class()
    }

    pub const fn tier_class(self) -> crate::PhysicalTierClass {
        self.route_metadata().tier_class()
    }

    pub const fn record(self) -> PersistedRecordIdentity {
        match self {
            Self::Inline(value) => value.record(),
            Self::Extent(value) => value.record(),
        }
    }

    pub const fn payload_bytes(self) -> u64 {
        match self {
            Self::Inline(value) => value.payload_bytes(),
            Self::Extent(value) => value.payload_bytes(),
        }
    }
}
