use super::super::arena::{ArenaEvacuationThreshold, ExtentArenaCapacity};
use super::super::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PageFillPercent,
    PhysicalRecordFormatDeclaration, RecordByteLimit, SegmentPageCount,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecordPlacementPolicy {
    segment_pages: SegmentPageCount,
    extent_threshold: RecordByteLimit,
    page_fill: PageFillPercent,
    manifest_capacity: ManifestEntryCapacity,
    arena_capacity: ExtentArenaCapacity,
    arena_evacuation: ArenaEvacuationThreshold,
    arena_index_bytes: RecordByteLimit,
}

#[derive(Debug, Default)]
pub struct PhysicalRecordPlacementPolicyBuilder {
    segment_pages: Option<SegmentPageCount>,
    extent_threshold: Option<RecordByteLimit>,
    page_fill: Option<PageFillPercent>,
    manifest_capacity: Option<ManifestEntryCapacity>,
    arena_capacity: Option<ExtentArenaCapacity>,
    arena_evacuation: Option<ArenaEvacuationThreshold>,
    arena_index_bytes: Option<RecordByteLimit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecordPlacementPolicyDenial {
    ExtentThresholdCannotFitPage,
    ManifestCapacityCannotBranch,
    ManifestCapacityCannotFitPage,
    SegmentManifestCannotFitPage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedRecordPlacementPolicy {
    policy: PhysicalRecordPlacementPolicy,
    format: PhysicalRecordFormatDeclaration,
}

impl PhysicalRecordPlacementPolicy {
    pub fn builder() -> PhysicalRecordPlacementPolicyBuilder {
        PhysicalRecordPlacementPolicyBuilder::default()
    }
}

impl PhysicalRecordPlacementPolicyBuilder {
    /// Reserved resident capacity for free-range indexes and in-flight claims.
    pub fn arena_index_bytes(mut self, bytes: RecordByteLimit) -> Self {
        self.arena_index_bytes = Some(bytes);
        self
    }
    pub fn arena_capacity(mut self, capacity: ExtentArenaCapacity) -> Self {
        self.arena_capacity = Some(capacity);
        self
    }

    pub fn arena_evacuation(mut self, threshold: ArenaEvacuationThreshold) -> Self {
        self.arena_evacuation = Some(threshold);
        self
    }

    pub fn segment_pages(mut self, pages: SegmentPageCount) -> Self {
        self.segment_pages = Some(pages);
        self
    }

    pub fn extent_threshold(mut self, bytes: RecordByteLimit) -> Self {
        self.extent_threshold = Some(bytes);
        self
    }

    pub fn page_fill(mut self, percent: PageFillPercent) -> Self {
        self.page_fill = Some(percent);
        self
    }

    pub fn manifest_capacity(mut self, entries: ManifestEntryCapacity) -> Self {
        self.manifest_capacity = Some(entries);
        self
    }

    pub fn admit(
        self,
        format: AdmittedPhysicalRecordFormat,
    ) -> Result<AdmittedRecordPlacementPolicy, PhysicalRecordPlacementPolicyDenial> {
        let page_bytes = format.declaration().page_size().bytes();
        let policy = PhysicalRecordPlacementPolicy {
            arena_index_bytes: self.arena_index_bytes.unwrap_or(RecordByteLimit(64 * 1024)),
            arena_capacity: self.arena_capacity.unwrap_or(ExtentArenaCapacity::DEFAULT),
            arena_evacuation: self
                .arena_evacuation
                .unwrap_or(ArenaEvacuationThreshold::DEFAULT),
            segment_pages: self.segment_pages.unwrap_or(SegmentPageCount(
                worth_store_physical_format::maximum_segment_manifest_pages(format.declaration()),
            )),
            extent_threshold: self
                .extent_threshold
                .unwrap_or(RecordByteLimit(page_bytes / 2)),
            page_fill: self.page_fill.unwrap_or(PageFillPercent(90)),
            manifest_capacity: self.manifest_capacity.unwrap_or(ManifestEntryCapacity(
                worth_store_physical_format::maximum_current_root_entries(format.declaration()),
            )),
        };
        if policy.extent_threshold.get() >= page_bytes {
            return Err(PhysicalRecordPlacementPolicyDenial::ExtentThresholdCannotFitPage);
        }
        if !super::policy_units::manifest_capacity_can_branch(policy.manifest_capacity.get()) {
            return Err(PhysicalRecordPlacementPolicyDenial::ManifestCapacityCannotBranch);
        }
        if policy.manifest_capacity.get()
            > worth_store_physical_format::maximum_current_root_entries(format.declaration())
        {
            return Err(PhysicalRecordPlacementPolicyDenial::ManifestCapacityCannotFitPage);
        }
        if policy.segment_pages.get()
            > worth_store_physical_format::maximum_segment_manifest_pages(format.declaration())
        {
            return Err(PhysicalRecordPlacementPolicyDenial::SegmentManifestCannotFitPage);
        }
        Ok(AdmittedRecordPlacementPolicy {
            policy,
            format: format.declaration(),
        })
    }
}

impl AdmittedRecordPlacementPolicy {
    pub const fn arena_index_bytes(self) -> RecordByteLimit {
        self.policy.arena_index_bytes
    }
    pub const fn arena_capacity(self) -> ExtentArenaCapacity {
        self.policy.arena_capacity
    }

    pub const fn arena_evacuation(self) -> ArenaEvacuationThreshold {
        self.policy.arena_evacuation
    }

    pub const fn segment_pages(self) -> SegmentPageCount {
        self.policy.segment_pages
    }

    pub const fn extent_threshold(self) -> RecordByteLimit {
        self.policy.extent_threshold
    }

    pub const fn page_fill(self) -> PageFillPercent {
        self.policy.page_fill
    }

    pub const fn manifest_capacity(self) -> ManifestEntryCapacity {
        self.policy.manifest_capacity
    }

    pub(in crate::physical_runtime::record_serving) fn admits(
        self,
        format: AdmittedPhysicalRecordFormat,
    ) -> bool {
        self.format == format.declaration()
    }
}
