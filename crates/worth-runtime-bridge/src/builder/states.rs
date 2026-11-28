use super::*;

#[derive(Clone)]
pub struct MissingCommittedPatchSource;

#[derive(Clone)]
pub struct PresentCommittedPatchSource(pub(super) Arc<dyn CommittedPatchSource>);

#[derive(Clone)]
pub struct MissingSnapshotReadSource;

#[derive(Clone)]
pub struct PresentSnapshotReadSource(pub(super) Arc<dyn SnapshotReadSource>);

#[derive(Clone)]
pub struct MissingSignalSink;

#[derive(Clone)]
pub struct PresentSignalSink(pub(super) Arc<dyn InvalidationSink>);

#[derive(Clone)]
pub struct MissingTruthBranchHeadSource;

#[derive(Clone)]
pub struct PresentTruthBranchHeadSource(pub(super) Arc<dyn TruthBranchHeadSource>);

#[derive(Clone)]
pub struct MissingMappingRegistrations;

#[derive(Clone)]
pub struct PresentMappingRegistrations(pub(super) Vec<BridgeMappingRegistration>);

#[derive(Clone)]
pub struct RuntimeBridgeBuilder<
    PatchState = MissingCommittedPatchSource,
    SnapshotState = MissingSnapshotReadSource,
    SignalState = MissingSignalSink,
    BranchHeadState = MissingTruthBranchHeadSource,
    MappingState = MissingMappingRegistrations,
> {
    pub(super) diagnostic_sink: Arc<dyn worth_execution::report::DiagnosticSink>,
    pub(super) policy: BridgeRuntimePolicy,
    pub(super) committed_patch_source: PatchState,
    pub(super) snapshot_read_source: SnapshotState,
    pub(super) signal_sink: SignalState,
    pub(super) truth_branch_head_source: BranchHeadState,
    pub(super) continuity_lineage_source: Option<Arc<dyn ContinuityLineageSource>>,
    pub(super) writeback_authority: Option<Arc<dyn TruthWritebackAuthority>>,
    pub(super) snapshot_reader_pool: Option<Arc<dyn SnapshotReaderPool>>,
    pub(super) source_adapter_registrations: Vec<Arc<dyn BridgeSourceAdapter>>,
    pub(super) source_declarations: Vec<SourceDeclaration>,
    pub(super) mapping_registrations: MappingState,
    pub(super) aspect_registrations: Vec<BridgeAspectRegistration>,
    pub(super) semantic_dependency_registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
}

impl Default
    for RuntimeBridgeBuilder<
        MissingCommittedPatchSource,
        MissingSnapshotReadSource,
        MissingSignalSink,
        MissingTruthBranchHeadSource,
        MissingMappingRegistrations,
    >
{
    fn default() -> Self {
        Self::new()
    }
}

impl
    RuntimeBridgeBuilder<
        MissingCommittedPatchSource,
        MissingSnapshotReadSource,
        MissingSignalSink,
        MissingTruthBranchHeadSource,
        MissingMappingRegistrations,
    >
{
    pub fn new() -> Self {
        Self {
            diagnostic_sink: Arc::new(worth_execution::report::NoopDiagnosticSink),
            policy: BridgeRuntimePolicy::default(),
            committed_patch_source: MissingCommittedPatchSource,
            snapshot_read_source: MissingSnapshotReadSource,
            signal_sink: MissingSignalSink,
            truth_branch_head_source: MissingTruthBranchHeadSource,
            continuity_lineage_source: None,
            writeback_authority: None,
            snapshot_reader_pool: None,
            source_adapter_registrations: Vec::new(),
            source_declarations: Vec::new(),
            mapping_registrations: MissingMappingRegistrations,
            aspect_registrations: Vec::new(),
            semantic_dependency_registrations: Vec::new(),
        }
    }
}

impl<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState> std::fmt::Debug
    for RuntimeBridgeBuilder<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeBridgeBuilder")
            .field("policy", &self.policy)
            .field(
                "aspect_registration_count",
                &self.aspect_registrations.len(),
            )
            .finish()
    }
}

impl<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState>
    RuntimeBridgeBuilder<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState>
{
    /// Receives operator reports disclosed at the installed diagnostics tier.
    pub fn with_diagnostic_sink(
        mut self,
        sink: Arc<dyn worth_execution::report::DiagnosticSink>,
    ) -> Self {
        self.diagnostic_sink = sink;
        self
    }
}
