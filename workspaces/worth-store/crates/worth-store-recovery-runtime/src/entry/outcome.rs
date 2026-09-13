use worth_store::physical_runtime::{
    RecoveredPhysicalRuntimeConstructionDenial, RecoveryDiscoveryFailure,
    StoreRecoveryBindingSampleDenial,
};
use worth_store_physical_format::{store_namespace::StableStoreIdentity, RecordArtifactFile};
use worth_store_recovery_physics::{
    OperationReconciliationDenial, PhysicalRedoPlanningDenial, PhysicalRedoTargetIdentity,
    RecoveryPlanCostDenial, RecoveryPlanningCounters,
};

use crate::progression::PhysicalRecoveryDiscoveryCounters;

#[derive(Debug)]
pub enum PhysicalRecoveryOutcome {
    Recovered(crate::handoff::RecoveredPhysicalRuntimeHandoff),
    Refused(PhysicalRecoveryRefusal),
    Blocked(PhysicalRecoveryBlock),
    PublicationIndeterminate(PhysicalRecoveryPublicationIndeterminate),
}

impl PhysicalRecoveryOutcome {
    pub(crate) fn with_block_integrity_observations(
        self,
        observations: super::PhysicalRecoveryIntegrityObservations,
    ) -> Self {
        match self {
            Self::Blocked(blocked) => {
                Self::Blocked(blocked.with_integrity_observations(observations))
            }
            _ => unreachable!("planning denial construction always yields a blocked outcome"),
        }
    }
}

#[derive(Debug)]
pub struct PhysicalRecoveryPublicationIndeterminate {
    store: StableStoreIdentity,
    session: super::PhysicalRecoverySessionIdentity,
    counters: super::PhysicalRecoveryPublicationCounters,
    settlement: super::PhysicalRecoveryPublicationSettlementLedger,
    root_protocol_denials: Vec<super::PhysicalRecoverySourceDenial>,
    root_protocol_counters: super::PhysicalRecoveryRootProtocolCounters,
    integrity_observations: super::PhysicalRecoveryIntegrityObservations,
    reopen: Option<super::PhysicalRecoveryReopenFailure>,
    handoff: Option<RecoveredPhysicalRuntimeConstructionDenial>,
    recovery_effects: u64,
    integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

mod refusal;
pub use refusal::{PhysicalRecoveryRefusal, PhysicalRecoveryRefusalKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryBlockKind {
    DiscoveryLimit,
    MediaObservation,
    RootProtocol,
    Checkpoint,
    WalInventory,
    SourceSelection,
    BindingFreshness,
    PageAdmission,
    OperationReconciliation,
    RedoPlanning,
    Staging,
    Publication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryLimitDimension {
    SelectorCandidates,
    ManifestBytes,
    ManifestEntries,
    WalSegments,
    WalFrames,
    WalBytes,
    DistinctPagesAndExtents,
    ObservationBytes,
    OperationBindings,
    RedoTargets,
    RedoBytes,
    StagingBytes,
    RecoveryMemoryBytes,
    DirtyFrames,
    PublicationEffects,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryLimitFailure {
    pub dimension: PhysicalRecoveryLimitDimension,
    pub observed: u64,
    pub admitted: u64,
}

#[derive(Debug, Default)]
pub struct PhysicalRecoveryBlockEvidence {
    pub counters: PhysicalRecoveryDiscoveryCounters,
    pub planning_counters: Option<RecoveryPlanningCounters>,
    pub root_protocol_counters: Option<super::PhysicalRecoveryRootProtocolCounters>,
    pub limit: Option<PhysicalRecoveryLimitFailure>,
    pub artifact: Option<String>,
    pub source_generation: Option<u64>,
    pub lsn: Option<u64>,
    pub source_denials: Vec<super::PhysicalRecoverySourceDenial>,
    pub integrity_observations: super::PhysicalRecoveryIntegrityObservations,
    pub planning_denial: Option<PhysicalRecoveryPlanningDenial>,
    pub staging_counters: Option<super::PhysicalRecoveryStagingCounters>,
    pub staging_denial: Option<super::PhysicalRecoveryStagingDenial>,
    pub staging_settlements: Option<super::PhysicalRecoveryStagingSettlementLedger>,
    pub publication_counters: Option<super::PhysicalRecoveryPublicationCounters>,
    pub publication_denial: Option<super::PhysicalRecoveryPublicationDenial>,
    pub publication_settlements: Option<super::PhysicalRecoveryPublicationSettlementLedger>,
    pub(crate) integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
}

impl PhysicalRecoveryBlockEvidence {
    pub const fn integrity_observation_count(&self) -> u64 {
        self.integrity_trace.counters().attempted
    }

    pub const fn integrity_counters(&self) -> crate::PhysicalRecoveryIntegrityCounters {
        self.integrity_trace.counters()
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.integrity_trace.observations()
    }
}

impl PhysicalRecoveryPublicationIndeterminate {
    pub(crate) const fn new(
        store: StableStoreIdentity,
        session: super::PhysicalRecoverySessionIdentity,
        counters: super::PhysicalRecoveryPublicationCounters,
        settlement: super::PhysicalRecoveryPublicationSettlementLedger,
        root_protocol_denials: Vec<super::PhysicalRecoverySourceDenial>,
        root_protocol_counters: super::PhysicalRecoveryRootProtocolCounters,
        recovery_effects: u64,
    ) -> Self {
        Self {
            store,
            session,
            counters,
            settlement,
            root_protocol_denials,
            root_protocol_counters,
            integrity_observations: super::PhysicalRecoveryIntegrityObservations::new(Vec::new()),
            reopen: None,
            handoff: None,
            recovery_effects,
            integrity_trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace::new(),
        }
    }
    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }
    pub const fn session_identity(&self) -> super::PhysicalRecoverySessionIdentity {
        self.session
    }
    pub const fn counters(&self) -> super::PhysicalRecoveryPublicationCounters {
        self.counters
    }
    pub const fn settlement(&self) -> &super::PhysicalRecoveryPublicationSettlementLedger {
        &self.settlement
    }
    pub fn root_protocol_denials(&self) -> &[super::PhysicalRecoverySourceDenial] {
        &self.root_protocol_denials
    }
    pub const fn root_protocol_counters(&self) -> super::PhysicalRecoveryRootProtocolCounters {
        self.root_protocol_counters
    }
    pub const fn recovery_effects(&self) -> u64 {
        self.recovery_effects
    }
    pub const fn integrity_observation_count(&self) -> u64 {
        self.integrity_trace.counters().attempted
    }

    pub const fn integrity_counters(&self) -> crate::PhysicalRecoveryIntegrityCounters {
        self.integrity_trace.counters()
    }

    pub fn integrity_observations(&self) -> &[crate::PhysicalRecoveryIntegrityObservation] {
        self.integrity_trace.observations()
    }

    pub(crate) fn with_integrity_trace(
        mut self,
        trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        self.integrity_trace = trace;
        self
    }

    pub(crate) fn with_integrity_observations(
        mut self,
        observations: super::PhysicalRecoveryIntegrityObservations,
    ) -> Self {
        self.integrity_observations = observations;
        self
    }

    pub const fn wal_integrity_observations(
        &self,
    ) -> &super::PhysicalRecoveryIntegrityObservations {
        &self.integrity_observations
    }

    pub(crate) fn with_reopen_failure(
        mut self,
        failure: super::PhysicalRecoveryReopenFailure,
    ) -> Self {
        self.reopen = Some(failure);
        self
    }

    pub const fn reopen_failure(&self) -> Option<&super::PhysicalRecoveryReopenFailure> {
        self.reopen.as_ref()
    }

    pub(crate) fn with_handoff_failure(
        mut self,
        failure: RecoveredPhysicalRuntimeConstructionDenial,
    ) -> Self {
        self.handoff = Some(failure);
        self
    }

    pub const fn handoff_failure(&self) -> Option<RecoveredPhysicalRuntimeConstructionDenial> {
        self.handoff
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryPlanningDenial {
    BindingFreshness(StoreRecoveryBindingSampleDenial),
    OperationReconciliation(OperationReconciliationDenial),
    Redo(PhysicalRedoPlanningDenial),
    Page(PhysicalRecoveryPageAdmissionDenial),
    SuccessorCandidate(super::PhysicalRecoverySuccessorCandidateDenial),
    Cost(RecoveryPlanCostDenial),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalRecoveryPageAdmissionDenial {
    Media {
        target: Option<PhysicalRedoTargetIdentity>,
        failure: RecoveryDiscoveryFailure,
    },
    MissingArtifact {
        target: Option<PhysicalRedoTargetIdentity>,
        artifact: RecordArtifactFile,
    },
    InvalidManifest {
        target: Option<PhysicalRedoTargetIdentity>,
        artifact: RecordArtifactFile,
    },
    Integrity {
        artifact: RecordArtifactFile,
        denial: super::PhysicalRecoveryRootProtocolDenial,
    },
    InvalidTarget(PhysicalRedoTargetIdentity),
    InvalidPage(PhysicalRedoTargetIdentity),
    ManifestEntryLimit,
    ObservationByteLimit,
}

#[derive(Debug)]
pub struct PhysicalRecoveryBlock {
    pub kind: PhysicalRecoveryBlockKind,
    store: StableStoreIdentity,
    session: super::PhysicalRecoverySessionIdentity,
    evidence: PhysicalRecoveryBlockEvidence,
    recovery_effects: u64,
}

impl PhysicalRecoveryBlock {
    pub(crate) const fn new(
        kind: PhysicalRecoveryBlockKind,
        store: StableStoreIdentity,
        session: super::PhysicalRecoverySessionIdentity,
        evidence: PhysicalRecoveryBlockEvidence,
        recovery_effects: u64,
    ) -> Self {
        Self {
            kind,
            store,
            session,
            evidence,
            recovery_effects,
        }
    }

    pub const fn store_identity(&self) -> StableStoreIdentity {
        self.store
    }

    pub const fn session_identity(&self) -> super::PhysicalRecoverySessionIdentity {
        self.session
    }

    pub const fn evidence(&self) -> &PhysicalRecoveryBlockEvidence {
        &self.evidence
    }

    pub const fn recovery_effects(&self) -> u64 {
        self.recovery_effects
    }

    fn with_integrity_observations(
        mut self,
        observations: super::PhysicalRecoveryIntegrityObservations,
    ) -> Self {
        self.evidence.integrity_observations = observations;
        self
    }
}

impl PhysicalRecoveryOutcome {
    pub(crate) fn with_integrity_trace(
        self,
        trace: crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Self {
        match self {
            Self::Blocked(mut block) => {
                block.evidence.integrity_trace.append(trace);
                Self::Blocked(block)
            }
            Self::Refused(refusal) => Self::Refused(refusal.with_integrity_trace(trace)),
            Self::PublicationIndeterminate(indeterminate) => {
                Self::PublicationIndeterminate(indeterminate.with_integrity_trace(trace))
            }
            Self::Recovered(recovered) => Self::Recovered(recovered),
        }
    }
}
