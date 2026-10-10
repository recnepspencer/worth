use super::*;

#[derive(Clone)]
struct FixtureSourceAdapter {
    capabilities: BridgeSourceCapabilitySet,
    source: super::super::super::fixtures::InMemoryRelationalBridgeSource,
}

#[derive(Clone)]
struct WrappedFixtureSourceAdapter {
    inner: SessionSourceAdapter,
}

struct DriftFixtureSnapshotReader {
    inner: Box<dyn crate::facade::TruthSnapshotReader>,
}

#[derive(Clone)]
struct SessionSourceAdapter {
    runtime: BridgeHarnessSession,
    inner: FixtureSourceAdapter,
}

pub(super) fn prepare_runtime_profile(
    runtime: &mut BridgeHarnessSession,
    profile: &ExecutionProfile,
) -> Result<(), BridgeHarnessError> {
    runtime.source_adapter_shape = match profile
        .metadata
        .get("source_adapter_shape")
        .map(String::as_str)
    {
        Some("wrapped") => SourceAdapterShape::Wrapped,
        _ => SourceAdapterShape::Direct,
    };
    runtime.source_builder_load_order = match profile
        .metadata
        .get("source_builder_load_order")
        .map(String::as_str)
    {
        Some("sources_first") => SourceBuilderLoadOrder::SourcesBeforeAdapter,
        _ => SourceBuilderLoadOrder::AdapterBeforeSources,
    };
    runtime.policy_builder_load_order = match profile
        .metadata
        .get("policy_builder_load_order")
        .map(String::as_str)
    {
        Some("sections_canonical") => PolicyBuilderLoadOrder::SectionsCanonical,
        Some("sections_reverse") => PolicyBuilderLoadOrder::SectionsReverse,
        _ => PolicyBuilderLoadOrder::WholePolicy,
    };
    runtime.source_adapter_behavior = match profile
        .metadata
        .get("source_adapter_behavior")
        .map(String::as_str)
    {
        Some("reject_open_snapshot") => SourceAdapterBehavior::RejectOpenSnapshot,
        Some("drift_snapshot_identity") => SourceAdapterBehavior::DriftSnapshotIdentity,
        _ => SourceAdapterBehavior::Honest,
    };
    match profile.execution_mode {
        ExecutionMode::RuntimeDefault | ExecutionMode::Serial => Ok(()),
        mode => Err(BridgeHarnessError::new(format!(
            "bridge harness does not support execution mode `{mode:?}`"
        ))),
    }
}

pub(super) fn load_bridge_fixture(
    runtime: &mut BridgeHarnessSession,
    fixture: &ScenarioFixture<BridgeHarnessFixture>,
) -> Result<(), BridgeHarnessError> {
    load_fixture_truth(runtime, fixture);

    let builder = apply_policy_builder_load_order(
        RuntimeBridgeBuilder::new()
            .with_relational_source(runtime.source.clone())
            .with_truth_branch_head_source(runtime.source.clone())
            .with_signal_sink(runtime.sink.clone())
            .with_writeback_authority(runtime.writeback_authority.clone())
            .with_continuity_lineage_source(runtime.source.clone()),
        runtime,
        fixture.fixture.policy(),
    );
    let (first_mapping, remaining_mappings) =
        fixture.fixture.mappings().split_first().ok_or_else(|| {
            BridgeHarnessError::new("bridge harness fixture requires at least one mapping")
        })?;
    let mut builder = builder.register_mapping(first_mapping.clone());
    for mapping in remaining_mappings {
        builder = builder.register_mapping(mapping.clone());
    }
    for aspect_mapping in fixture.fixture.aspect_mappings() {
        builder = builder.register_aspect_mapping(aspect_mapping.clone());
    }
    for structural_declaration in fixture.fixture.structural_declarations() {
        builder = builder.register_structural(structural_declaration.clone());
    }
    for merge_declaration in fixture.fixture.merge_declarations() {
        builder = builder.register_merge(merge_declaration.clone());
    }
    if !fixture.fixture.source_declarations().is_empty() {
        let source_adapter = FixtureSourceAdapter {
            capabilities: fixture_source_adapter_capabilities(fixture),
            source: runtime.source.clone(),
        };
        builder = match runtime.source_builder_load_order {
            SourceBuilderLoadOrder::AdapterBeforeSources => {
                let mut builder = with_session_source_adapter(builder, runtime, source_adapter);
                for declaration in fixture.fixture.source_declarations() {
                    builder = builder.register_source(declaration.clone());
                }
                builder
            }
            SourceBuilderLoadOrder::SourcesBeforeAdapter => {
                let mut builder = builder;
                for declaration in fixture.fixture.source_declarations() {
                    builder = builder.register_source(declaration.clone());
                }
                with_session_source_adapter(builder, runtime, source_adapter)
            }
        };
    }
    runtime.runtime = Some(builder.build().map_err(|error| {
        BridgeHarnessError::new(format!("failed to build bridge runtime: {error}"))
    })?);
    Ok(())
}

pub(super) fn apply_bridge_mutation_batch(
    runtime: &mut BridgeHarnessSession,
    batch: &MutationBatch<BridgeHarnessMutation>,
) -> Result<(), BridgeHarnessError> {
    for operation in &batch.operations {
        match operation {
            BridgeHarnessMutation::PublishCommittedPatch(patch) => {
                runtime.source.insert_committed_patch(patch.clone());
            }
            BridgeHarnessMutation::PublishSnapshot(snapshot) => {
                runtime.source.insert_snapshot(snapshot.clone());
            }
        }
    }
    Ok(())
}

fn load_fixture_truth(
    runtime: &mut BridgeHarnessSession,
    fixture: &ScenarioFixture<BridgeHarnessFixture>,
) {
    for patch in fixture.fixture.committed_patches() {
        runtime.source.insert_committed_patch(patch.clone());
    }
    for snapshot in fixture.fixture.snapshots() {
        runtime.source.insert_snapshot(snapshot.clone());
    }
    for (entity_identity, authority) in fixture.fixture.continuity_authorities() {
        runtime
            .source
            .insert_continuity_authority(entity_identity.clone(), authority.clone());
    }
}

fn fixture_source_adapter_capabilities(
    fixture: &ScenarioFixture<BridgeHarnessFixture>,
) -> BridgeSourceCapabilitySet {
    fixture
        .fixture
        .source_adapter_capabilities()
        .cloned()
        .unwrap_or_else(|| {
            let mut capabilities = Vec::new();
            for declaration in fixture.fixture.source_declarations() {
                capabilities.extend_from_slice(declaration.required_capabilities().capabilities());
            }
            BridgeSourceCapabilitySet::new(capabilities)
        })
}

fn with_session_source_adapter<
    PatchState,
    SnapshotState,
    SignalState,
    BranchHeadState,
    MappingState,
>(
    builder: RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        SignalState,
        BranchHeadState,
        MappingState,
    >,
    runtime: &BridgeHarnessSession,
    adapter: FixtureSourceAdapter,
) -> RuntimeBridgeBuilder<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState> {
    let adapter = SessionSourceAdapter {
        runtime: runtime.clone(),
        inner: adapter,
    };
    match runtime.source_adapter_shape {
        SourceAdapterShape::Direct => builder.with_source_adapter(adapter),
        SourceAdapterShape::Wrapped => {
            builder.with_source_adapter(WrappedFixtureSourceAdapter { inner: adapter })
        }
    }
}

fn apply_policy_builder_load_order<
    PatchState,
    SnapshotState,
    SignalState,
    BranchHeadState,
    MappingState,
>(
    builder: RuntimeBridgeBuilder<
        PatchState,
        SnapshotState,
        SignalState,
        BranchHeadState,
        MappingState,
    >,
    runtime: &BridgeHarnessSession,
    policy: crate::facade::BridgeRuntimePolicy,
) -> RuntimeBridgeBuilder<PatchState, SnapshotState, SignalState, BranchHeadState, MappingState> {
    match runtime.policy_builder_load_order {
        PolicyBuilderLoadOrder::WholePolicy => builder.with_policy(policy),
        PolicyBuilderLoadOrder::SectionsCanonical => builder
            .with_execution_policy_baseline(policy.execution())
            .with_diagnostics_policy_baseline(policy.diagnostics())
            .with_artifact_policy_baseline(policy.artifacts()),
        PolicyBuilderLoadOrder::SectionsReverse => builder
            .with_artifact_policy_baseline(policy.artifacts())
            .with_diagnostics_policy_baseline(policy.diagnostics())
            .with_execution_policy_baseline(policy.execution()),
    }
}

impl crate::adapter::BridgeSourceAdapter for FixtureSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        self.capabilities.clone()
    }

    fn open_snapshot(
        &self,
        identity: &crate::facade::TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Box<dyn crate::facade::TruthSnapshotReader>,
        crate::adapter::RelationalBridgeSourceError,
    > {
        crate::adapter::SnapshotReadSource::open_snapshot(&self.source, identity, execution)
    }
}

impl crate::facade::TruthSnapshotReader for DriftFixtureSnapshotReader {
    fn snapshot_identity(&self) -> crate::facade::TruthSnapshotIdentity {
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-drift")
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::facade::SnapshotReadPacketResult, crate::facade::BridgeSnapshotReadError>
    {
        self.inner.read_packet(request, execution)
    }
}

impl crate::adapter::BridgeSourceAdapter for SessionSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        self.inner.declared_capabilities()
    }

    fn open_snapshot(
        &self,
        identity: &crate::facade::TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Box<dyn crate::facade::TruthSnapshotReader>,
        crate::adapter::RelationalBridgeSourceError,
    > {
        match self.runtime.source_adapter_behavior {
            SourceAdapterBehavior::Honest => self.inner.open_snapshot(identity, execution),
            SourceAdapterBehavior::RejectOpenSnapshot => {
                Err(crate::adapter::RelationalBridgeSourceError::new(
                    crate::adapter::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                    format!(
                        "session source adapter refused snapshot `{}`",
                        identity.as_str()
                    ),
                ))
            }
            SourceAdapterBehavior::DriftSnapshotIdentity => {
                let reader = self.inner.open_snapshot(identity, execution)?;
                Ok(Box::new(DriftFixtureSnapshotReader { inner: reader }))
            }
        }
    }
}

impl crate::adapter::BridgeSourceAdapter for WrappedFixtureSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        self.inner.declared_capabilities()
    }

    fn open_snapshot(
        &self,
        identity: &crate::facade::TruthSnapshotIdentity,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Box<dyn crate::facade::TruthSnapshotReader>,
        crate::adapter::RelationalBridgeSourceError,
    > {
        self.inner.open_snapshot(identity, execution)
    }
}
