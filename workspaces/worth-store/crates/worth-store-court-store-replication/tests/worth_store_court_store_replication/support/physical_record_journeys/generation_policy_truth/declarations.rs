pub(crate) use {std::path::Path};

pub(crate) use {worth_store::physical_runtime::ManifestEntryCapacity, worth_store::physical_runtime::PageFillPercent, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::SegmentPageCount};
pub(crate) use {worth_store_offline_verifier::OfflineDurableManifestWalk, worth_store_offline_verifier::OfflineRecordPlacement};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};
pub(crate) use {worth_store_physical_format::RecordAllocationClass};

pub(crate) use {worth_store_testkit_store::durable_publication, worth_store_testkit_store::physical_record_journeys::media, worth_store_testkit_store::physical_record_journeys::read_record, worth_store_testkit_store::physical_record_journeys::scenario_configuration::dense_configuration, worth_store_testkit_store::physical_record_journeys::success};

pub(crate) fn placement(
    format: worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    segment_pages: u32,
) -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(segment_pages).unwrap())
        .extent_threshold(worth_store::physical_runtime::RecordByteLimit::new(8_000).unwrap())
        .page_fill(worth_store::physical_runtime::PageFillPercent::new(50).unwrap())
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(128).unwrap())
        .admit(format)
        .unwrap()
}

pub(crate) fn decode_root(
    root: &Path,
    generation: u64,
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> OfflineDurableManifestWalk {
    worth_store_testkit_store::physical_record_journeys::manifest_fixture::decode_routing_tree(root, generation, format, 128)
}

pub(crate) fn inline(
    root: &OfflineDurableManifestWalk,
    record: worth_store::physical_runtime::PhysicalRecordId,
) -> InlinePlacement {
    match root
        .placements()
        .iter()
        .find(|entry| {
            entry.record().allocation_epoch() == record.allocation_epoch()
                && entry.record().ordinal() == record.ordinal()
        })
        .copied()
        .unwrap()
    {
        worth_store_offline_verifier::OfflineRecordPlacement::Inline {
            segment,
            page,
            segment_generation,
            page_generation,
            segment_page_capacity,
            ..
        } => InlinePlacement {
            segment,
            page,
            segment_generation,
            page_generation,
            segment_page_capacity,
        },
        worth_store_offline_verifier::OfflineRecordPlacement::Extent { .. } => panic!("inline record routed to extent"),
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InlinePlacement {
    pub(crate) segment: u64,
    pub(crate) page: u64,
    pub(crate) segment_generation: u64,
    pub(crate) page_generation: u64,
    pub(crate) segment_page_capacity: u32,
}

impl InlinePlacement {
    pub(crate) const fn segment(self) -> u64 {
        self.segment
    }
    pub(crate) const fn segment_generation(self) -> u64 {
        self.segment_generation
    }
    pub(crate) const fn page_generation(self) -> u64 {
        self.page_generation
    }
    pub(crate) const fn page_cell(self) -> (u64, u64, u64) {
        (self.segment, self.page, self.page_generation)
    }
    pub(crate) const fn segment_page_capacity(self) -> u32 {
        self.segment_page_capacity
    }
}