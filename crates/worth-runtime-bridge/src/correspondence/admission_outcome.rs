#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeCorrespondenceDenialKind {
    InvalidPortableDependency,
    ProjectionMaskNotAdmitted,
    EmptyTargetSet,
    DuplicateTarget,
    MissingMapping,
    MappingSemanticMismatch,
    AmbiguousMapping,
    CapacityExhausted,
    SlotAlreadyOwned,
    SharedSlotRequiresDeclaredWidening,
    MissingOrStaleSignalNode,
    SignalNodeContractMismatch,
    MixedGraphTargetSet,
    PortableDependencyNotOwnedByOperation,
    StaleSourceInstallation,
    GraphParticipationNotOwnedByOperation,
    AuthoritativeSourceMismatch,
    CommittedPatchRequestMismatch,
    PortableDependencyNotInstalled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeCorrespondenceDenial {
    kind: BridgeCorrespondenceDenialKind,
    counters: CorrespondenceAdmissionCounters,
}

impl BridgeCorrespondenceDenial {
    pub(crate) const fn new(
        kind: BridgeCorrespondenceDenialKind,
        counters: CorrespondenceAdmissionCounters,
    ) -> Self {
        Self { kind, counters }
    }

    pub const fn without_admission(kind: BridgeCorrespondenceDenialKind) -> Self {
        Self::new(kind, CorrespondenceAdmissionCounters::zero())
    }

    pub const fn kind(self) -> BridgeCorrespondenceDenialKind {
        self.kind
    }

    pub const fn counters(self) -> CorrespondenceAdmissionCounters {
        self.counters
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CorrespondenceAdmissionCounters {
    pub(crate) semantic_dependency_lookups: usize,
    pub(crate) provided_registration_reads: usize,
    pub(crate) registered_targets_materialized: usize,
    pub(crate) source_profile_cache_reads: usize,
    pub(crate) allocation_registry_lock_attempts: usize,
    pub(crate) mapping_lookups: usize,
    pub(crate) allocation_owner_lookups: usize,
    pub(crate) allocation_keys_examined: usize,
    pub(crate) exact_matches: usize,
    pub(crate) widened_matches: usize,
    pub(crate) entity_widened_matches: usize,
    pub(crate) aspect_widened_matches: usize,
    pub(crate) surface_widened_matches: usize,
    pub(crate) partition_widened_matches: usize,
    pub(crate) field_to_whole_source_admissions: usize,
    pub(crate) aspect_to_entity_source_admissions: usize,
    pub(crate) surface_broadening_source_admissions: usize,
    pub(crate) opaque_source_admissions: usize,
    pub(crate) signal_node_admissions: usize,
    pub(crate) targets_admitted: usize,
    pub(crate) capacity_denials: usize,
    pub(crate) authoritative_records_committed: usize,
    pub(crate) failed_admissions: usize,
}

impl CorrespondenceAdmissionCounters {
    pub const fn zero() -> Self {
        Self {
            semantic_dependency_lookups: 0,
            provided_registration_reads: 0,
            registered_targets_materialized: 0,
            source_profile_cache_reads: 0,
            allocation_registry_lock_attempts: 0,
            mapping_lookups: 0,
            allocation_owner_lookups: 0,
            allocation_keys_examined: 0,
            exact_matches: 0,
            widened_matches: 0,
            entity_widened_matches: 0,
            aspect_widened_matches: 0,
            surface_widened_matches: 0,
            partition_widened_matches: 0,
            field_to_whole_source_admissions: 0,
            aspect_to_entity_source_admissions: 0,
            surface_broadening_source_admissions: 0,
            opaque_source_admissions: 0,
            signal_node_admissions: 0,
            targets_admitted: 0,
            capacity_denials: 0,
            authoritative_records_committed: 0,
            failed_admissions: 0,
        }
    }

    pub const fn semantic_dependency_lookups(self) -> usize {
        self.semantic_dependency_lookups
    }

    pub const fn provided_registration_reads(self) -> usize {
        self.provided_registration_reads
    }

    pub const fn mapping_lookups(self) -> usize {
        self.mapping_lookups
    }

    pub const fn registered_targets_materialized(self) -> usize {
        self.registered_targets_materialized
    }

    pub const fn source_profile_cache_reads(self) -> usize {
        self.source_profile_cache_reads
    }

    pub const fn allocation_registry_lock_attempts(self) -> usize {
        self.allocation_registry_lock_attempts
    }

    pub const fn allocation_owner_lookups(self) -> usize {
        self.allocation_owner_lookups
    }

    pub const fn allocation_keys_examined(self) -> usize {
        self.allocation_keys_examined
    }

    pub const fn exact_matches(self) -> usize {
        self.exact_matches
    }

    pub const fn widened_matches(self) -> usize {
        self.widened_matches
    }

    pub const fn entity_widened_matches(self) -> usize {
        self.entity_widened_matches
    }

    pub const fn aspect_widened_matches(self) -> usize {
        self.aspect_widened_matches
    }

    pub const fn surface_widened_matches(self) -> usize {
        self.surface_widened_matches
    }

    pub const fn partition_widened_matches(self) -> usize {
        self.partition_widened_matches
    }

    pub const fn field_to_whole_source_admissions(self) -> usize {
        self.field_to_whole_source_admissions
    }

    pub const fn aspect_to_entity_source_admissions(self) -> usize {
        self.aspect_to_entity_source_admissions
    }

    pub const fn surface_broadening_source_admissions(self) -> usize {
        self.surface_broadening_source_admissions
    }

    pub const fn opaque_source_admissions(self) -> usize {
        self.opaque_source_admissions
    }

    pub const fn signal_node_admissions(self) -> usize {
        self.signal_node_admissions
    }

    pub const fn targets_admitted(self) -> usize {
        self.targets_admitted
    }

    pub const fn capacity_denials(self) -> usize {
        self.capacity_denials
    }

    pub const fn authoritative_records_committed(self) -> usize {
        self.authoritative_records_committed
    }

    pub const fn failed_admissions(self) -> usize {
        self.failed_admissions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeCorrespondenceDeferred {
    GraphMutationInProgress,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeCorrespondenceStale {
    BridgeRuntimeBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeCorrespondenceRebindRequired {
    SignalGraphGeneration,
    SignalGraphLoweringOwner,
    AllocationSourceSet,
    ConditionalBasis,
    ConditionalDefinitionReadmission,
    ConditionalDefinitionMismatch,
    ConditionalTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeCorrespondenceAdmissionFailure {
    LockPoisoned,
    SourceLoadFailed,
    SignalMutationFailed,
}
