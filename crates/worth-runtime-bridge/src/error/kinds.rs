use super::typed::BridgeTypedError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeBuildErrorKind {
    MissingRelationalSource,
    MissingSignalSink,
    MissingMappingRegistrations,
    MissingSourceAdapter,
    DuplicateMappingRegistration,
    AmbiguousMappingRegistration,
    DuplicateAspectRegistration,
    AmbiguousAspectRegistration,
    DuplicateSourceDeclaration,
    AmbiguousSourceDeclaration,
    DuplicateStructuralDeclaration,
    AmbiguousStructuralDeclaration,
    DuplicateMergeDeclaration,
    AmbiguousMergeDeclaration,
    MergeOntologyLoweringMismatch,
    MergeAuthorityBasisMismatch,
    StructuralComparisonModeMismatch,
    SourceCapabilityMismatch,
    BuilderConfigurationConflict,
    InvalidFineGrainedWideningPolicy,
    AmbiguousSemanticDependencyRegistration,
    MixedSemanticCorrespondenceSignalGraphs,
}

pub type BridgeBuildError = BridgeTypedError<BridgeBuildErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeRouteErrorKind {
    EmptyBulkWorkloadRequest,
    RoutePolicyMismatch,
    UnsupportedProducerEnvelope,
    MissingMappingRegistration,
    AmbiguousSliceMappingRegistration,
    UnsupportedTruthPatchScope,
    UnsupportedTruthDeltaSurface,
    UnsupportedSubscriptionSlice,
    InconsistentNormalizedSurfaceDigest,
    InvalidAuthoritativePatchSemantics,
    UnsupportedAuthoritativePatchPrecision,
    SliceReadPacketConstructionFailure,
    InvalidLoweringContract,
}

pub type BridgeRouteError = BridgeTypedError<BridgeRouteErrorKind>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeDeliveryErrorKind {
    ExecutionDenied(super::BridgeExecutionDenial),
    SourceContractMismatch,
    InvalidWideningAdmission,
    BulkDeliveryRejected,
    HistoricalPolicyRejected,
    HistoricalTruthViewUnavailable(crate::adapter::RelationalBridgeSourceError),
    MissingBranchHeadSource,
    HistoricalBranchMismatch,
    HistoricalCommitMismatch,
    HistoricalSelectorMissingCommit,
    SnapshotAcquisitionFailure(crate::adapter::RelationalBridgeSourceError),
    SnapshotReadFailure(crate::snapshot::BridgeSnapshotReadError),
    SnapshotReadContractViolation(crate::snapshot::BridgeSnapshotReadError),
    SnapshotIdentityMismatch,
    StructuralContractMismatch,
    StructuralPlanRejected,
    SignalSinkRejection(crate::adapter::SignalBridgeSinkErrorKind),
}

pub type BridgeDeliveryError = BridgeTypedError<BridgeDeliveryErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeReplayErrorKind {
    ReplayArtifactsDisabled,
    CanonicalArtifactCoherenceFailure,
    StructuralReplayBasisTruncated,
    BulkPlanReplayMismatch,
    HistoricalEvaluationDeclarationMismatch,
    HistoricalEvaluationPolicyMismatch,
    HistoricalEvaluationAuthorityMismatch,
    RouteMismatch,
    InvalidationMismatch,
    SubscriptionSliceMismatch,
    ContinuityRequestMismatch,
    ContinuityResolutionMismatch,
    ContinuityArtifactMismatch,
    DigestMismatch,
    PlanningContractMismatch,
    LoweringContractMismatch,
}

pub type BridgeReplayError = BridgeTypedError<BridgeReplayErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeStreamErrorKind {
    UnsupportedConsumerShape,
    UnsupportedResumeMode,
    ProtocolVersionMismatch,
    CheckpointContractMismatch,
    CheckpointStreamMismatch,
    CheckpointTruncated,
    IllegalCoalescingBoundary,
    NonIdempotentDuplicateObservation,
    BackpressurePolicyViolation,
    StreamReplayMismatch,
    StreamDeliveryRejected,
    InvalidStreamMaterial,
}

pub type BridgeStreamError = BridgeTypedError<BridgeStreamErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeContinuityErrorKind {
    MissingLineageContext,
    MissingLineageSource,
    UnsupportedContinuityClass,
    InvalidContinuityRequestSet,
    LineageAuthorityMismatch,
    HistoricalResolutionFailure,
}

pub type BridgeContinuityError = BridgeTypedError<BridgeContinuityErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeMergeErrorKind {
    MergeContractMismatch,
    MergeContinuityDenied,
    MergeStructuralContradiction,
    MergeCausalFrontierTruncated,
    MergePolicyRejected,
    MergeDeletionDenied,
    MergeTopologyRewireDenied,
}

pub type BridgeMergeError = BridgeTypedError<BridgeMergeErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeSpeculationErrorKind {
    PreviewRequestKindMismatch,
    PreviewSessionIdentityConflict,
    PreviewBranchBindingMismatch,
    IllegalPreviewLifecycleTransition,
    PromotionAdmissibilityMismatch,
    PreviewReuseEquivalenceMismatch,
    PreviewResidueClassificationMismatch,
}

pub type BridgeSpeculationError = BridgeTypedError<BridgeSpeculationErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeWritebackErrorKind {
    WritebackNotRequested,
    PolicyRejected,
    StrategyUnavailable,
    FamilyBindingMismatch,
    StrategyDescriptorMismatch,
    IdempotenceBasisMismatch,
    CausalityEffectMismatch,
    StaleTruthBasis,
    InvariantRejected,
    MergeAuthorityRejected,
    StrategyFailed,
    StrategyPanicked,
    ReplayMismatch,
    AuthorityDenied,
    PreviewWritebackRejected,
}

pub type BridgeWritebackError = BridgeTypedError<BridgeWritebackErrorKind>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeLineageSourceErrorKind {
    UnsupportedContinuityClass,
    HistoricalResolutionFailure,
    DuplicateResolvedLineageIdentities,
    NonCanonicalResolvedLineageIdentities,
    DuplicateResolvedRecordIdentities,
    NonCanonicalResolvedRecordIdentities,
    DuplicateTraversedEventIds,
    NonCanonicalTraversedEventIds,
}

pub type BridgeLineageSourceError = BridgeTypedError<BridgeLineageSourceErrorKind>;
