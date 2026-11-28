use super::*;

impl<SnapshotState, SignalState, BranchHeadState, MappingState>
    RuntimeBridgeBuilder<
        MissingCommittedPatchSource,
        SnapshotState,
        SignalState,
        BranchHeadState,
        MappingState,
    >
{
    pub fn with_committed_patch_source<S>(
        self,
        source: S,
    ) -> RuntimeBridgeBuilder<
        PresentCommittedPatchSource,
        SnapshotState,
        SignalState,
        BranchHeadState,
        MappingState,
    >
    where
        S: CommittedPatchSource,
    {
        RuntimeBridgeBuilder {
            policy: self.policy,
            diagnostic_sink: self.diagnostic_sink,
            committed_patch_source: PresentCommittedPatchSource(Arc::new(source)),
            snapshot_read_source: self.snapshot_read_source,
            signal_sink: self.signal_sink,
            truth_branch_head_source: self.truth_branch_head_source,
            continuity_lineage_source: self.continuity_lineage_source,
            writeback_authority: self.writeback_authority,
            snapshot_reader_pool: self.snapshot_reader_pool,
            source_adapter_registrations: self.source_adapter_registrations,
            source_declarations: self.source_declarations,
            structural_declarations: self.structural_declarations,
            mapping_registrations: self.mapping_registrations,
            aspect_registrations: self.aspect_registrations,
            semantic_dependency_registrations: self.semantic_dependency_registrations,
        }
    }
}

impl<PatchState, SignalState, BranchHeadState, MappingState>
    RuntimeBridgeBuilder<
        PatchState,
        MissingSnapshotReadSource,
        SignalState,
        BranchHeadState,
        MappingState,
    >
{
    pub fn with_snapshot_read_source<S>(
        self,
        source: S,
    ) -> RuntimeBridgeBuilder<
        PatchState,
        PresentSnapshotReadSource,
        SignalState,
        BranchHeadState,
        MappingState,
    >
    where
        S: SnapshotReadSource,
    {
        RuntimeBridgeBuilder {
            policy: self.policy,
            diagnostic_sink: self.diagnostic_sink,
            committed_patch_source: self.committed_patch_source,
            snapshot_read_source: PresentSnapshotReadSource(Arc::new(source)),
            signal_sink: self.signal_sink,
            truth_branch_head_source: self.truth_branch_head_source,
            continuity_lineage_source: self.continuity_lineage_source,
            writeback_authority: self.writeback_authority,
            snapshot_reader_pool: self.snapshot_reader_pool,
            source_adapter_registrations: self.source_adapter_registrations,
            source_declarations: self.source_declarations,
            structural_declarations: self.structural_declarations,
            mapping_registrations: self.mapping_registrations,
            aspect_registrations: self.aspect_registrations,
            semantic_dependency_registrations: self.semantic_dependency_registrations,
        }
    }
}

impl<SignalState, MappingState>
    RuntimeBridgeBuilder<
        MissingCommittedPatchSource,
        MissingSnapshotReadSource,
        SignalState,
        MissingTruthBranchHeadSource,
        MappingState,
    >
{
    pub fn with_relational_source<S>(
        self,
        source: S,
    ) -> RuntimeBridgeBuilder<
        PresentCommittedPatchSource,
        PresentSnapshotReadSource,
        SignalState,
        MissingTruthBranchHeadSource,
        MappingState,
    >
    where
        S: CommittedPatchSource + SnapshotReadSource,
    {
        let source = Arc::new(source);
        RuntimeBridgeBuilder {
            policy: self.policy,
            diagnostic_sink: self.diagnostic_sink,
            committed_patch_source: PresentCommittedPatchSource(source.clone()),
            snapshot_read_source: PresentSnapshotReadSource(source.clone()),
            signal_sink: self.signal_sink,
            truth_branch_head_source: MissingTruthBranchHeadSource,
            continuity_lineage_source: self.continuity_lineage_source,
            writeback_authority: self.writeback_authority,
            snapshot_reader_pool: self.snapshot_reader_pool,
            source_adapter_registrations: self.source_adapter_registrations,
            source_declarations: self.source_declarations,
            structural_declarations: self.structural_declarations,
            mapping_registrations: self.mapping_registrations,
            aspect_registrations: self.aspect_registrations,
            semantic_dependency_registrations: self.semantic_dependency_registrations,
        }
    }

    pub fn with_truth_source<S>(
        self,
        source: S,
    ) -> RuntimeBridgeBuilder<
        PresentCommittedPatchSource,
        PresentSnapshotReadSource,
        SignalState,
        MissingTruthBranchHeadSource,
        MappingState,
    >
    where
        S: CommittedPatchSource + SnapshotReadSource,
    {
        self.with_relational_source(source)
    }
}

impl<PatchState, SnapshotState, BranchHeadState, MappingState>
    RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        MissingSignalSink,
        BranchHeadState,
        MappingState,
    >
{
    pub fn with_signal_sink<S>(
        self,
        sink: S,
    ) -> RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        PresentSignalSink,
        BranchHeadState,
        MappingState,
    >
    where
        S: InvalidationSink,
    {
        RuntimeBridgeBuilder {
            policy: self.policy,
            diagnostic_sink: self.diagnostic_sink,
            committed_patch_source: self.committed_patch_source,
            snapshot_read_source: self.snapshot_read_source,
            signal_sink: PresentSignalSink(Arc::new(sink)),
            truth_branch_head_source: self.truth_branch_head_source,
            continuity_lineage_source: self.continuity_lineage_source,
            writeback_authority: self.writeback_authority,
            snapshot_reader_pool: self.snapshot_reader_pool,
            source_adapter_registrations: self.source_adapter_registrations,
            source_declarations: self.source_declarations,
            structural_declarations: self.structural_declarations,
            mapping_registrations: self.mapping_registrations,
            aspect_registrations: self.aspect_registrations,
            semantic_dependency_registrations: self.semantic_dependency_registrations,
        }
    }

    pub fn with_compute_sink<S>(
        self,
        sink: S,
    ) -> RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        PresentSignalSink,
        BranchHeadState,
        MappingState,
    >
    where
        S: InvalidationSink,
    {
        self.with_signal_sink(sink)
    }
}

impl<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState>
    RuntimeBridgeBuilder<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState>
{
    pub fn with_policy(mut self, policy: BridgeRuntimePolicy) -> Self {
        self.policy = super::policy::replace_runtime_policy(self.policy, policy);
        self
    }

    pub fn with_execution_policy_baseline(
        mut self,
        execution: crate::policy::BridgeExecutionPolicyBaseline,
    ) -> Self {
        self.policy = super::policy::replace_execution_policy_baseline(self.policy, execution);
        self
    }

    pub fn with_diagnostics_policy_baseline(
        mut self,
        diagnostics: crate::policy::BridgeDiagnosticsPolicyBaseline,
    ) -> Self {
        self.policy = super::policy::replace_diagnostics_policy_baseline(self.policy, diagnostics);
        self
    }

    pub fn with_artifact_policy_baseline(
        mut self,
        artifacts: crate::policy::BridgeArtifactPolicyBaseline,
    ) -> Self {
        self.policy = super::policy::replace_artifact_policy_baseline(self.policy, artifacts);
        self
    }

    pub fn with_conditional_retention_budget(
        mut self,
        budget: crate::policy::BridgeConditionalRetentionBudget,
    ) -> Self {
        self.policy = self.policy.with_conditional_retention(budget);
        self
    }

    pub fn with_snapshot_reader_pool<P>(mut self, pool: P) -> Self
    where
        P: SnapshotReaderPool,
    {
        self.snapshot_reader_pool = Some(Arc::new(pool));
        self
    }

    pub fn with_source_adapter<S>(mut self, source_adapter: S) -> Self
    where
        S: BridgeSourceAdapter,
    {
        self.source_adapter_registrations
            .push(Arc::new(source_adapter));
        self
    }

    pub fn with_continuity_lineage_source<S>(mut self, source: S) -> Self
    where
        S: ContinuityLineageSource,
    {
        self.continuity_lineage_source = Some(Arc::new(source));
        self
    }

    pub fn with_writeback_authority<S>(mut self, authority: S) -> Self
    where
        S: TruthWritebackAuthority,
    {
        self.writeback_authority = Some(Arc::new(authority));
        self
    }

    pub fn with_truth_branch_head_source<S>(
        self,
        source: S,
    ) -> RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        SignalState,
        PresentTruthBranchHeadSource,
        MappingState,
    >
    where
        S: TruthBranchHeadSource,
    {
        RuntimeBridgeBuilder {
            policy: self.policy,
            diagnostic_sink: self.diagnostic_sink,
            committed_patch_source: self.committed_patch_source,
            snapshot_read_source: self.snapshot_read_source,
            signal_sink: self.signal_sink,
            truth_branch_head_source: PresentTruthBranchHeadSource(Arc::new(source)),
            continuity_lineage_source: self.continuity_lineage_source,
            writeback_authority: self.writeback_authority,
            snapshot_reader_pool: self.snapshot_reader_pool,
            source_adapter_registrations: self.source_adapter_registrations,
            source_declarations: self.source_declarations,
            structural_declarations: self.structural_declarations,
            mapping_registrations: self.mapping_registrations,
            aspect_registrations: self.aspect_registrations,
            semantic_dependency_registrations: self.semantic_dependency_registrations,
        }
    }
}
