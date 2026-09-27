# WORTH Signal Public Export Inventory

## Purpose

This document is the strict public export audit for `worth-signal`.

It inventories:

- every root public module exposed from [`crates/worth-signal/src/lib.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/lib.rs)
- every public namespace exported from [`crates/worth-signal/src/facade.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/facade.rs)
- every additional direct public export path exposed through
  [`crates/worth-signal/src/diagnostics/mod.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/diagnostics/mod.rs)
  and [`crates/worth-signal/src/easy/mod.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/easy/mod.rs)
- notable edge cases like feature-gated and test-only exports

This is an **inventory and classification** artifact, not a recommendation that
all of these remain public.

---

## Tier Legend

- `P0`: primary public product surface
- `P1`: advanced public surface
- `P2`: bridge-facing / integration-author surface
- `P3`: internal / certification / should not be part of the main public story

---

## Root Public Entry Paths

From [`crates/worth-signal/src/lib.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/lib.rs):

- `worth_signal::facade` â†’ main public boundary
- `worth_signal::easy` â†’ convenience surface
- `worth_signal::diagnostics` â†’ direct diagnostics surface

Assessment:

- `facade` is the intended primary boundary
- `easy` is a real public entry path and is explicitly convenience-only
- `diagnostics` is a second large public entry path that bypasses `facade`

Implication:

- if we want strict exposure discipline, `diagnostics` is a major place where
  surface area can leak regardless of what we do with `facade`

---

## Facade Inventory

Source: [`crates/worth-signal/src/facade.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/facade.rs)

## `facade::types`

### `P0`

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
- `SignalBranchHandle`
- `SignalBranchId`
- `SignalSnapshotId`
- `SignalSnapshotMeta`
- `SignalSnapshotV1`
- `SnapshotRestoreIntent`
- `SnapshotRestorePlan`
- `CheckpointPolicy`
- `CheckpointBarrier`
- `TierPolicy`
- `DependencyMode`
- `DirtyPropagation`
- `EvaluationTrigger`
- `ComputationFamily`
- `ComputationKey`
- `KeyedComputation`

### `P1`

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

### `P2`

- `AspectMaskBits`
- `SignalCoreStorageProfile`
- `StableHashValue`
- `CORE_STORAGE_PROFILE`
- `CORE_STORAGE_PROFILE_ID`
- `HOT_VEC_INLINE_CAPACITY`
- `STABLE_HASH_WIDTH_BITS`
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

### `P3`

- none currently exposed from `facade::types`

Assessment:

- `facade::types` mixes core product concepts with highly specialized proof,
  reuse, and storage-profile machinery
- this namespace is currently doing too much work

---

## `facade::graph`

### `P0`

- `SignalGraph`
- `NodeBuilder`

### `P1`

- `VersionComparatorPolicy`
- `DefaultComparatorPolicyResolver`
- `DefaultComparatorResolver`
- `ComparatorPolicyResolver`
- `VersionComparatorResolver`
- `EvaluationStrategy`
- `GcPressure`
- `GraphObserver`
- `ObservationLevel`
- `ParallelismHint`

### `P2`

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

### `P3`

- none currently exposed from `facade::graph`

Assessment:

- `SignalGraph` and `NodeBuilder` are clearly primary
- several observer/tooling/integration hooks currently sit beside the main graph
  surface

---

## `facade::evaluation`

### `P0`

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

### `P1`

- `EvaluationContext`
- `ConditionEvaluationContext`
- `ConditionResolver`
- `DefaultConditionResolver`
- `DeferralReason`
- `SuppressionReason`
- `AppliedEffectReport`
- `EvaluationExecutionMetadata`

### `P2`

- `OperationalEffect`
- `DiagnosticEnvelope`

### `P3`

- none currently exposed from `facade::evaluation`

Assessment:

- the explanation surface feels product-worthy
- the raw effect/envelope layer is more integration/runtime-facing than
  everyday application-facing

---

## `facade::planning`

### `P0`

- `build_evaluation_plan`
- `execute_prepared_plan`
- `EvaluationPlan`
- `ExecutionReport`
- `StageExecutor`

### `P1`

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
- `ParallelExecutionPolicy` `feature = "parallel"`

### `P2`

- none currently exposed from `facade::planning`

### `P3`

- none currently exposed from `facade::planning`

Assessment:

- the planning surface is relatively disciplined
- it is still fairly verbose for a day-one story

---

## `facade::performance`

### `P2`

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

### `P0`

- none

### `P1`

- none

### `P3`

- none

Assessment:

- this is a specialized namespace and should stay clearly specialist

---

## `facade::proof`

### `P2`

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

### `P0`

- none

### `P1`

- none

### `P3`

- none

Assessment:

- this namespace is coherent but highly specialist
- it should not bleed into the main adoption story

---

## `facade::transaction`

### `P0`

- `SignalRuntime`
- `SignalRuntimeBuilder`
- `SignalRuntimeConfig`
- `SignalTransaction`
- `TransactionOutcome`
- `TransactionResult`
- `TransactionTiming`
- `EvaluationSummary`
- `mark_dirty_batch`
- `ComputationSpec`
- `DefinedComputation`
- `DefinedKeyedComputation`

### `P1`

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

### `P2`

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

### `P3`

- none currently exposed from `facade::transaction`

Assessment:

- this namespace is carrying both the main runtime surface and a large merge /
  reconstructability surface
- it is one of the strongest candidates for future boundary narrowing

---

## `facade::diagnostics`

### `P1`

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

### `P2`

- none in the current facade split, though some of the lineage and retention
  policy surface is arguably bridge/forensic-specialist

### `P0`

- none

### `P3`

- none

Assessment:

- diagnostics is intentionally large and product-differentiating
- but it is still too large to present as a flat â€œstart hereâ€ surface

---

## `facade::harness`

### `P3`

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

### `P0`

- none

### `P1`

- none

### `P2`

- none

Assessment:

- this entire namespace is the clearest current candidate for removal from the
  main public product surface

---

## Additional Direct Public Surface Outside `facade`

These are public even if users never import `facade`.

## `worth_signal::easy`

Source: [`crates/worth-signal/src/easy/mod.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/easy/mod.rs)

### `P0`

- `ReactiveGraph`
- `InputSignal`
- `ComputedSignal`
- `Signal`

### `P1`

- `ComputeContext`

Notes:

- `ReactiveGraph` is publicly documented as convenience-only, not kernel-grade
- this is still a real top-level product surface and will strongly influence
  first impressions

---

## `worth_signal::diagnostics`

Source: [`crates/worth-signal/src/diagnostics/mod.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/diagnostics/mod.rs)

### Direct public submodules

- `comparison`
- `inspection`
- `model`
- `policy`

### Re-export groups

- `compare`
- `diff`
- `access`
- `display`
- `history`
- `epochs`
- `facts`
- `failure`
- `flow`
- `lineage`
- `profile`
- `replay`
- `summary`

### Public symbols present here but not surfaced in `facade::diagnostics`

Classified as `P1` unless later narrowed:

- `ExplanationFact`
- `ProvenanceFact`
- `RetainedLineageView`
- `SynthesizedLineageChain`
- `ReplayEvent`
- `RetainedReplayView`
- `SynthesizedReplaySlice`

Assessment:

- `worth_signal::diagnostics` is broader than `facade::diagnostics`
- if we want strict curation, `diagnostics` itself needs a product-boundary pass

---

## Edge Cases

### Feature-gated public export

- `facade::planning::ParallelExecutionPolicy` is only available with
  `feature = "parallel"`

### Test-only public export

From [`crates/worth-signal/src/facade.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/facade.rs):

- `GraphDependencyBatchExt` is only `pub` under `#[cfg(test)]`

This is not part of the shipped public library surface.

### Convenience-only public export

From [`crates/worth-signal/src/easy/runtime.rs`](/Users/spenstar/Documents/programming/WORTH/WORTH/crates/worth-signal/src/easy/runtime.rs):

- `ReactiveGraph` has a doc-only deprecation note signaling that it is not the
  production runtime surface

This is an important messaging clue:

- the crate already knows this surface is convenience-only
- the public docs and namespace strategy should reinforce that consistently

---

## Highest-Confidence Findings

1. `facade::harness` is overwhelmingly `P3`.
2. `facade::transaction` is mixing `P0` runtime operations with a large `P2`
   merge and reconstructability surface.
3. `facade::types` is carrying too much specialist machinery for a namespace
   that sounds foundational and everyday.
4. `worth_signal::diagnostics` is a second major public boundary and is broader
   than the curated diagnostics namespace inside `facade`.
5. The crate already has enough surface for power users. The bigger problem is
   curation and presentation, not lack of knobs.

---

## Recommended Next Audit Step

Use this document as the source of truth for a keep/hide/move plan:

1. decide which `P3` surfaces leave the main facade entirely
2. decide which `P2` surfaces move into narrower specialist namespaces
3. tighten `P0` into the product story users see first
