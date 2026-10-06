use std::collections::BTreeMap;
use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, CurrentPhysicalRecordPlacement,
    DerivedFamilyRootDirectoryBinding, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    DurableRootSelector, IndexedThroughBlobPublication, PersistedPhysicalRecoveryRootState,
    PersistedRecordIdentity, PhysicalCheckpointIdentity, RecordArtifactFile,
    RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{
    HistoricalConsumedOperationSet, ImmutablePhysicalRedoPlan, PhysicalRedoDecisionKind,
    PhysicalRedoDecisionPrior, PhysicalRedoTarget, PhysicalRedoTargetIdentity,
    PhysicalSourceSelection, ReconciledOperationFates, RecoveryPageObservation,
};

type SelectedRootTopologyEntry = (
    worth_store_physical_format::ManifestBlockReference,
    worth_store_physical_format::PhysicalRootRoutingBlock,
);

mod base_image_accessors;
mod command;
mod derivation;
mod frame_identity;
mod identity;
mod publication_accessors;
mod publication_candidate;
mod publication_types;
mod resident_storage;
mod source_inventory;
mod staging_cost;

use super::{PlanningMemoryDenial, PlanningResidentAllowance};
pub(crate) use derivation::{derive_execution_basis, requires_successor_candidate};
pub(crate) use publication_candidate::verified_historical_release_transition;
pub(crate) use publication_candidate::CandidateMaterializationCost;
pub(crate) use publication_types::RecoveryReleaseTopologyProof;
pub use publication_types::{RecoveryPublicationAction, RecoveryPublicationCandidateArtifact};
pub(crate) use source_inventory::{
    RecoveryObservedCandidateArtifact, RecoveryObservedSuccessorCandidate,
    RecoverySelectedSegmentPage, RecoverySelectedSourceInventory,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExecutionBasisDenial {
    PublicationCandidateAllocation {
        requested_bytes: u64,
        cause: std::collections::TryReserveError,
    },
    ImageAllocation {
        requested_bytes: u64,
        cause: std::collections::TryReserveError,
    },
    RecoveryMemoryBytes {
        observed: u64,
    },
    StagingBytes {
        observed: u64,
    },
    DirtyFrames {
        observed: u64,
    },
    SuccessorCandidate(crate::entry::PhysicalRecoverySuccessorCandidateDenial),
    RootProtocol {
        artifact: crate::entry::PhysicalRecoveryRootProtocolArtifact,
        denial: crate::entry::PhysicalRecoveryRootProtocolDenial,
        counters: crate::entry::PhysicalRecoveryRootProtocolCounters,
    },
    Invalid,
}

impl From<PlanningMemoryDenial> for ExecutionBasisDenial {
    fn from(denial: PlanningMemoryDenial) -> Self {
        match denial {
            PlanningMemoryDenial::RecoveryMemoryBytes { observed } => {
                Self::RecoveryMemoryBytes { observed }
            }
            PlanningMemoryDenial::Allocation {
                requested_bytes,
                cause,
            } => Self::ImageAllocation {
                requested_bytes,
                cause,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryStagingLayoutPlan {
    source_generation: u64,
    staging_generation: u64,
    base: RecoveryBaseImagePlan,
    actions: Box<[RecoveryStagingAction]>,
    commands: Box<[RecoveryStagingCommandPlan]>,
    source_copies: Box<[worth_store_physical_format::PersistedExtentCopyRecipe]>,
    allocated_targets: Box<[PhysicalRedoTargetIdentity]>,
    allocated_bytes: u64,
    write_bytes: u64,
}

/// One complete, immutable artifact construction fixed before Phase 5.
///
/// The bytes include every frame needed by the destination artifact, not only
/// the frames whose logical redo decision was `Apply`. Phase 5 may schedule
/// and settle this command, but it may not regroup or reconstruct it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryStagingCommandPlan {
    ordinal: u64,
    artifact: RecordArtifactFile,
    offset: u64,
    bytes: Box<[u8]>,
    payload_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryBaseImagePlan {
    selected_selector: DurableRootSelector,
    selected_root: DurablePhysicalRootManifest,
    latest_blob_publication: Option<IndexedThroughBlobPublication>,
    latest_blob_quarantine: Option<PersistedRecordIdentity>,
    tier_epoch_anchor: Option<[u8; 32]>,
    derived_family_directory: Option<DerivedFamilyRootDirectoryBinding>,
    selected_root_topology: Box<[SelectedRootTopologyEntry]>,
    destination_generation: u64,
    actions: Box<[RecoveryBaseImageAction]>,
    segment_updates: Box<[RecoverySegmentRoutingAction]>,
    manifests: Box<[RecoveryPayloadManifestAction]>,
    root_states: Box<[PersistedPhysicalRecoveryRootState]>,
    release_head_replay: Option<crate::progression::PendingReleaseReplay>,
    source_artifacts: Box<[RecordArtifactFile]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryBaseImageAction {
    ReuseImmutableSelectedPlacement {
        ordinal: u64,
        placement: CurrentPhysicalRecordPlacement,
    },
    ProjectRecoveryPlacement {
        ordinal: u64,
        placement: CurrentPhysicalRecordPlacement,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoverySegmentRoutingAction {
    ordinal: u64,
    update: RecordSegmentPageManifestEntry,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPayloadManifestAction {
    ordinal: u64,
    coordinate: worth_store_physical_format::RecordFrameCoordinate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryStagingAction {
    ordinal: u64,
    steps: Box<[RecoveryStagingRedoStep]>,
    source: PhysicalRedoTarget,
    destination_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryStagingRedoStep {
    operation: [u8; 32],
    record_index: u64,
    target_index: u64,
    record_lsn: u64,
    prior: RecoveryPageObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPublicationPlan {
    store: StableStoreIdentity,
    checkpoint: Option<PhysicalCheckpointIdentity>,
    source_generation: u64,
    staging_generation: u64,
    actions: Box<[RecoveryPublicationAction]>,
    plan_identity: [u8; 32],
    root_protocol: worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan,
    current_selector: worth_store_physical_format::DurableRootSelector,
    recovered_root: DurablePhysicalRootManifest,
    referenced_artifacts: Box<[RecordArtifactFile]>,
    candidates: Box<[RecoveryPublicationCandidateArtifact]>,
    created_artifacts: Box<[RecordArtifactFile]>,
    release_topology: Option<RecoveryReleaseTopologyProof>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPublicationExpectation {
    store: StableStoreIdentity,
    checkpoint: Option<PhysicalCheckpointIdentity>,
    source_generation: u64,
    staging_generation: u64,
    plan_identity: [u8; 32],
    root_protocol: worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan,
    current_selector: worth_store_physical_format::DurableRootSelector,
    recovered_root: DurablePhysicalRootManifest,
    referenced_artifacts: Box<[RecordArtifactFile]>,
    created_artifacts: Box<[RecordArtifactFile]>,
    release_topology: Option<RecoveryReleaseTopologyProof>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryQuiescencePlan {
    staging_commands: u64,
    publication_commands: u64,
    expected_live_commands_after_close: u64,
    expected_live_media_handles_after_close: u64,
}

impl RecoveryStagingLayoutPlan {
    pub(crate) fn source_copies(
        &self,
    ) -> &[worth_store_physical_format::PersistedExtentCopyRecipe] {
        &self.source_copies
    }
    pub(crate) fn materialization_count(&self) -> u64 {
        self.commands.len() as u64
            + self
                .source_copies
                .iter()
                .map(|copy| u64::from(copy.intent().chunk_count()) + 1)
                .sum::<u64>()
    }
    pub(crate) fn scheduler_command_count(&self) -> u64 {
        self.materialization_count() * 2
            + self
                .source_copies
                .iter()
                .map(|copy| u64::from(copy.intent().chunk_count()) + 1)
                .sum::<u64>()
    }
    pub const fn source_generation(&self) -> u64 {
        self.source_generation
    }
    pub const fn staging_generation(&self) -> u64 {
        self.staging_generation
    }
    pub const fn base_image(&self) -> &RecoveryBaseImagePlan {
        &self.base
    }
    pub fn actions(&self) -> &[RecoveryStagingAction] {
        &self.actions
    }
    pub fn commands(&self) -> &[RecoveryStagingCommandPlan] {
        &self.commands
    }
    pub fn allocated_targets(&self) -> &[PhysicalRedoTargetIdentity] {
        &self.allocated_targets
    }
    pub const fn allocated_bytes(&self) -> u64 {
        self.allocated_bytes
    }
    pub const fn write_bytes(&self) -> u64 {
        self.write_bytes
    }
    pub fn dirty_frames(&self) -> u64 {
        self.allocated_targets.len() as u64
    }

    pub(crate) fn into_base_image(self) -> RecoveryBaseImagePlan {
        self.base
    }
}

impl RecoveryStagingCommandPlan {
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.artifact
    }
    pub const fn offset(&self) -> u64 {
        self.offset
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn byte_count(&self) -> u64 {
        self.bytes.len() as u64
    }
    pub const fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
}

impl RecoveryBaseImageAction {
    pub const fn ordinal(self) -> u64 {
        match self {
            Self::ReuseImmutableSelectedPlacement { ordinal, .. }
            | Self::ProjectRecoveryPlacement { ordinal, .. } => ordinal,
        }
    }
    pub const fn placement(self) -> CurrentPhysicalRecordPlacement {
        match self {
            Self::ReuseImmutableSelectedPlacement { placement, .. }
            | Self::ProjectRecoveryPlacement { placement, .. } => placement,
        }
    }
    pub const fn is_projected(self) -> bool {
        matches!(self, Self::ProjectRecoveryPlacement { .. })
    }
}

impl RecoverySegmentRoutingAction {
    pub const fn ordinal(self) -> u64 {
        self.ordinal
    }
    pub const fn update(self) -> RecordSegmentPageManifestEntry {
        self.update
    }
}

impl RecoveryPayloadManifestAction {
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }
    pub const fn artifact(&self) -> RecordArtifactFile {
        self.coordinate.artifact()
    }
    pub const fn coordinate(&self) -> worth_store_physical_format::RecordFrameCoordinate {
        self.coordinate
    }
}

impl RecoveryStagingAction {
    pub const fn ordinal(&self) -> u64 {
        self.ordinal
    }
    pub fn steps(&self) -> &[RecoveryStagingRedoStep] {
        &self.steps
    }
    pub const fn source(&self) -> &PhysicalRedoTarget {
        &self.source
    }
    pub const fn destination_generation(&self) -> u64 {
        self.destination_generation
    }
}

impl RecoveryStagingRedoStep {
    pub const fn operation(&self) -> [u8; 32] {
        self.operation
    }
    pub const fn record_lsn(&self) -> u64 {
        self.record_lsn
    }
    pub const fn record_index(&self) -> u64 {
        self.record_index
    }
    pub const fn target_index(&self) -> u64 {
        self.target_index
    }
    pub const fn prior(&self) -> RecoveryPageObservation {
        self.prior
    }
}

impl RecoveryQuiescencePlan {
    pub const fn staging_commands(self) -> u64 {
        self.staging_commands
    }
    pub const fn publication_commands(self) -> u64 {
        self.publication_commands
    }
    pub const fn expected_live_commands_after_close(self) -> u64 {
        self.expected_live_commands_after_close
    }
    pub const fn expected_live_media_handles_after_close(self) -> u64 {
        self.expected_live_media_handles_after_close
    }
}
