use worth_runtime_bridge::facade::{
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchItem, BridgeDeliveryReceipt,
    BridgeRuntimePolicy, BridgeSignalInvalidationDelivery, BridgeSnapshotReadError,
    BridgeSourceAdapter, BridgeSourceCapability, BridgeSourceCapabilitySet, CoarseRoutingMode,
    InvalidationSink, RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadPacketResult,
    SnapshotReadRecord, SnapshotReadSource, SourceDeclaration, SourceDeclarationIdentity,
    TruthBranchHeadSource, TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity,
    TruthPatchScope, TruthSnapshotIdentity, TruthSnapshotReader,
};

#[derive(Clone)]
struct StaticSource;

impl worth_runtime_bridge::facade::CommittedPatchSource for StaticSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(BridgeCommittedPatchEnvelope::new(
            worth_runtime_bridge::facade::BridgeCommittedPatchEnvelopeIdentity::new(
                request.commit_identity().clone(),
                TruthPatchIdentity::from_relational_patch_position(1),
                TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
                TruthBranchIdentity::from_bridge_harness_label("analysis"),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                "entity-1",
                worth_runtime_bridge::facade::BridgeCommittedPatchTarget::entity_field_path(
                    worth_foundational::facade::AspectLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid native bridge patch aspect key"),
                    ),
                    worth_foundational::facade::CanonicalFieldPath::single(
                        worth_foundational::facade::FieldKey::new("name".to_owned())
                            .expect("valid native bridge patch field key"),
                    ),
                ),
            )],
        )
        .expect("native bridge patch envelope fixture must construct"))
    }
}

#[derive(Clone)]
struct StaticSnapshotReader;

impl TruthSnapshotReader for StaticSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a")
    }

    fn read_packet(
        &self,
        request: &worth_runtime_bridge::facade::SnapshotReadPacket,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        Ok(SnapshotReadPacketResult::new(
            TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
            request
                .reads()
                .iter()
                .map(|read| {
                    SnapshotReadRecord::for_request(
                        read,
                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                            "fixture",
                        ),
                    )
                })
                .collect(),
        ))
    }
}

impl SnapshotReadSource for StaticSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if identity == &TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a") {
            Ok(Box::new(StaticSnapshotReader))
        } else {
            Err(RelationalBridgeSourceError::new(format!(
                "unknown snapshot `{}`",
                identity
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting()
            )))
        }
    }
}

impl TruthBranchHeadSource for StaticSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(BridgeCommittedPatchEnvelope::new(
            worth_runtime_bridge::facade::BridgeCommittedPatchEnvelopeIdentity::new(
                TruthCommitIdentity::from_relational_commit_id(100),
                TruthPatchIdentity::from_relational_patch_position(100),
                TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
                branch_identity.clone(),
            ),
            vec![BridgeCommittedPatchItem::with_target(
                "entity-1",
                worth_runtime_bridge::facade::BridgeCommittedPatchTarget::entity_field_path(
                    worth_foundational::facade::AspectLocator::new(
                        worth_foundational::facade::LocatorAuthority::Authoritative,
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid native bridge patch aspect key"),
                    ),
                    worth_foundational::facade::CanonicalFieldPath::single(
                        worth_foundational::facade::FieldKey::new("name".to_owned())
                            .expect("valid native bridge patch field key"),
                    ),
                ),
            )],
        )
        .expect("native bridge branch head envelope fixture must construct"))
    }
}

#[derive(Clone)]
struct StaticSourceAdapter;

impl BridgeSourceAdapter for StaticSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::HistoricalRead,
            BridgeSourceCapability::BranchRead,
            BridgeSourceCapability::ReplayContinuityRead,
        ])
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if identity
            .bridge_admission_evidence()
            .terminal_projection_for_reporting()
            == "snapshot-a"
        {
            Ok(Box::new(StaticSnapshotReader))
        } else {
            Err(RelationalBridgeSourceError::new(format!(
                "unknown snapshot `{}`",
                identity
                    .bridge_admission_evidence()
                    .terminal_projection_for_reporting()
            )))
        }
    }
}

struct StaticSink;

impl InvalidationSink for StaticSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub(super) fn runtime(policy: BridgeRuntimePolicy) -> RuntimeBridge {
    RuntimeBridgeBuilder::new()
        .with_policy(policy)
        .with_relational_source(StaticSource)
        .with_source_adapter(StaticSourceAdapter)
        .with_truth_branch_head_source(StaticSource)
        .with_signal_sink(StaticSink)
        .register_source(registered_source(
            "source:analysis-snapshot",
            worth_runtime_bridge::facade::BridgeTruthViewSelector::branch_snapshot(
                TruthBranchIdentity::from_bridge_harness_label("analysis"),
                TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ],
        ))
        .register_source(registered_source(
            "source:analysis-history",
            worth_runtime_bridge::facade::BridgeTruthViewSelector::historical_commit(
                TruthBranchIdentity::from_bridge_harness_label("analysis"),
                TruthCommitIdentity::from_bridge_harness_label("commit-a"),
            ),
            vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::HistoricalRead,
                BridgeSourceCapability::BranchRead,
                BridgeSourceCapability::ReplayContinuityRead,
            ],
        ))
        .register_mapping(
            worth_runtime_bridge::facade::BridgeMappingRegistration::new(
                worth_runtime_bridge::facade::BridgeMappingId::from_stable_name("mapping"),
                TruthPatchScope::new(
                    worth_runtime_bridge::facade::MappingSelector::exact("entity-1"),
                    worth_runtime_bridge::facade::AspectKeySelector::exact(
                        worth_foundational::facade::AspectKey::new("profile")
                            .expect("valid native mapping aspect key"),
                    ),
                    worth_runtime_bridge::facade::TruthPatchTargetSelector::entity_field(
                        worth_foundational::facade::FieldKey::new("name".to_owned())
                            .expect("valid native mapping field key"),
                    ),
                ),
                worth_runtime_bridge::facade::SnapshotReadContract::scalar(
                    worth_foundational::facade::AspectKey::new("profile")
                        .expect("valid native snapshot aspect key"),
                    worth_foundational::facade::ScalarAspectType::String,
                ),
                SignalInvalidationScope::from_stable_name("signal:profile"),
                CoarseRoutingMode::Direct,
            ),
        )
        .build()
        .expect("runtime should build for historical lowering tests")
}

fn registered_source(
    id: &str,
    selector: worth_runtime_bridge::facade::BridgeTruthViewSelector,
    capabilities: Vec<BridgeSourceCapability>,
) -> SourceDeclaration {
    SourceDeclaration::new(
        SourceDeclarationIdentity::from_stable_name(id),
        selector,
        BridgeSourceCapabilitySet::new(capabilities),
    )
}
