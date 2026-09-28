# WORTH Relational DX Exhaustive Audit

This document is generated from the live code in `crates/worth-relational/src/facade.rs`.
It exists so DX work can reference the real public facade surface instead of stale narrative docs.

Important note:

- this audit tracks the non-test public facade
- `#[cfg(test)]` facade modules and exports are intentionally excluded from the
  counts below

## Summary

| Module | Symbols | Source Groups |
| --- | ---: | ---: |
| `config` | 26 | 1 |
| `commit_strategies` | 70 | 4 |
| `diagnostics` | 10 | 2 |
| `durability` | 28 | 1 |
| `errors` | 5 | 1 |
| `history` | 30 | 1 |
| `identity` | 12 | 1 |
| `inspection` | 44 | 2 |
| `indexes` | 10 | 1 |
| `lineage` | 37 | 1 |
| `merge` | 92 | 3 |
| `runtime` | 44 | 6 |
| `payloads` | 5 | 1 |
| `publication` | 26 | 4 |
| `query` | 25 | 1 |
| `replay` | 20 | 1 |
| `schema` | 94 | 2 |
| `snapshots` | 4 | 1 |
| `storage` | 1 | 1 |
| `symbols` | 5 | 1 |
| `transactions` | 65 | 2 |

## Source Group Notes

- One source group usually means the facade module is mostly a straight re-export bucket.
- Multiple source groups usually means the public module is combining several architectural jobs.
- This audit only covers the external crate contract reachable through the top-level facade.

## `facade::config`

- Total symbols: `26`
- Source groups: `1`

### Source Groups

- `crate::config::data::`
  - count: `26`

### Symbols

#### `crate::config::data::`

- `AdjacencyBackend`
- `AdjacencyPolicy`
- `CascadeDeletePolicy`
- `CheckpointPolicy`
- `CommitStrategiesConfig`
- `CompiledLanePolicy`
- `ConfigProvenance`
- `ConfigProvenanceEntry`
- `ConfigValueSource`
- `CrossContextPolicy`
- `DiagnosticsBoundary`
- `DurabilityPolicy`
- `DurableLogPolicy`
- `DurableLogRetentionMode`
- `MvccConfig`
- `PatchSurfacePolicy`
- `PublicationConfig`
- `RelationalConfigOverride`
- `RelationalRuntimeProfile`
- `RetentionBackend`
- `RetentionPolicy`
- `RuntimeExecutionLane`
- `RuntimeProfileBoundaryPolicy`
- `SnapshotReleasePolicy`
- `StorageLayoutConfig`
- `VisibilityCachePolicy`

## `facade::commit_strategies`

- Total symbols: `70`
- Source groups: `4`

### Source Groups

- `crate::commit_strategies::`
  - count: `2`
- `crate::commit_strategies::data::`
  - count: `51`
- `crate::commit_strategies::facade::`
  - count: `2`
- `crate::commit_strategies::strategies::`
  - count: `15`

### Symbols

#### `crate::commit_strategies::`

- `FrozenCommitStrategyRegistry`
- `StrategyExecutionError`

#### `crate::commit_strategies::data::`

- `CanonicalStrategyCommitRequest`
- `CanonicalStrategyInputArtifact`
- `CanonicalStrategyInputDigest`
- `CanonicalStrategyOutputArtifact`
- `CanonicalStrategyOutputDigest`
- `CommitStrategyDescriptor`
- `CommitStrategyDescriptorDigest`
- `CommitStrategyExecutionRegistration`
- `CommitStrategyExecutor`
- `CommitStrategyFamilyName`
- `CommitStrategyId`
- `CommitStrategyRegistration`
- `CommitStrategyRegistrationError`
- `CommitStrategySemanticName`
- `CommitStrategyVersion`
- `LoweredStrategyCommitPlan`
- `PersistentArtifactName`
- `RawStrategyCommitRequest`
- `StrategyCallerProvenance`
- `StrategyCommitArtifactBundle`
- `StrategyCommitRequestError`
- `StrategyExecutionDraft`
- `StrategyExecutionResult`
- `StrategyExecutionSummary`
- `StrategyExecutorFailure`
- `StrategyExecutorFailureClass`
- `StrategyInputSchemaName`
- `StrategyInputSchemaVersion`
- `StrategyIntentName`
- `StrategyIntentScopeDigest`
- `StrategyLoweringError`
- `StrategyLoweringProvenance`
- `StrategyLoweringSummary`
- `StrategyMergeConflictClass`
- `StrategyMergeDescriptor`
- `StrategyMergeSemantics`
- `StrategyMutationProgram`
- `StrategyMutationProgramDigest`
- `StrategyObservationContext`
- `StrategyOutputSchemaName`
- `StrategyPacketContract`
- `StrategyReadContract`
- `StrategyReadCostClass`
- `StrategyReadLocalityClass`
- `StrategyReadScopeClass`
- `StrategyReplayDescriptor`
- `StrategyRequestCanonicalization`
- `StrategyRequestOrigin`
- `StrategyTraversalBasis`
- `StrategyVisibilityReadView`
- `ValidatedStrategyCommitPlan`

#### `crate::commit_strategies::facade::`

- `CommitStrategiesAuthorityFacade`
- `CommitStrategiesFacade`

#### `crate::commit_strategies::strategies::`

- `AspectFieldReconciliationInput`
- `AspectFieldReconciliationOutput`
- `AspectFieldReconciliationStrategy`
- `EntityReplacementReconciliationAction`
- `EntityReplacementReconciliationInput`
- `EntityReplacementReconciliationOutput`
- `EntityReplacementReconciliationStrategy`
- `IntentReconciliationAction`
- `IntentReconciliationInput`
- `IntentReconciliationOutput`
- `IntentReconciliationStrategy`
- `ReplicaConvergenceAction`
- `ReplicaConvergenceInput`
- `ReplicaConvergenceOutput`
- `ReplicaConvergenceStrategy`

## `facade::diagnostics`

- Total symbols: `10`
- Source groups: `2`

### Source Groups

- `crate::diagnostics::data::`
  - count: `9`
- `crate::diagnostics::facade::`
  - count: `1`

### Symbols

#### `crate::diagnostics::data::`

- `DeterminismExpectation`
- `DiagnosticCode`
- `DiagnosticsArtifactKind`
- `DiagnosticsDeliveryClass`
- `DiagnosticsScope`
- `RelationalArtifactPolicy`
- `RelationalDiagnosticArtifact`
- `RelationalDiagnosticsEntry`
- `RelationalDiagnosticsProfile`

#### `crate::diagnostics::facade::`

- `RelationalDiagnosticsFacade`

## `facade::durability`

- Total symbols: `28`
- Source groups: `1`

### Source Groups

- `crate::durability::data::`
  - count: `28`

### Symbols

#### `crate::durability::data::`

- `CheckpointCoverage`
- `CompactionOutcome`
- `CompactionPlan`
- `CompactionPolicy`
- `DurabilityError`
- `DurabilityMode`
- `DurableCheckpoint`
- `DurableCheckpointId`
- `DurableCheckpointManifest`
- `DurableIntegrityStatus`
- `DurableSegmentId`
- `DurableSegmentManifest`
- `DurableStore`
- `DurableStoreLayout`
- `PartitionCheckpointImage`
- `RecoveryAuthorityParity`
- `RecoveryCompatibilityCheck`
- `RecoveryCompatibilityMismatch`
- `RecoveryCoverage`
- `RecoveryCursor`
- `RecoveryFailureClass`
- `RecoveryIntegrityReport`
- `RecoveryPlan`
- `RecoveryVerificationMode`
- `RecoveryVerificationOutcome`
- `RecoveryVerificationPlan`
- `RelationIntegrityContractFamily`
- `SegmentRetentionClass`

## `facade::errors`

- Total symbols: `5`
- Source groups: `1`

### Source Groups

- `crate::errors::data::`
  - count: `5`

### Symbols

#### `crate::errors::data::`

- `ErrorContext`
- `ErrorOperation`
- `RelationalError`
- `RelationalSubsystem`
- `SuggestedFix`

## `facade::history`

- Total symbols: `30`
- Source groups: `1`

### Source Groups

- `crate::history::data::`
  - count: `30`

### Symbols

#### `crate::history::data::`

- `AspectFilter`
- `AspectFilterMode`
- `AspectHistoryCommitSpan`
- `AspectHistoryDigest`
- `AspectHistoryEntry`
- `AspectHistoryLineageEventSpan`
- `AspectHistoryOrigin`
- `AspectHistoryQueryResult`
- `AspectHistoryResolutionTrace`
- `AspectResolutionContext`
- `BranchCreateError`
- `BranchCreateErrorClass`
- `BranchHead`
- `BranchId`
- `CommitId`
- `CommitReference`
- `HistoryAspectQueryTarget`
- `HistoryDriftClass`
- `HistoryRetentionClass`
- `HistoryShapeClassification`
- `LineageAspectHistory`
- `LineageAspectHistoryQueryResult`
- `LineageAspectResolutionDigest`
- `MergeConflictRecord`
- `MergeInspection`
- `OrderedParentList`
- `RequestedAspectSet`
- `VersionGraphPolicy`
- `VersionGraphSnapshot`
- `VersionNode`

## `facade::identity`

- Total symbols: `12`
- Source groups: `1`

### Source Groups

- `crate::identity::data::`
  - count: `12`

### Symbols

#### `crate::identity::data::`

- `EntityId`
- `EntityStorageId`
- `Generation`
- `KindId`
- `LineageId`
- `LocalSlot`
- `PartitionId`
- `RelationId`
- `RelationStorageId`
- `StructuralFingerprint`
- `VersionBound`
- `VersionId`

## `facade::inspection`

- Total symbols: `44`
- Source groups: `2`

### Source Groups

- `crate::inspection::data::`
  - count: `43`
- `crate::inspection::logic::`
  - count: `1`

### Symbols

#### `crate::inspection::data::`

- `CommitInspection`
- `ConnectivityComponentSummary`
- `ConnectivityInspectionBudget`
- `ConnectivityInspectionRequest`
- `ConnectivityInspectionSummary`
- `GraphInspectionBudget`
- `GraphInspectionRequest`
- `GraphInspectionSummary`
- `HistoricalAspectObservation`
- `HistoricalAvailabilityObservation`
- `HistoricalInspectionMode`
- `HistoricalOpenResult`
- `HistoricalRecordInspection`
- `HistoricalRecordObservation`
- `HistoricalRecordValue`
- `HistoricalSnapshotView`
- `InspectionAccessPath`
- `InspectionAvailability`
- `InspectionDegradation`
- `InspectionOrigin`
- `InspectionRecordClass`
- `InspectionResolutionContext`
- `InspectionScope`
- `KindInspectionRequest`
- `KindInspectionSummary`
- `NeighborInspectionResult`
- `PinStateObservation`
- `RecentCommitInspectionRequest`
- `RecentCommitInspectionWindow`
- `ReclaimEligibility`
- `RecordRetentionInspection`
- `RetentionExecutionInspection`
- `RetentionInspectionRequest`
- `RetentionInspectionSummary`
- `RetentionStateObservation`
- `SavepointInspectionSurface`
- `SnapshotPinInspection`
- `StructuralIdentityComparison`
- `StructuralIdentityComparisonVerdict`
- `StructuralIdentityEvidence`
- `StructuralIdentityQueryRequest`
- `TransactionInspectionSurface`
- `TransactionIntentCounts`

#### `crate::inspection::logic::`

- `InspectionAccess`

## `facade::indexes`

- Total symbols: `10`
- Source groups: `1`

### Source Groups

- `crate::indexes::data::`
  - count: `10`

### Symbols

#### `crate::indexes::data::`

- `DerivedIndexBuildOutcome`
- `DerivedIndexBuildRequest`
- `DerivedIndexCompatibility`
- `DerivedIndexDefinition`
- `DerivedIndexGeneration`
- `DerivedIndexGenerationId`
- `DerivedIndexId`
- `DerivedIndexKind`
- `DerivedIndexPayload`
- `DerivedIndexPublicationStatus`

## `facade::lineage`

- Total symbols: `37`
- Source groups: `1`

### Source Groups

- `crate::lineage::data::`
  - count: `37`

### Symbols

#### `crate::lineage::data::`

- `CorrespondenceCandidate`
- `CorrespondenceCandidateId`
- `CorrespondencePromotionExecutionFailureClass`
- `CorrespondencePromotionOutcome`
- `CorrespondencePromotionRejectionClass`
- `CorrespondenceResolution`
- `HistoricalLineageResolution`
- `HistoricalLineageResolutionDigestBasis`
- `HistoricalLineageResolutionMetrics`
- `HistoricalResolutionBoundednessBasis`
- `HistoricalResolutionDigestMode`
- `HistoricalResolutionRequest`
- `HistoricalResolutionTrace`
- `LineageArtifactCounters`
- `LineageCheckpointArtifact`
- `LineageCheckpointCounters`
- `LineageCheckpointDigestBasis`
- `LineageDecisionKind`
- `LineageDecisionLogDigestBasis`
- `LineageDigestBasis`
- `LineageDivergenceMetrics`
- `LineageDivergenceRequest`
- `LineageDivergenceSummary`
- `LineageDivergenceTraversalBasis`
- `LineageEventBatchDigestBasis`
- `LineageEventKind`
- `LineageEventRecord`
- `LineageGraphDigestBasis`
- `LineageGraphDigestMode`
- `LineageGraphMetrics`
- `LineageGraphRequest`
- `LineageGraphSnapshot`
- `LineageGraphTraversalBasis`
- `LineageInvariant`
- `LineageNode`
- `LineageResolutionStatus`
- `RecordHistoryRequest`

## `facade::merge`

- Total symbols: `92`
- Source groups: `3`

### Source Groups

- `crate::merge::data::`
  - count: `90`
- `crate::merge::logic::`
  - count: `1`
- `crate::transactions::data::`
  - count: `1`

### Symbols

#### `crate::merge::data::`

- `AspectMergePolicyDeclaration`
- `AspectMergePolicyKind`
- `BranchCausalDot`
- `BranchDeltaSummary`
- `CausalAnnotationSummary`
- `CausalFrontier`
- `CommitCausalMetadata`
- `CommitCausalRelation`
- `ConflictClassificationSummary`
- `CustomIdentityBasisIdentity`
- `CustomMergePolicyIdentity`
- `DeletionExecutionClass`
- `DeletionMergeClass`
- `EndpointContinuityClass`
- `ExecutedMergeAspectClass`
- `ExecutedMergeAspectDiagnosticRow`
- `ExecutedMergeRecordClass`
- `ExecutedMergeRecordDiagnosticRow`
- `IdentityBasisDeclaration`
- `IdentityBasisKind`
- `IdentityBasisScope`
- `IdentityDiscoverySummary`
- `IdentityMatchCandidate`
- `IdentityMatchClass`
- `IdentityResolutionReason`
- `LoweredAspectAction`
- `LoweredAspectOutcome`
- `LoweredMergeAction`
- `LoweredMergeBlockedReason`
- `LoweredMergePlanRecord`
- `LoweredMergePlanSummary`
- `LoweredMergeRejectedReason`
- `LoweredRecordDecision`
- `LoweredRecordDecisionKind`
- `LoweredRecordDenialKind`
- `MergeAncestrySummary`
- `MergeArtifactDigestBasis`
- `MergeBaseSelectionRule`
- `MergeCausalEvidenceModel`
- `MergeConflictClass`
- `MergeConflictClassification`
- `MergeExecutableClass`
- `MergeExecutionAuthorityContract`
- `MergeExecutionAuthorizationRule`
- `MergeExecutionCompilationError`
- `MergeExecutionDecisionSurface`
- `MergeExecutionDeniedRecord`
- `MergeExecutionDiagnosticsPlan`
- `MergeExecutionError`
- `MergeExecutionFreshnessPolicy`
- `MergeExecutionMutationPlanError`
- `MergeExecutionPreparationError`
- `MergeExecutionReadiness`
- `MergeExecutionReadinessReport`
- `MergeExecutionRequest`
- `MergeIntent`
- `MergeManualResolutionClass`
- `MergePlanningArtifactCore`
- `MergePlanningDecisionKind`
- `MergePlanningDecisionLog`
- `MergePlanningDecisionLogDigestBasis`
- `MergePlanningDecisionRecord`
- `MergePlanningError`
- `MergePlanningRequest`
- `MergePlanningSummary`
- `MergePolicyDecisionBoundary`
- `MergePolicyOwnershipClass`
- `MergePolicyOwnershipSurface`
- `MergePolicyProofBoundary`
- `MergePolicyRejectClass`
- `MergePolicyResolution`
- `MergeRecordCausalAnnotation`
- `MergeRecordCausalDisposition`
- `MergeRecordIdentity`
- `MergeResolutionClass`
- `MergeResolvedAspectValueStrategy`
- `MergeSchemaKindClass`
- `MergeSchemaKindSemanticSnapshot`
- `MergeSchemaSnapshotDigestBasis`
- `MergeVisibilityEvidence`
- `MergeVisibilityEvidenceKind`
- `MergeVisibilityState`
- `PreparedMergeExecution`
- `RelationConflictPropagation`
- `RelationContinuityClass`
- `ResolvedMergeBase`
- `SchemaDeclaredCorrespondenceValidationSummary`
- `TopologyExecutionClass`
- `TopologyRegionConflictReason`
- `TopologyRewireAdmissionPolicy`

#### `crate::merge::logic::`

- `MergeAccess`

#### `crate::transactions::data::`

- `MergeExecutionOutcome`

## `facade::runtime`

- Total symbols: `44`
- Source groups: `6`

### Source Groups

- `crate::logic::builder::`
  - count: `1`
- `crate::logic::commit::`
  - count: `1`
- `crate::logic::planning::`
  - count: `2`
- `crate::logic::runtime::`
  - count: `36`
- `crate::presentation::api::`
  - count: `1`
- `crate::presentation::contracts::`
  - count: `3`

### Symbols

#### `crate::logic::builder::`

- `RelationalRuntimeBuilder`

#### `crate::logic::commit::`

- `CommitAuthorityContract`

#### `crate::logic::planning::`

- `PlanningContract`
- `RelationalExecutionModel`

#### `crate::logic::runtime::`

- `ChunkVisibilitySummary`
- `ChunkedStorageSummary`
- `CompiledArtifactCompatibility`
- `CompiledArtifactError`
- `CompiledExecutionArtifact`
- `ComplexityContract`
- `ComplexityStatus`
- `EntityReadRecord`
- `EntityRecordProjection`
- `InvariantAccess`
- `InvariantCatalog`
- `InvariantCheckResult`
- `InvariantClass`
- `InvariantDecisionKind`
- `InvariantDecisionRecord`
- `InvariantExecutionPoint`
- `InvariantFailureEffect`
- `InvariantRegistration`
- `InvariantRule`
- `PartitionStorageStats`
- `RelationReadRecord`
- `RelationRecordProjection`
- `RelationalReadView`
- `RelationalReplayRecord`
- `RelationalRuntime`
- `RelationalRuntimeConfig`
- `ReplaySchemaVersion`
- `RuntimeComplexityCounters`
- `SimulationAccess`
- `SimulationAuthority`
- `SnapshotGuard`
- `StorageStats`
- `TopologyFreezeMode`
- `VisibilityProjectionView`
- `VisibilityReadContext`
- `VisibilityRetentionAuthority`

#### `crate::presentation::api::`

- `RelationalRuntimeApi`

#### `crate::presentation::contracts::`

- `ImmutableReadContract`
- `RelationalBoundaryContract`
- `SerializedAuthorityContract`

## `facade::payloads`

- Total symbols: `5`
- Source groups: `1`

### Source Groups

- `crate::payloads::data::`
  - count: `5`

### Symbols

#### `crate::payloads::data::`

- `PayloadClass`
- `PayloadCompatibility`
- `PayloadEncoding`
- `PayloadPolicy`
- `RecordPayload`

## `facade::publication`

- Total symbols: `26`
- Source groups: `4`

### Source Groups

- `crate::publication::bundle::`
  - count: `3`
- `crate::publication::cdc::data::`
  - count: `8`
- `crate::publication::data::`
  - count: `1`
- `crate::publication::patch::data::`
  - count: `14`

### Symbols

#### `crate::publication::bundle::`

- `PublicationBundle`
- `PublicationStage`
- `PublicationStatus`

#### `crate::publication::cdc::data::`

- `SubscriberCheckpoint`
- `SubscriberRecoveryDecision`
- `SubscriberRecoveryDisposition`
- `SubscriberRecoverySource`
- `SubscriberResumeRequest`
- `SubscriberStreamBatch`
- `SubscriberStreamFailure`
- `SubscriberStreamFailureClass`

#### `crate::publication::data::`

- `PublicationError`

#### `crate::publication::patch::data::`

- `AspectKey`
- `CanonicalAspectSet`
- `PatchFragmentBudget`
- `PatchOrdering`
- `PatchPublicationMode`
- `PatchRecord`
- `PatchRecordKind`
- `PatchStreamBatch`
- `PatchStreamPosition`
- `PatchStreamReadError`
- `PatchStreamReadErrorClass`
- `PatchStreamRequest`
- `RecordStructuralChange`
- `RelationalPatchRecord`

## `facade::query`

- Total symbols: `25`
- Source groups: `1`

### Source Groups

- `crate::query::data::`
  - count: `25`

### Symbols

#### `crate::query::data::`

- `CanonicalQueryResult`
- `DeterministicQueryFragmentKey`
- `DeterministicQueryPlanKey`
- `FallbackParityMode`
- `FallbackParityVerifiedQueryOutcome`
- `IndexQueryRejectionClass`
- `PartitionHint`
- `PlannedQueryPacket`
- `QueryAccessPath`
- `QueryComplexitySummary`
- `QueryExecutionOutcome`
- `QueryExecutionShape`
- `QueryFallbackContract`
- `QueryFragmentCounters`
- `QueryLocalityClass`
- `QueryOrderingContract`
- `QueryParallelLegality`
- `QueryParallelProfitability`
- `QueryPlanContextId`
- `QueryPlanEvidenceBasis`
- `QueryScope`
- `QuerySerialReason`
- `QueryWorkerFragment`
- `ReductionDiscipline`
- `SnapshotPinnedQueryPlan`

## `facade::replay`

- Total symbols: `20`
- Source groups: `1`

### Source Groups

- `crate::replay::data::`
  - count: `20`

### Symbols

#### `crate::replay::data::`

- `CanonicalCommitAuthorityKind`
- `CanonicalCommitEnvelope`
- `CertifiedLineageSurfaceComparisonBasis`
- `CertifiedLineageSurfaceDigest`
- `LineageCertifiedSurfaceKind`
- `RelationalReplayOutcome`
- `RelationalReplayRequest`
- `ReplayAuthorityBasisKind`
- `ReplayError`
- `ReplayExecutionMode`
- `ReplayFailureClass`
- `ReplayLineageAuthorityBasis`
- `ReplayLineageDigestMode`
- `ReplayMismatch`
- `ReplayMismatchClass`
- `ReplayObservableSurface`
- `ReplaySnapshotSurface`
- `ReplayVerificationLayer`
- `ReplayVerificationMode`
- `ReplayVerificationPlan`

## `facade::schema`

- Total symbols: `94`
- Source groups: `2`

### Source Groups

- `crate::publication::patch::data::`
  - count: `1`
- `crate::schema::data::`
  - count: `93`

### Symbols

#### `crate::publication::patch::data::`

- `AspectKey`

#### `crate::schema::data::`

- `AcyclicityContractDeclaration`
- `AllowedCycleClass`
- `AspectBinding`
- `AspectComparator`
- `AspectDeclarationTrace`
- `AspectDeclarationTraceRow`
- `AspectLoweringTrace`
- `AspectLoweringTraceRow`
- `AspectPlanRevision`
- `AspectPrecision`
- `CardinalityContractDeclaration`
- `CompatibilityObservation`
- `ConnectivityMinimumContractDeclaration`
- `ConnectivityMinimumEnforcement`
- `ContractId`
- `DeclaredAspect`
- `DescriptorCanonicalizationCompatibilityPolicy`
- `DescriptorCanonicalizationVersion`
- `DescriptorSemanticsCompatibilityPolicy`
- `DescriptorSemanticsVersion`
- `DirectedTraversalKind`
- `EndpointDeletionIntegrityDeclaration`
- `EndpointDeletionIntegrityMode`
- `EndpointKindContractDeclaration`
- `EntityKindRegistration`
- `FreeFormSchemaDiffIntent`
- `HistoricalInterpretationSensitivity`
- `KindAspectDeclarations`
- `KindResolution`
- `LoweredAcyclicityContract`
- `LoweredAspectBinding`
- `LoweredAspectComparator`
- `LoweredAspectExtractor`
- `LoweredAspectPlan`
- `LoweredCardinalityMaximumContract`
- `LoweredCardinalityMinimumContract`
- `LoweredConnectivityMinimumContract`
- `LoweredEndpointDeletionIntegrityContract`
- `LoweredEndpointKindContract`
- `LoweredExecutableAspectBindingKind`
- `LoweredPartitionIsolationContract`
- `LoweredPayloadSchemaContract`
- `LoweredRelationIntegrityPlan`
- `LoweredSchemaTransitionPlan`
- `LoweredSymmetryContract`
- `LoweredUniquenessContract`
- `MinimumCardinalityEnforcement`
- `PairMinimumSemantics`
- `PartitionIsolationContractDeclaration`
- `PartitionIsolationMode`
- `PayloadContractRecordKind`
- `PayloadFieldConstraint`
- `PayloadFieldConstraintDeclaration`
- `PayloadSchemaDeclaration`
- `PayloadSchemaValueType`
- `ProposedSchemaTransition`
- `RelationIntegrityDeclarations`
- `RelationIntegrityPlanCatalog`
- `RelationIntegrityPlanRevision`
- `RelationKindRegistration`
- `RelationPayloadClass`
- `RelationalSchemaRegistry`
- `SchemaBoundaryFingerprint`
- `SchemaBridgeDescriptor`
- `SchemaBridgeabilityClassification`
- `SchemaContinuationClassification`
- `SchemaContinuationDescriptor`
- `SchemaDiffAtom`
- `SchemaDiffDetail`
- `SchemaElementKind`
- `SchemaElementRef`
- `SchemaId`
- `SchemaLineageArtifact`
- `SchemaLineageOrderingSemantics`
- `SchemaPublicationImpact`
- `SchemaReconciliationClassification`
- `SchemaReconciliationDescriptor`
- `SchemaReconciliationOrderingMode`
- `SchemaReconciliationPolicy`
- `SchemaRegistryError`
- `SchemaRegistryErrorClass`
- `SchemaStratum`
- `SchemaSubscriberImpact`
- `SchemaTransitionArtifact`
- `SchemaTransitionBarrier`
- `SchemaTransitionSummary`
- `SchemaVersionId`
- `SubscriberBoundaryVisibility`
- `SymmetryContractDeclaration`
- `SymmetryMode`
- `UniquenessContractDeclaration`
- `UniquenessScope`
- `ValidatedSchemaTransition`

## `facade::snapshots`

- Total symbols: `4`
- Source groups: `1`

### Source Groups

- `crate::snapshots::data::`
  - count: `4`

### Symbols

#### `crate::snapshots::data::`

- `SnapshotHandle`
- `SnapshotId`
- `SnapshotInspectionSummary`
- `SnapshotReadPolicy`

## `facade::storage`

- Total symbols: `1`
- Source groups: `1`

### Source Groups

- `crate::storage::data::`
  - count: `1`

### Symbols

#### `crate::storage::data::`

- `RecordLifecycleState`

## `facade::symbols`

- Total symbols: `5`
- Source groups: `1`

### Source Groups

- `crate::symbols::data::`
  - count: `5`

### Symbols

#### `crate::symbols::data::`

- `InternedString`
- `StringInterner`
- `Symbol`
- `SymbolPolicy`
- `SymbolTableSnapshot`

## `facade::transactions`

- Total symbols: `65`
- Source groups: `2`

### Source Groups

- `crate::transactions::data::`
  - count: `64`
- `crate::transactions::logic::`
  - count: `1`

### Symbols

#### `crate::transactions::data::`

- `AspectEmissionTrace`
- `AspectEvaluationTrace`
- `AspectEvaluationTraceRow`
- `AspectLifecycleTransitionClass`
- `AspectTagAccuracyReport`
- `AspectTraceEvidence`
- `AuthoritativeApplyPlan`
- `AuthorityMode`
- `BulkEntityCreateIntent`
- `BulkMutationLineagePlan`
- `BulkMutationLocalityFootprint`
- `BulkMutationNamingPlan`
- `BulkMutationProvenancePlan`
- `BulkMutationScope`
- `BulkRelationCreateIntent`
- `CommitAspectSummary`
- `CommitAuthority`
- `CommitChangeSummary`
- `CommitConflict`
- `CommitHistorySummary`
- `CommitLog`
- `CommitOutcome`
- `CommitPatchBudgetSummary`
- `CommitPhase`
- `CommitPhaseTiming`
- `CommitPublicationSummary`
- `CommitResult`
- `CommitSchemaSummary`
- `CommitStructuralSummary`
- `CommitSummary`
- `CommitTopology`
- `CommitTraceEvent`
- `ConflictClass`
- `CreateIntent`
- `CrossContextEndpointClass`
- `DeleteEntityIntent`
- `DeleteRelationIntent`
- `EntityMutationIntent`
- `LineageSafeBulkMutationBatch`
- `MergeCommitMutationPlan`
- `MergeExecutionOutcome`
- `MergeExecutionStructuralSummary`
- `MergeExecutionSummary`
- `MergedCommitPlan`
- `MutationIntent`
- `NamingStableBulkMutationBatch`
- `PatchVsTruthDeltaReport`
- `PlannedBulkMutationBatch`
- `PlannedLineageTransition`
- `ProvenanceCompleteBulkMutationBatch`
- `RecordRef`
- `RelationMutationIntent`
- `RelationScope`
- `ReplaceEntityIntent`
- `RollbackEffect`
- `RollbackOutcome`
- `RollbackSummary`
- `SavepointId`
- `TransactionCommitError`
- `TransactionId`
- `TransactionOptions`
- `UndoRecord`
- `UpdateEntityIntent`
- `WorkerIntentBatch`

#### `crate::transactions::logic::`

- `RelationalTransaction`

