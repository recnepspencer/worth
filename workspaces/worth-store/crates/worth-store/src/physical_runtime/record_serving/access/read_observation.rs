use super::super::RecordByteLimit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordReadLimits {
    pub(in crate::physical_runtime::record_serving) maximum_payload: RecordByteLimit,
}

impl RecordReadLimits {
    pub const fn new(maximum_payload: RecordByteLimit) -> Self {
        Self { maximum_payload }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RecordReadObservation {
    pub(in crate::physical_runtime::record_serving) touched_segments: u64,
    pub(in crate::physical_runtime::record_serving) touched_pages: u64,
    pub(in crate::physical_runtime::record_serving) touched_extents: u64,
    pub(in crate::physical_runtime::record_serving) payload_bytes: u64,
    pub(in crate::physical_runtime::record_serving) requested_bytes: u64,
    pub(in crate::physical_runtime::record_serving) transfer_count: u64,
    pub(in crate::physical_runtime::record_serving) peak_transfer_width: u64,
    pub(in crate::physical_runtime::record_serving) explicit_copy_count: u64,
    pub(in crate::physical_runtime::record_serving) copied_bytes: u64,
    pub(in crate::physical_runtime::record_serving) generation_checks: u64,
    pub(in crate::physical_runtime::record_serving) generation_rejections: u64,
    pub(in crate::physical_runtime::record_serving) peak_scratch_bytes: u64,
    pub(in crate::physical_runtime::record_serving) manifest_blocks: u64,
    pub(in crate::physical_runtime::record_serving) manifest_comparisons: u64,
    pub(in crate::physical_runtime::record_serving) manifest_bytes: u64,
    pub(in crate::physical_runtime::record_serving) physical_work_count: u64,
    pub(in crate::physical_runtime::record_serving) first_physical_work:
        Option<crate::physical_runtime::PhysicalWorkIdentity>,
    pub(in crate::physical_runtime::record_serving) last_physical_work:
        Option<crate::physical_runtime::PhysicalWorkIdentity>,
}

impl RecordReadObservation {
    pub const fn touched_segments(self) -> u64 {
        self.touched_segments
    }
    pub const fn touched_pages(self) -> u64 {
        self.touched_pages
    }
    pub const fn touched_extents(self) -> u64 {
        self.touched_extents
    }
    pub const fn payload_bytes(self) -> u64 {
        self.payload_bytes
    }
    pub const fn bytes_requested(self) -> u64 {
        self.requested_bytes
    }
    pub const fn bytes_completed(self) -> u64 {
        self.payload_bytes
    }
    pub const fn transfer_count(self) -> u64 {
        self.transfer_count
    }
    pub const fn peak_transfer_width(self) -> u64 {
        self.peak_transfer_width
    }
    pub const fn explicit_copy_count(self) -> u64 {
        self.explicit_copy_count
    }
    pub const fn copied_bytes(self) -> u64 {
        self.copied_bytes
    }
    pub const fn generation_checks(self) -> u64 {
        self.generation_checks
    }
    pub const fn generation_rejections(self) -> u64 {
        self.generation_rejections
    }
    pub const fn peak_scratch_bytes(self) -> u64 {
        self.peak_scratch_bytes
    }
    pub const fn manifest_blocks(self) -> u64 {
        self.manifest_blocks
    }
    pub const fn manifest_comparisons(self) -> u64 {
        self.manifest_comparisons
    }
    pub const fn manifest_bytes(self) -> u64 {
        self.manifest_bytes
    }
    pub const fn physical_work_count(self) -> u64 {
        self.physical_work_count
    }
    pub const fn first_physical_work(
        self,
    ) -> Option<crate::physical_runtime::PhysicalWorkIdentity> {
        self.first_physical_work
    }
    pub const fn last_physical_work(self) -> Option<crate::physical_runtime::PhysicalWorkIdentity> {
        self.last_physical_work
    }

    pub(in crate::physical_runtime::record_serving) fn observe_manifest(
        &mut self,
        snapshot: super::super::access::manifest_routing::ManifestDiscoveryCounterSnapshot,
    ) {
        self.manifest_blocks = self.manifest_blocks.saturating_add(snapshot.blocks_read());
        self.manifest_comparisons = self
            .manifest_comparisons
            .saturating_add(snapshot.comparisons());
        self.manifest_bytes = self.manifest_bytes.saturating_add(snapshot.bytes_read());
        self.physical_work_count = self
            .physical_work_count
            .saturating_add(snapshot.work_count());
        self.first_physical_work = self.first_physical_work.or(snapshot.first_work());
        self.last_physical_work = snapshot.last_work().or(self.last_physical_work);
    }

    pub(in crate::physical_runtime::record_serving) fn observe_manifest_block(
        &mut self,
        bytes: usize,
    ) {
        self.manifest_blocks = self.manifest_blocks.saturating_add(1);
        self.manifest_bytes = self.manifest_bytes.saturating_add(bytes as u64);
    }

    pub(in crate::physical_runtime::record_serving) fn check_generation(
        &mut self,
        matches: bool,
    ) -> bool {
        self.generation_checks = self.generation_checks.saturating_add(1);
        if !matches {
            self.generation_rejections = self.generation_rejections.saturating_add(1);
        }
        matches
    }

    pub(in crate::physical_runtime::record_serving) fn check_page_identity_generation(
        &mut self,
        matches: bool,
    ) -> bool {
        self.check_generation(matches)
    }

    pub(in crate::physical_runtime::record_serving) fn check_slot_generation(
        &mut self,
        matches: bool,
    ) -> bool {
        self.check_generation(matches)
    }

    pub(in crate::physical_runtime::record_serving) fn observe_transfer(&mut self, bytes: usize) {
        self.transfer_count = self.transfer_count.saturating_add(1);
        self.peak_transfer_width = self.peak_transfer_width.max(bytes as u64);
    }

    pub(in crate::physical_runtime::record_serving) fn observe_physical_work(
        &mut self,
        work: super::super::residency::frame_work_trace::FrameWorkTrace,
    ) {
        self.physical_work_count = self.physical_work_count.saturating_add(work.count());
        self.first_physical_work = self.first_physical_work.or(work.first());
        self.last_physical_work = work.last().or(self.last_physical_work);
    }

    pub(in crate::physical_runtime::record_serving) fn observe_copy(&mut self, bytes: usize) {
        if bytes != 0 {
            self.explicit_copy_count = self.explicit_copy_count.saturating_add(1);
            self.copied_bytes = self.copied_bytes.saturating_add(bytes as u64);
        }
    }
}

/// Failure to open or continue a bounded physical record read.
///
/// Inspect `denial` for the broad class, `observation` for work already
/// observed by the session, and `pressure` for exact physical-pressure
/// evidence when the denial is `RecordReadDenial::PhysicalPressure`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordReadError {
    denial: RecordReadDenial,
    observation: RecordReadObservation,
    pressure: Option<super::super::PhysicalRecordPressureEvidence>,
}

impl RecordReadError {
    pub(in crate::physical_runtime::record_serving) const fn new(
        denial: RecordReadDenial,
        observation: RecordReadObservation,
    ) -> Self {
        Self {
            denial,
            observation,
            pressure: None,
        }
    }

    pub(in crate::physical_runtime::record_serving) const fn from_pressure(
        pressure: super::super::PhysicalRecordPressureEvidence,
        observation: RecordReadObservation,
    ) -> Self {
        Self {
            denial: RecordReadDenial::PhysicalPressure,
            observation,
            pressure: Some(pressure),
        }
    }
    /// Returns the broad Store-facing denial class.
    pub const fn denial(self) -> RecordReadDenial {
        self.denial
    }

    /// Returns record-read progress and physical work observed before failure.
    pub const fn observation(self) -> RecordReadObservation {
        self.observation
    }

    /// Returns exact physical-pressure evidence when pressure caused failure.
    ///
    /// The evidence is descriptive and cannot be used as retry or allocation
    /// authority.
    pub const fn pressure(&self) -> Option<super::super::PhysicalRecordPressureEvidence> {
        self.pressure
    }
}

/// Store-facing reason a bounded physical record read was denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordReadDenial {
    ServingRequiresInspection,
    StoreIdentityMismatch,
    RecordNotFound,
    CallerLimitExceeded,
    AccessLimitExceeded,
    ArtifactUnavailable,
    ArtifactDamaged,
    BackendUnavailable(worth_store_physical_backend::ArtifactTreeFailure),
    PhysicalWork(RecordReadWorkDenial),
    FormatMismatch,
    PhysicalPressure,
    ResidencyUnavailable(super::super::PhysicalRecordResidencyFailure),
    StalePlacement(StalePhysicalRecordPlacement),
}

impl RecordReadDenial {
    pub(in crate::physical_runtime::record_serving) fn from_residency(
        denial: worth_store_buffer_pool::PhysicalResidencyDenial,
    ) -> Self {
        Self::ResidencyUnavailable(denial.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordReadWorkDenial {
    RuntimeReleased,
    InvalidCoordinate,
    SubmissionRejected,
    AdmissionRejected,
    DependencyBlocked,
    SchedulerReservationRejected,
    SchedulerRejected,
    CommandRejected,
    SchedulerSettlementRejected,
    SettlementMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StalePhysicalRecordPlacement {
    SegmentGeneration,
    SegmentMembership,
    PageGeneration,
    PageIdentity,
    SlotGeneration,
    ExtentMembership,
}
