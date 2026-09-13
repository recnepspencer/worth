use worth_store_physical_integrity::{
    PhysicalIntegrityObservationCounters, PhysicalIntegrityObservationOutcome,
    PhysicalQuarantineObservation,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalIntegrityScrubCounters {
    pub declared_targets: u64,
    pub completed_windows: u64,
    pub acquired_bytes: u64,
    pub validated_windows: u64,
    pub damaged_windows: u64,
    pub indeterminate_windows: u64,
    pub unknown_windows: u64,
    pub unsupported_windows: u64,
    pub deferred_windows: u64,
    pub peak_allocation_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum PhysicalIntegrityScrubDeferral {
    AnotherWindowActive,
    Allocation,
    SchedulerOrDependency(
        crate::physical_runtime::record_serving::PhysicalIntegrityScrubReadDeferral,
    ),
}

#[derive(Debug, Clone, Copy)]
pub struct PhysicalIntegrityScrubWindowObservation {
    pub selector_identity: Option<worth_store_physical_format::RootSelectorIdentity>,
    pub ordinal: u64,
    pub outcome: PhysicalIntegrityObservationOutcome,
    pub validation_counters: PhysicalIntegrityObservationCounters,
    pub quarantine: Option<PhysicalQuarantineObservation>,
    pub counters: PhysicalIntegrityScrubCounters,
}

#[derive(Debug, Clone, Copy)]
pub enum ManagedPhysicalIntegrityScrubProgress {
    WindowInspected(PhysicalIntegrityScrubWindowObservation),
    Deferred(PhysicalIntegrityScrubDeferral),
    Paused,
    Completed(PhysicalIntegrityScrubCounters),
    Indeterminate(PhysicalIntegrityScrubCounters),
    Cancelled(PhysicalIntegrityScrubCounters),
    Closed(PhysicalIntegrityScrubCounters),
    StaleRuntimeGeneration(PhysicalIntegrityScrubCounters),
    DeadlineExceeded(PhysicalIntegrityScrubCounters),
}
