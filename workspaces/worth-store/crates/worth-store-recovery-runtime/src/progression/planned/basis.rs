use std::collections::{BTreeMap, BTreeSet};
use worth_store::physical_runtime::StoreRecoveryBindingFreshnessSample;
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, CurrentPhysicalRecordPlacement,
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, DurableRootSelector,
    PersistedPhysicalRecoveryRootState, PhysicalCheckpointIdentity, RecordArtifactFile,
    RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{
    ImmutablePhysicalRedoPlan, PhysicalRedoDecisionKind, PhysicalRedoDecisionPrior,
    PhysicalRedoTarget, PhysicalRedoTargetIdentity, PhysicalSourceSelection,
    ReconciledOperationFates, RecoveryPageObservation,
};

type SelectedRootTopologyEntry = (
    worth_store_physical_format::ManifestBlockReference,
    worth_store_physical_format::PhysicalRootRoutingBlock,
);

mod command;
mod derivation;
mod frame_identity;
mod identity;
mod publication_accessors;
mod publication_candidate;
mod staging_cost;

pub(crate) use derivation::{derive_execution_basis, requires_successor_candidate};
pub(crate) use publication_candidate::CandidateMaterializationCost;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExecutionBasisDenial {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecoverySelectedSourceInventory {
    pub(crate) free_space: DurableFreeSpaceManifestHeader,
    pub(crate) segment_pages: BTreeMap<(u64, u64), RecoverySelectedSegmentPage>,
    pub(crate) segment_topology:
        BTreeMap<(u64, u64), worth_store_physical_format::PhysicalSegmentMembershipBlock>,
    pub(crate) free_entries: Box<[RecordFreeSpaceManifestEntry]>,
    pub(crate) free_topology:
        BTreeMap<(u64, u64), worth_store_physical_format::PhysicalFreeSpaceMembershipBlock>,
    pub(crate) source_artifacts: Box<[RecordArtifactFile]>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RecoveryObservedSuccessorCandidate {
    pub(crate) root: DurablePhysicalRootManifest,
    pub(crate) free_space: DurableFreeSpaceManifestHeader,
    pub(crate) placements: Box<[CurrentPhysicalRecordPlacement]>,
    pub(crate) segment_entries: Box<[RecordSegmentPageManifestEntry]>,
    pub(crate) free_entries: Box<[RecordFreeSpaceManifestEntry]>,
    pub(crate) referenced_artifacts: Box<[RecordArtifactFile]>,
    pub(crate) artifacts: Box<[RecoveryObservedCandidateArtifact]>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RecoveryObservedCandidateArtifact {
    pub(crate) artifact: RecordArtifactFile,
    pub(crate) bytes: Box<[u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecoverySelectedSegmentPage {
    pub(crate) entry: RecordSegmentPageManifestEntry,
    pub(crate) routing_identity: [u8; 32],
    pub(crate) membership_artifact: RecordArtifactFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryStagingLayoutPlan {
    source_generation: u64,
    staging_generation: u64,
    base: RecoveryBaseImagePlan,
    actions: Box<[RecoveryStagingAction]>,
    commands: Box<[RecoveryStagingCommandPlan]>,
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
    bytes: Box<[u8]>,
    payload_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryBaseImagePlan {
    selected_selector: DurableRootSelector,
    selected_root: DurablePhysicalRootManifest,
    selected_root_topology: Box<[SelectedRootTopologyEntry]>,
    destination_generation: u64,
    actions: Box<[RecoveryBaseImageAction]>,
    segment_updates: Box<[RecoverySegmentRoutingAction]>,
    manifests: Box<[RecoveryPayloadManifestAction]>,
    root_states: Box<[PersistedPhysicalRecoveryRootState]>,
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
    artifact: RecordArtifactFile,
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
    checkpoint: PhysicalCheckpointIdentity,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPublicationCandidateArtifact {
    artifact: RecordArtifactFile,
    bytes: Box<[u8]>,
    payload_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryPublicationExpectation {
    store: StableStoreIdentity,
    checkpoint: PhysicalCheckpointIdentity,
    source_generation: u64,
    staging_generation: u64,
    plan_identity: [u8; 32],
    root_protocol: worth_store::physical_runtime::RecoveryRootProtocolPublicationPlan,
    current_selector: worth_store_physical_format::DurableRootSelector,
    recovered_root: DurablePhysicalRootManifest,
    referenced_artifacts: Box<[RecordArtifactFile]>,
    created_artifacts: Box<[RecordArtifactFile]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryPublicationAction {
    MaterializeRootCandidate { artifact: RecordArtifactFile },
    SynchronizeRootCandidate { artifact: RecordArtifactFile },
    ReplaceRootProtocol,
    SynchronizeStoreNamespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryQuiescencePlan {
    staging_commands: u64,
    publication_commands: u64,
    expected_live_commands_after_close: u64,
    expected_live_media_handles_after_close: u64,
}

impl RecoveryStagingLayoutPlan {
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

impl RecoveryBaseImagePlan {
    pub const fn selected_selector(&self) -> DurableRootSelector {
        self.selected_selector
    }
    pub const fn selected_root(&self) -> &DurablePhysicalRootManifest {
        &self.selected_root
    }
    pub(crate) fn selected_root_topology(&self) -> &[SelectedRootTopologyEntry] {
        &self.selected_root_topology
    }
    pub const fn destination_generation(&self) -> u64 {
        self.destination_generation
    }
    pub fn actions(&self) -> &[RecoveryBaseImageAction] {
        &self.actions
    }
    pub fn segment_updates(&self) -> &[RecoverySegmentRoutingAction] {
        &self.segment_updates
    }
    pub fn manifests(&self) -> &[RecoveryPayloadManifestAction] {
        &self.manifests
    }
    pub fn root_states(&self) -> &[PersistedPhysicalRecoveryRootState] {
        &self.root_states
    }
    pub fn source_artifacts(&self) -> &[RecordArtifactFile] {
        &self.source_artifacts
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
        self.artifact
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
