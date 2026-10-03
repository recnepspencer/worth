#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSignalRuntimeIdentity([u8; 16]);

impl PhysicalSignalRuntimeIdentity {
    pub(super) const fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalSignalConstructionFailure {
    ProfileRejected(crate::physical_runtime::PhysicalWorkProfileDenial),
    SchedulerCapabilityRejected(worth_store_io_scheduler::IoSchedulerBackendCapabilityDenial),
    DurabilityStateReopenRejected(super::super::PhysicalDurabilityStateReopenFailure),
    /// Serving could not reserve the capture envelope of the released-drop
    /// custody it would hold; denied before Serving installs any owner.
    ServingCaptureCustodyReservationRejected(
        crate::physical_runtime::ReleaseCertificateCapacityDenial,
    ),
    IdentityEntropyUnavailable,
    WorkerSpawnRejected,
    WorkerReadinessLost,
    CapabilityDeclarationRejected,
    DependencyInitializationRejected,
    ClockBridgeUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalSignalShutdownOutcome {
    Disposed,
    OwnerRevoked,
    DerivedReconciliationPending { pending: usize, overflow: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSignalClockObservation {
    current_tick: u64,
    last_advance_ordinal: u64,
}

impl PhysicalSignalClockObservation {
    pub(super) const fn new(current_tick: u64, last_advance_ordinal: u64) -> Self {
        Self {
            current_tick,
            last_advance_ordinal,
        }
    }
    pub const fn current_tick(self) -> u64 {
        self.current_tick
    }
    pub const fn last_advance_ordinal(self) -> u64 {
        self.last_advance_ordinal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalSignalClockObservationFailure {
    OwnerUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalSignalDeltaApplicationFailure {
    BindingNotInstalled,
    BindingCapabilityMismatch,
    SemanticBasisRejected(crate::physical_runtime::PhysicalWorkAspectDeltaDenial),
    VersionExhausted,
    SignalMutationRejected,
    SignalEvaluationRejected,
    OwnerUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalSignalObservation {
    profile: crate::physical_runtime::PhysicalSignalProfileIdentity,
    graph_owner_count: u8,
    aspect_binding_count: u16,
    locality_owner_count: u16,
    active_locality_count: usize,
    active_graph_node_count: usize,
    active_in_flight_count: u64,
    request_admission_count: u64,
    async_family_count: u8,
    aspect_invalidation_count: u64,
    clock: PhysicalSignalClockObservation,
}

pub(super) struct PhysicalSignalTopologyObservation {
    pub(super) graph_owner_count: u8,
    pub(super) aspect_binding_count: u16,
    pub(super) locality_owner_count: u16,
    pub(super) active_locality_count: usize,
    pub(super) active_graph_node_count: usize,
    pub(super) active_in_flight_count: u64,
    pub(super) request_admission_count: u64,
    pub(super) async_family_count: u8,
    pub(super) aspect_invalidation_count: u64,
}

impl PhysicalSignalObservation {
    pub(super) const fn new(
        profile: crate::physical_runtime::PhysicalSignalProfileIdentity,
        topology: PhysicalSignalTopologyObservation,
        clock: PhysicalSignalClockObservation,
    ) -> Self {
        Self {
            profile,
            graph_owner_count: topology.graph_owner_count,
            aspect_binding_count: topology.aspect_binding_count,
            locality_owner_count: topology.locality_owner_count,
            active_locality_count: topology.active_locality_count,
            active_graph_node_count: topology.active_graph_node_count,
            active_in_flight_count: topology.active_in_flight_count,
            request_admission_count: topology.request_admission_count,
            async_family_count: topology.async_family_count,
            aspect_invalidation_count: topology.aspect_invalidation_count,
            clock,
        }
    }

    pub const fn profile(self) -> crate::physical_runtime::PhysicalSignalProfileIdentity {
        self.profile
    }

    pub const fn aspect_binding_count(self) -> u16 {
        self.aspect_binding_count
    }

    pub const fn graph_owner_count(self) -> u8 {
        self.graph_owner_count
    }

    pub const fn locality_owner_count(self) -> u16 {
        self.locality_owner_count
    }

    pub const fn active_locality_count(self) -> usize {
        self.active_locality_count
    }

    pub const fn active_graph_node_count(self) -> usize {
        self.active_graph_node_count
    }

    pub const fn active_in_flight_count(self) -> u64 {
        self.active_in_flight_count
    }

    pub const fn request_admission_count(self) -> u64 {
        self.request_admission_count
    }

    pub const fn async_family_count(self) -> u8 {
        self.async_family_count
    }

    pub const fn aspect_invalidation_count(self) -> u64 {
        self.aspect_invalidation_count
    }

    pub const fn clock(self) -> PhysicalSignalClockObservation {
        self.clock
    }
}
