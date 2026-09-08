use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessRecoveryObservation {
    pub(crate) observed_store_identity: Option<[u8; 16]>,
    pub(crate) posture: ProcessRecoveryPosture,
    pub(crate) recovery_effects: u64,
    pub(crate) discovery: Option<ProcessRecoveryDiscoveryCounters>,
    pub(crate) root_protocol: ProcessRecoveryRootProtocolCounters,
    pub(crate) root_protocol_denials: Vec<ProcessRootProtocolDenial>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProcessRecoveryStoreIdentityDenial {
    NotObserved,
    Substituted {
        expected: [u8; 16],
        observed: [u8; 16],
    },
}

impl ProcessRecoveryObservation {
    pub(crate) fn require_store_identity(
        &self,
        expected: [u8; 16],
    ) -> Result<[u8; 16], ProcessRecoveryStoreIdentityDenial> {
        match self.observed_store_identity {
            Some(observed) if observed == expected => Ok(observed),
            Some(observed) => {
                Err(ProcessRecoveryStoreIdentityDenial::Substituted { expected, observed })
            }
            None => Err(ProcessRecoveryStoreIdentityDenial::NotObserved),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessRecoveryPosture {
    Recovered,
    Refused(ProcessRecoveryRefusalCause),
    Blocked(ProcessRecoveryBlockCause),
    PublicationIndeterminate,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessRecoveryRefusalCause {
    CancelledBeforeDiscovery,
    CancelledBeforeReconstruction,
    CancelledBeforeExecution,
    EntryBindingDrift,
    PersistedStoreAdmission,
    CoordinationUnavailable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessRecoveryBlockCause {
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

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessRecoveryDiscoveryCounters {
    pub(crate) current_selector_integrity_admissions: u64,
    pub(crate) previous_selector_integrity_admissions: u64,
    pub(crate) current_selector_interpretations: u64,
    pub(crate) previous_selector_interpretations: u64,
    pub(crate) current_root_integrity_admissions: u64,
    pub(crate) previous_root_integrity_admissions: u64,
    pub(crate) current_root_candidate_interpretations: u64,
    pub(crate) previous_root_candidate_interpretations: u64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessRecoveryRootProtocolCounters {
    pub(crate) successor_root_integrity_admissions: u64,
    pub(crate) successor_root_interpretations: u64,
    pub(crate) staged_selector_integrity_admissions: u64,
    pub(crate) closeout_selector_interpretations: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessRootProtocolDenial {
    pub(crate) artifact: ProcessRootProtocolArtifact,
    pub(crate) denial: ProcessRootProtocolDenialKind,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessRootProtocolArtifact {
    BootstrapCatalog,
    CheckpointSourceRoot { generation: u64 },
    CurrentSelector,
    PreviousSelector,
    StagedCurrentSelector { publication: u64 },
    CurrentRoot { generation: u64 },
    PreviousRoot { generation: u64 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessRootProtocolDenialKind {
    Absent,
    ConflictingDuplication { observed_sources: u64 },
    Integrity(ProcessIntegrityRejection),
    NonCanonicalEncoding,
    ScopeMismatch,
    SourceIncarnationMismatch,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessIntegrityRejection {
    Damaged(ProcessDamageLocalization),
    Unsupported {
        scope: ProcessIntegrityScope,
        axis: ProcessIntegrityVersionAxis,
        observed: u32,
    },
    Unknown {
        scope: ProcessIntegrityScope,
        cause: ProcessUnknownIntegrityCause,
    },
    Indeterminate {
        scope: ProcessIntegrityScope,
        cause: ProcessIndeterminateIntegrityCause,
        observed_range: Option<ProcessByteRange>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessDamageLocalization {
    pub(crate) scope: ProcessIntegrityScope,
    pub(crate) cause: ProcessDamageCause,
    pub(crate) damaged_range: ProcessByteRange,
    pub(crate) field: Option<ProcessFormatField>,
    pub(crate) blast_radius: ProcessBlastRadius,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessIntegrityScope {
    pub(crate) store_identity: [u8; 16],
    pub(crate) family: ProcessIntegrityArtifactFamily,
    pub(crate) root_generation: Option<u64>,
    pub(crate) byte_range: ProcessByteRange,
    pub(crate) record_format_identity: Option<[u8; 10]>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProcessByteRange {
    pub(crate) offset: u64,
    pub(crate) length: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessIntegrityVersionAxis {
    EnvelopeSchema,
    PhysicalFormat,
    PhysicalWorkObligation,
    WalFrame,
    CheckpointRecordSchema,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessUnknownIntegrityCause {
    ExpectedArtifactAbsent,
    UnrecognizedArtifact,
    ExpectedScopeUnavailable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessIndeterminateIntegrityCause {
    SourceChangedDuringInspection,
    ObservationBoundExhausted,
    StableRangeNotProven,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessDamageCause {
    WrongMagic,
    FamilyMismatch,
    FramingLengthMismatch,
    ChecksumMismatch,
    FormatMismatch,
    StoreIdentityMismatch,
    ArtifactIdentityMismatch,
    PhysicalGenerationMismatch,
    SelectorRoleMismatch,
    RecordKindMismatch,
    ChildReferenceMismatch,
    SequenceMismatch,
    AggregateMismatch,
    MalformedStructure,
    Truncated,
    MissingArtifact,
    DuplicateArtifact,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessFormatField {
    Magic,
    EnvelopeSchema,
    FormatVersion,
    FormatDeclaration,
    EncodedLength,
    Checksum,
    StoreIdentity,
    ArtifactFamily,
    ArtifactIdentity,
    PhysicalGeneration,
    RuntimeIdentity,
    OperationIdentity,
    OperationFamily,
    TargetShape,
    PayloadDigestPresence,
    SelectorRole,
    RootGeneration,
    TreeIdentity,
    BlockIdentity,
    SegmentIdentity,
    PageIdentity,
    ExtentIdentity,
    RecordIdentity,
    ChunkOrdinal,
    WalLsnRange,
    CheckpointIdentity,
    CheckpointRecordKind,
    RoutingNodeKind,
    CheckpointAggregate,
    LinkedSelector,
    ChildReference,
    CompleteChildChecksum,
    NodeCapacity,
    SegmentPageCapacity,
    FreeSpaceEntryCount,
    AllocationFrontier,
    MembershipKind,
    MembershipCount,
    MembershipRange,
    Reserved,
    Payload,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessBlastRadius {
    DamagedRange,
    CanonicalFrame,
    CompleteArtifact,
    ReachableSubtree,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum ProcessIntegrityArtifactFamily {
    NamespaceIdentity,
    PhysicalWorkObligation,
    PageFrame,
    ExtentChunk,
    WalFrame,
    CheckpointStreamHeader,
    CheckpointDirtyBasis,
    CheckpointBindingCompaction,
    CheckpointBinding,
    CheckpointFooter,
    BootstrapCatalog,
    CurrentRootSelector,
    PreviousRootSelector,
    RootManifest,
    RootRoutingBlock,
    SegmentMembership,
    ExtentManifest,
    FreeSpaceHeader,
    FreeSpaceMembershipBlock,
}
