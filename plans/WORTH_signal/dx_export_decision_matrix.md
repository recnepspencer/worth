# WORTH Signal Export Decision Matrix

## Purpose

This document is the full decision pass over the current public export surface.

It extends:

- [`plans/worth_signal/dx_api_matrix.md`](./dx_api_matrix.md)
- [`plans/worth_signal/dx_export_inventory.md`](./dx_export_inventory.md)
- [`plans/worth_signal/dx_exposure_cleanup_strategy.md`](./dx_exposure_cleanup_strategy.md)

This is a **full list** review, not a shortlist.

Every currently exported public surface is assigned:

- `Layer`
- `Action`

## Layer Legend

- `A`: semantic authoring surface
- `B`: execution and optimization policy surface
- `C`: integration-author surface
- `D`: internal enforcement / certification surface

## Action Legend

- `Keep`: remain part of the stable visible product surface
- `Condense`: keep the capability, but prefer a higher-level guided surface
- `Contain`: keep public, but move out of the main public path into a narrower namespace
- `Hide`: remove from the main facade or default docs, but not necessarily private yet
- `Internalize`: target for crate-private, support-only, test-only, or separate-crate treatment

This pass is intentionally conservative.

---

## Root Public Entry Paths

### `worth_signal::facade`

- Layer: `A/B/C`
- Action: `Keep`

Reason:

- this is the intended public boundary and should remain the primary curated
  entry path

### `worth_signal::easy`

- Layer: `A`
- Action: `Keep`

Reason:

- this is a legitimate guided surface
- it should remain clearly positioned as the fast-start layer, not the full
  production/runtime identity

### `worth_signal::diagnostics`

- Layer: `B/C`
- Action: `Contain`

Reason:

- direct top-level diagnostics exposure is valid, but this is a second large
  public boundary that can bypass facade curation

---

## `facade::types`

## Keep, Layer `A`

- `Aspect`
- `AspectMask`
- `AspectVersion`
- `NodeId`
- `SignalError`
- `DependencyEdge`
- `NodeState`
- `EvaluationCondition`
- `ChangedRegion`
- `NodeEvaluationResult`
- `OutputChange`
- `OutputIdentity`
- `PartitionSubscription`
- `PartitionToken`

Reason:

- these are the core semantic vocabulary of the runtime

## Keep, Layer `B`

- `CheckpointPolicy`
- `CheckpointBarrier`
- `TierPolicy`
- `DependencyMode`
- `DirtyPropagation`
- `EvaluationTrigger`
- `ComputationFamily`
- `ComputationKey`
- `KeyedComputation`

Reason:

- these are legitimate control surfaces, not architectural leakage

## Contain, Layer `B`

- `SignalBranchHandle`
- `SignalBranchId`
- `SignalSnapshotId`
- `SignalSnapshotMeta`
- `SignalSnapshotV1`
- `SnapshotRestoreIntent`
- `SnapshotRestorePlan`
- `MAX_ASPECTS`
- `BitsetFrontier`
- `DenseBitset`
- `CanonicalChangedRegions`
- `PartitionMatchMode`
- `MemoizedResultOrigin`
- `NodeContract`
- `ContextRequirement`
- `NodeEvaluationConfig`
- `SignalSnapshotDiagnostics`
- `SnapshotArtifactRestoreMode`
- `SnapshotArtifactRetentionPolicy`
- `SnapshotDependencyRestoreMode`
- `SnapshotRestoreCoarseReason`
- `SnapshotStateRestoreMode`
- `TraceSummary`
- `CausalityMetadata`
- `RuntimeArtifactHot`
- `RuntimeArtifactWarm`

Reason:

- these are real and useful, but they should not crowd the core primitive
  namespace

## Contain, Layer `C`

- `CanonicalDependencies`
- `CommittedSnapshotUpdate`
- `DependencySnapshotShape`
- `ReplacementSnapshotUpdate`
- `SharedDependencySnapshot`
- `SnapshotChangeKind`
- `SnapshotDeltaRecord`
- `SnapshotShapeHandle`
- `StableShapeSnapshotBasis`
- `VersionOnlySnapshotUpdate`
- `VersionVector`
- `BatchedDirtySet`
- `DomainImpact`
- `ArtifactEquivalenceContract`
- `ArtifactSemanticBoundary`
- `PersistentCorrespondenceEvidence`
- `ReuseBasis`
- `ReuseBoundaryAuthority`
- `ReuseBoundaryContext`
- `ReuseBoundaryEvidence`
- `ReuseBoundaryFailure`
- `ReuseBoundaryProof`
- `ReuseCertificationFailure`
- `ReuseCertificationRecord`
- `ReuseCrossing`
- `ReuseOrigin`
- `ReuseSemanticRegionIdentity`
- `ReuseSource`
- `ReuseStrategy`
- `ReuseStrategyBoundaryAuthority`
- `ArtifactAuthorityClass`
- `ArtifactMergeAuthority`
- `ColdArtifactIntent`
- `ColdArtifactRecord`
- `HistoricalArtifactRecord`
- `HotArtifactWrite`
- `MergeAdoptability`
- `RetainedDiagnosticArtifact`

Reason:

- these belong to integration/reuse/history specialists, not the main semantic
  surface

## Hide, Layer `C`

- `AspectMaskBits`
- `SignalCoreStorageProfile`
- `StableHashValue`
- `CORE_STORAGE_PROFILE`
- `CORE_STORAGE_PROFILE_ID`
- `HOT_VEC_INLINE_CAPACITY`
- `STABLE_HASH_WIDTH_BITS`
- `ArtifactPolicyClass`
- `AuthorityPolicy`
- `CanonicalDependencyOrder`
- `ComparatorBasis`
- `CompileTimePerformanceContract`
- `EquivalenceContract`
- `IdentityBasis`
- `MaintenanceMode`
- `NodeAuthorityContract`
- `NodeExecutionContract`
- `NodeProjectionContract`
- `NodeReuseContract`
- `NodeSemanticContract`
- `PathClass`
- `PerformanceCounterSurface`
- `PerformanceEnforcementLayer`
- `ResolvedPerformancePolicy`
- `SuppressionBasis`
- `StructuralMemoKey`

Reason:

- these are valid internal architecture concepts, but they are too low-level or
  proof-oriented for the main public path

## Contain, Layer `C`, with likely future condensation

- `CanonicalForm`
- `DedupedNodeBatch`
- `DeltaForm`
- `DependencyBatchEdit`
- `DependencySetEdit`
- `DesiredState`
- `DirtyBatch`
- `DirtyBatchEntry`
- `DirtyDelta`
- `FrontierEntryClassification`
- `FrontierExecutionCounters`
- `FrontierExecutionSummary`
- `FrontierInclusionBasis`
- `FrontierPlan`
- `FrontierPredictedCounters`
- `FrontierSeedCause`
- `FrontierValidationDecision`
- `FrontierWave`
- `FrontierWaveEntryPlan`
- `FrontierWaveEntrySummary`
- `FrontierWavePlan`
- `FrontierWaveSummary`
- `InvalidationFrontier`
- `InvalidationSeed`
- `InvalidationSeedBatch`
- `InvalidationTraceRecord`
- `LocalityFootprint`
- `LocallyOrderedShard`
- `LoweredForm`
- `MergeableOrderedStream`
- `MixedSnapshotBatchCommit`
- `NarrowedPropagationSet`
- `OrderedStreamItem`
- `OrderedStreamMergeError`
- `PartitionScopeSet`
- `PatchPlan`
- `PendingSnapshotBatch`
- `ResolvedForm`
- `SemanticBatchCommit`
- `SingleConsumer`
- `SnapshotBatchCommit`
- `SortedSourceBatch`
- `StableShapeSnapshotBatchCommit`
- `StructuralDelta`
- `SubscriberRepair`
- `SubscriberRepairBatch`
- `SummaryForm`
- `TouchedScopeSummary`
- `TransitiveFrontierRoot`

Reason:

- these are specialist proof-bearing forms
- the capabilities are important, but the raw type surface should not sit in
  `facade::types`

---

## `facade::graph`

## Keep, Layer `A`

- `SignalGraph`
- `NodeBuilder`

Reason:

- these are central semantic-authoring objects

## Condense, Layer `B`

- `VersionComparatorPolicy`
- `DefaultComparatorPolicyResolver`
- `DefaultComparatorResolver`
- `ComparatorPolicyResolver`
- `VersionComparatorResolver`
- `EvaluationStrategy`
- `GcPressure`
- `ObservationLevel`
- `ParallelismHint`

Reason:

- these are legitimate controls, but several of them represent policy plumbing
  that should likely be reachable through stronger guided configuration objects

## Contain, Layer `B`

- `GraphObserver`

Reason:

- observation is important, but it is not part of the first-layer graph
  authoring story

## Contain, Layer `C`

- `TierPolicyResolver`
- `EffectMapping`
- `CheckpointEvaluator`
- `EventSubscriber`
- `SubscriberId`
- `GraphMaterializer`
- `NodeMetaStore`
- `SubscriberContext`
- `SubscriberContextError`
- `RuntimeTelemetry`
- `TierPolicyTable`

Reason:

- these are integration/tooling surfaces

---

## `facade::evaluation`

## Keep, Layer `A`

- `EvaluationRequestMode`
- `EvaluationOutput`
- `IntoEvaluationOutput`
- `EvaluationVerdict`
- `NodeExplanation`
- `CausalDisposition`
- `CausalLink`
- `ConditionDecision`
- `MeaningfulChangeReason`
- `RewiringDependency`
- `RewiringSummary`
- `ScopeProvenance`
- `ScopeProvenanceKind`
- `UpstreamCause`

Reason:

- explanation is part of the product, not ancillary tooling

## Condense, Layer `B`

- `EvaluationContext`
- `ConditionEvaluationContext`
- `ConditionResolver`
- `DefaultConditionResolver`
- `DeferralReason`
- `SuppressionReason`
- `AppliedEffectReport`
- `EvaluationExecutionMetadata`

Reason:

- several of these are support shapes around evaluation control and reporting
- they should likely live behind clearer guided execution/session APIs

## Contain, Layer `C`

- `OperationalEffect`
- `DiagnosticEnvelope`

Reason:

- these are lower-level execution pipeline forms, useful to specialists but not
  part of the primary user-facing semantic story

---

## `facade::planning`

## Keep, Layer `B`

- `build_evaluation_plan`
- `execute_prepared_plan`
- `EvaluationPlan`
- `ExecutionReport`
- `StageExecutor`

Reason:

- explicit planning/execution is a legitimate advanced public capability

## Contain, Layer `B`

- `CandidateTask`
- `EligibleTask`
- `ExecutedTask`
- `ExecutionPruneReason`
- `ExecutionRecordId`
- `ExecutionStage`
- `PlanSummary`
- `ResolvedExecutionStrategy`
- `ResolvedMaintenanceStrategy`
- `SemanticSegmentId`
- `SemanticTaskRange`
- `StageBarrier`
- `StageExecutionOutcome`
- `StageExecutionRecord`
- `TaskExecutionOutcome`
- `TaskExecutionRecord`
- `TaskReason`
- `ParallelExecutionPolicy`

Reason:

- useful for observability and tuning, but too detailed for the main planning
  entry surface

## Condense, Layer `B`

- `build_evaluation_plan`
- `execute_prepared_plan`

Additional note:

- this pair is a candidate for a more guided request/session object over time

---

## `facade::performance`

## Hide, Layer `C`

- `ArtifactPolicyClass`
- `AuthorityPolicy`
- `CanonicalDependencyOrder`
- `ComparatorBasis`
- `CompileTimePerformanceContract`
- `EquivalenceContract`
- `IdentityBasis`
- `MaintenanceMode`
- `PathClass`
- `PerformanceCounterSurface`
- `PerformanceEnforcementLayer`
- `ResolvedExecutionStrategy`
- `ResolvedMaintenanceStrategy`
- `ResolvedPerformancePolicy`
- `SuppressionBasis`

Reason:

- these are meaningful, but as a top-level public namespace they are too raw
- they should be reintroduced only if we can package them around coherent
  guided control

---

## `facade::proof`

## Contain, Layer `C`

- `CanonicalForm`
- `DedupedNodeBatch`
- `DeltaForm`
- `DependencyBatchEdit`
- `DependencySetEdit`
- `DesiredState`
- `DirtyBatch`
- `DirtyBatchEntry`
- `DirtyDelta`
- `FrontierEntryClassification`
- `FrontierExecutionCounters`
- `FrontierExecutionSummary`
- `FrontierInclusionBasis`
- `FrontierPlan`
- `FrontierPredictedCounters`
- `FrontierSeedCause`
- `FrontierValidationDecision`
- `FrontierWave`
- `FrontierWaveEntryPlan`
- `FrontierWaveEntrySummary`
- `FrontierWavePlan`
- `FrontierWaveSummary`
- `InvalidationFrontier`
- `InvalidationSeed`
- `InvalidationSeedBatch`
- `InvalidationTraceRecord`
- `LocalityFootprint`
- `LocallyOrderedShard`
- `LoweredForm`
- `MergeableOrderedStream`
- `MixedSnapshotBatchCommit`
- `NarrowedPropagationSet`
- `OrderedStreamItem`
- `OrderedStreamMergeError`
- `PartitionScopeSet`
- `PatchPlan`
- `PendingSnapshotBatch`
- `ResolvedForm`
- `SemanticBatchCommit`
- `SingleConsumer`
- `SnapshotBatchCommit`
- `SortedSourceBatch`
- `StableShapeSnapshotBatchCommit`
- `StructuralDelta`
- `SubscriberRepair`
- `SubscriberRepairBatch`
- `SummaryForm`
- `TouchedScopeSummary`
- `TransitiveFrontierRoot`

Reason:

- this namespace should remain specialist only

## Condense, Layer `C`

- `DependencyBatchEdit`
- `DirtyBatch`
- `PendingSnapshotBatch`
- `SnapshotBatchCommit`
- `SemanticBatchCommit`

Reason:

- these raw batch forms likely need stronger guided batch/session surfaces above
  them

---

## `facade::transaction`

## Keep, Layer `A`

- `SignalRuntime`
- `SignalRuntimeBuilder`
- `SignalRuntimeConfig`
- `SignalTransaction`
- `TransactionOutcome`
- `TransactionResult`
- `TransactionTiming`
- `EvaluationSummary`
- `ComputationSpec`
- `DefinedComputation`
- `DefinedKeyedComputation`

Reason:

- these are core product surfaces

## Condense, Layer `A/B`

- `SignalRuntimeBuilder`
- `SignalRuntimeConfig`
- `ComputationSpec`
- `DefinedComputation`
- `DefinedKeyedComputation`
- `mark_dirty_batch`

Reason:

- runtime setup and computation definition are exactly the areas where guided
  builders and higher-level request/session objects can most improve DX

## Keep, Layer `B`

- `mark_dirty_batch`

Reason:

- batch invalidation is architecturally legitimate and should remain public

## Contain, Layer `B`

- `CheckpointRuntime`
- `EventBus`
- `EventFlushError`
- `SubscriberRegistryError`
- `RuntimeObserver`
- `AdvisoryRecord`
- `DecisionDetail`
- `DecisionLog`
- `DecisionRecord`
- `DecisionSummary`
- `IntegrityMarkers`
- `TransactionReplayEntry`
- `emit_event_in_txn`
- `flush_checkpoint_in_txn`

Reason:

- these are valid advanced/public controls, but they should not dominate the
  main runtime API path

## Contain, Layer `C`

- `ArtifactMergeAction`
- `ArtifactMergeComparable`
- `BranchMergeBase`
- `BranchMergeConflictEvidence`
- `BranchMergeConflictKind`
- `BranchMergeConflictRecord`
- `BranchMergeConflictSummary`
- `BranchMergeCounters`
- `BranchMergeDivergence`
- `BranchMergeExecutionSummary`
- `BranchMergeFailureKind`
- `BranchMergeKind`
- `BranchMergePlan`
- `BranchMergeReconciliationPolicy`
- `BranchMergeRequest`
- `BranchMergeResult`
- `BranchMergeStrategy`
- `BranchMutationJournalSlice`
- `BranchMutationLedger`
- `ConflictMergePolicy`
- `ConservativeOverlapExpansion`
- `DependencyFingerprint`
- `DependencyRemapRecord`
- `ExistingTargetMergePolicy`
- `LoweredMergePlan`
- `MergeBoundaryWitness`
- `MergeBoundaryWitnessKind`
- `MergeDecisionBasis`
- `MergeNodeMap`
- `MergeTouchedNodeSet`
- `MergedArtifactRecord`
- `NodeMergeInputState`
- `NodeMergePlan`
- `NodeReconciliationDecision`
- `NodeReconciliationShape`
- `PlannedMergeCandidateSet`
- `ProofMinimalOverlapBasis`
- `RuntimeMaterializer`
- `SourceNodeAdoptionPlanCore`
- `SourceOnlyMergePolicy`
- `StructuralMergeCandidateRecord`
- `StructuralMergeJournalSlice`

Reason:

- real integration/bridge power, but not core runtime identity

## Condense, Layer `C`

- `BranchMergeRequest`
- `BranchMergePlan`
- `BranchMergeResult`
- `BranchMergeReconciliationPolicy`
- `ConflictMergePolicy`
- `ExistingTargetMergePolicy`
- `SourceOnlyMergePolicy`

Reason:

- merge is a strong candidate for guided orchestration objects rather than a
  wide raw policy surface

---

## `facade::diagnostics`

## Keep, Layer `B`

- `diagnostics_for_graph`
- `diagnostics_for_runtime`
- `inspect_execution`
- `inspect_flow`
- `inspect_graph`
- `inspect_plan`
- `inspect_report`
- `render_execution_history_summary`
- `render_execution_report_summary`
- `render_explanation_summary`
- `render_failure_summary`
- `render_flow_summary`
- `render_graph_summary`
- `render_plan_summary`
- `compare_execution_history`
- `compare_execution_reports`
- `compare_explanations`
- `compare_failures`
- `compare_flows`
- `compare_graphs`
- `compare_lineage_records`
- `compare_plans`
- `compare_replay_slices`
- `explanations_semantically_equivalent`
- `graphs_semantically_equivalent`
- `lineage_records_equivalent`
- `plans_semantically_equivalent`
- `repeat_run_summaries_equal`
- `replay_slices_equivalent`
- `reports_semantically_equivalent`
- `serial_parallel_reports_equivalent`
- `ApplySummary`
- `ArtifactRetentionPolicy`
- `ArtifactTransitionKind`
- `ChangeInputSummary`
- `DiagnosticMismatch`
- `DiagnosticMismatchCategory`
- `DiagnosticsAvailability`
- `DiagnosticsTier`
- `EvaluationPlanSummary`
- `EventEpochOutcome`
- `EventEpochSummary`
- `EventSubscriberOutcome`
- `EventSubscriberOutcomeKind`
- `ExecutionFailureContext`
- `ExecutionFailurePhase`
- `ExecutionHistoryNodeSummary`
- `ExecutionHistorySummary`
- `ExecutionInspector`
- `ExecutionReportDiff`
- `ExecutionReportSummary`
- `ExplanationDiff`
- `ExplanationSummary`
- `FailureDiff`
- `FailureSummary`
- `FlowCauseSample`
- `FlowDiff`
- `FlowInspector`
- `FlowSummary`
- `FrontierCyclePolicy`
- `FrontierPropagationPolicy`
- `FrontierTracingPolicy`
- `GraphDiagnostics`
- `GraphDiff`
- `GraphInspector`
- `GraphSummary`
- `HistoryDiff`
- `InvalidationCause`
- `InvalidationSummary`
- `LineageArtifactId`
- `LineageDiff`
- `LineageRecord`
- `LineageRecordKind`
- `ParallelAdmissionPolicy`
- `PlanDiff`
- `PlanInspector`
- `PlanningSummary`
- `PrecomputeSummary`
- `ReconstructionBudget`
- `ReplayCursor`
- `ReplayDetailPolicy`
- `ReplayDiff`
- `ReplayEventKind`
- `ReplayFrame`
- `ReplaySlice`
- `ReportInspector`
- `RetentionBudget`
- `RollbackDiagnostic`
- `RollbackSummary`
- `RuntimeDiagnostics`
- `SemanticRetentionPolicy`
- `SignalRuntimePolicy`
- `SnapshotRestoreKind`
- `SnapshotRestoreLineageMode`

Reason:

- diagnostics is part of the product's value proposition

## Condense, Layer `B`

- `diagnostics_for_graph`
- `diagnostics_for_runtime`
- `inspect_execution`
- `inspect_flow`
- `inspect_graph`
- `inspect_plan`
- `inspect_report`
- render functions as a family
- compare functions as a family

Reason:

- these are good capabilities but still feel somewhat flat; a more guided access
  pattern could reduce sprawl without removing power

## Contain, Layer `C`

- `ArtifactTransitionKind`
- `InvalidationCause`
- `LineageArtifactId`
- `LineageDiff`
- `LineageRecord`
- `LineageRecordKind`
- `ReconstructionBudget`
- `SnapshotRestoreKind`
- `SnapshotRestoreLineageMode`

Reason:

- these are especially bridge/forensic flavored and may benefit from a narrower
  history/lineage namespace over time

---

## `facade::harness`

## Internalize, Layer `D`

- `DependencyGraphContract`
- `RawPathComputeContract`
- `StructuralStateBoundaryContract`
- `SignalDeploymentPlan`
- `SignalDeploymentPreset`
- `signal_bench`
- `signal_parity_suite`
- `SignalEvaluationDriver`
- `SignalFixtureFactory`
- `SignalHarnessAssert`
- `SignalHarnessBridge`
- `SignalHarnessRuntime`
- `SignalHarnessRuntimeBuilder`
- `SignalHarnessSession`
- `SignalMutationAction`
- `SignalMutationBatch`
- `SignalProfileCatalog`
- `SignalScenario`
- `GraphMetrics`
- `RuntimeMetrics`
- `TransactionRuntimeContract`

Reason:

- this entire namespace is internal/certification/support machinery, not
  product identity

---

## `worth_signal::easy`

## Keep, Layer `A`

- `ReactiveGraph`
- `InputSignal`
- `ComputedSignal`
- `Signal`

Reason:

- this is a valid guided on-ramp

## Condense, Layer `A`

- `ReactiveGraph`
- `ComputeContext`

Reason:

- this surface should be shaped as a strongly guided API with minimal ceremony,
  not as a second sprawling architecture

## Contain, Layer `A`

- `ComputeContext`

Reason:

- useful, but should stay subordinate to the guided easy-mode story

---

## `worth_signal::diagnostics`

## Contain, Layer `B/C`

- public submodules: `comparison`, `inspection`, `model`, `policy`
- re-export groups: `compare`, `diff`, `access`, `display`, `history`, `epochs`,
  `facts`, `failure`, `flow`, `lineage`, `profile`, `replay`, `summary`

Reason:

- this top-level boundary is too broad to remain unconstrained if `facade`
  becomes the curated product surface

## Contain, Layer `B`

- `ExplanationFact`
- `ProvenanceFact`
- `ReplayEvent`

## Contain, Layer `C`

- `RetainedLineageView`
- `SynthesizedLineageChain`
- `RetainedReplayView`
- `SynthesizedReplaySlice`

Reason:

- these are meaningful diagnostics primitives, but they are specialist support
  forms rather than the default product surface

---

## Edge Cases

## Keep, Layer `B`

- `ParallelExecutionPolicy` behind `feature = "parallel"`

Reason:

- feature-gated exposure is appropriate here

## Internalize

- `GraphDependencyBatchExt` under `#[cfg(test)]`

Reason:

- test-only helper, not shipped public API

---

## Cross-Cutting Cleanup Directions

The matrix implies these broad moves:

1. `facade::harness` should leave the visible product boundary first.
2. `facade::types` should be thinned aggressively.
3. `facade::transaction` should be split conceptually into:
   runtime operations, advanced transaction/history, and bridge/merge integration.
4. `facade::diagnostics` should stay powerful, but become less flat.
5. several important capabilities should be improved by condensation rather than
   simple hiding:
   runtime setup, computation definition, batch orchestration, merge
   orchestration, and diagnostics access.

---

## Immediate Follow-On

This matrix is now the full working list.

The next step is not more discovery. The next step is to decide the first
cleanup tranche from this matrix and turn that into concrete code changes.
