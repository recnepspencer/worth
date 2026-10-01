use std::sync::Arc;
use worth_foundational::facade::{
    AspectKey, AspectLocator, CanonicalFieldPath, FieldKey, LocatorAuthority, ScalarAspectType,
};
use worth_runtime_bridge::facade::{
    AspectKeySelector, BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity,
    BridgeCommittedPatchItem, BridgeCommittedPatchTarget, BridgeDeliveryReceipt, BridgeMappingId,
    BridgeMappingRegistration, BridgeRuntimePolicy, BridgeSignalInvalidationDelivery,
    BridgeSnapshotReadError, BridgeSourceAdapter, BridgeSourceCapability,
    BridgeSourceCapabilitySet, BridgeTruthViewSelector, CoarseRoutingMode, InvalidationSink,
    MappingSelector, RelationalBridgeRecordIdentityParts, RelationalBridgeSnapshotIdentityParts,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract,
    SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadSource,
    SourceDeclaration, SourceDeclarationIdentity, TruthBranchHeadSource, TruthBranchIdentity,
    TruthCommitIdentity, TruthPatchIdentity, TruthPatchScope, TruthPatchTargetSelector,
    TruthSnapshotIdentity, TruthSnapshotReader,
};

type ProjectionBridgeRow = (RelationalBridgeRecordIdentityParts, String, String);

pub(super) fn projection_bridge_runtime() -> RuntimeBridge {
    let rows = Arc::new(vec![
        (
            RelationalBridgeRecordIdentityParts::entity(1, 1, 1),
            "task-1".to_string(),
            "todo".to_string(),
        ),
        (
            RelationalBridgeRecordIdentityParts::entity(1, 2, 1),
            "task-2".to_string(),
            "doing".to_string(),
        ),
    ]);
    RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(ProjectionBridgeSource { rows: rows.clone() })
        .with_source_adapter(ProjectionBridgeSourceAdapter { rows: rows.clone() })
        .with_truth_branch_head_source(ProjectionBridgeSource { rows })
        .with_signal_sink(ProjectionBridgeSink)
        .register_source(SourceDeclaration::new(
            SourceDeclarationIdentity::from_stable_name("source:lower-runtime-certification"),
            BridgeTruthViewSelector::branch_snapshot(
                TruthBranchIdentity::from_relational_branch_id("main"),
                projection_snapshot_identity(),
            ),
            BridgeSourceCapabilitySet::new(vec![
                BridgeSourceCapability::SnapshotRead,
                BridgeSourceCapability::BranchRead,
            ]),
        ))
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("projection-bridge-mapping"),
            TruthPatchScope::new(
                MappingSelector::exact("entity-1"),
                AspectKeySelector::exact(aspect_key("status")),
                TruthPatchTargetSelector::entity_field(field_key("lane")),
            ),
            SnapshotReadContract::scalar(aspect_key("status"), ScalarAspectType::String),
            SignalInvalidationScope::from_stable_name("signal:projection-bridge"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("projection bridge runtime should build")
}

#[derive(Clone)]
struct ProjectionBridgeSource {
    rows: Arc<Vec<ProjectionBridgeRow>>,
}

impl worth_runtime_bridge::facade::CommittedPatchSource for ProjectionBridgeSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(native_projection_patch_envelope(
            request.commit_identity().clone(),
            TruthPatchIdentity::from_relational_patch_position(79),
            projection_snapshot_identity(),
            TruthBranchIdentity::from_relational_branch_id("main"),
        ))
    }
}

#[derive(Clone)]
struct ProjectionBridgeSnapshotReader {
    rows: Arc<Vec<ProjectionBridgeRow>>,
}

impl TruthSnapshotReader for ProjectionBridgeSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        projection_snapshot_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        Ok(SnapshotReadPacketResult::new(
            projection_snapshot_identity(),
            request
                .reads()
                .iter()
                .map(|read| {
                    let payload = self
                        .rows
                        .iter()
                        .find_map(|(record_identity, identity_value, grouping_value)| {
                            (read.relational_record_identity_parts() == Some(*record_identity))
                                .then(|| match read.aspect_key().as_str() {
                                    "identity.id" => {
                                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                                            identity_value.as_str(),
                                        )
                                    }
                                    "status" => {
                                        crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                                            grouping_value.as_str(),
                                        )
                                    }
                                    _ => crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                                        "unknown",
                                    ),
                                })
                        })
                        .unwrap_or_else(|| {
                            crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("unknown")
                        });
                    SnapshotReadRecord::for_request(read, payload)
                })
                .collect(),
        ))
    }
}

impl SnapshotReadSource for ProjectionBridgeSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if identity.relational_snapshot_parts().is_some() {
            Ok(Box::new(ProjectionBridgeSnapshotReader {
                rows: self.rows.clone(),
            }))
        } else {
            Err(RelationalBridgeSourceError::new(format!(
                "unknown snapshot `{:?}`",
                identity
            )))
        }
    }
}

impl TruthBranchHeadSource for ProjectionBridgeSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(native_projection_patch_envelope(
            TruthCommitIdentity::from_relational_commit_id(80),
            TruthPatchIdentity::from_relational_patch_position(80),
            projection_snapshot_identity(),
            branch_identity.clone(),
        ))
    }
}

#[derive(Clone)]
struct ProjectionBridgeSourceAdapter {
    rows: Arc<Vec<ProjectionBridgeRow>>,
}

impl BridgeSourceAdapter for ProjectionBridgeSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ])
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        ProjectionBridgeSource {
            rows: self.rows.clone(),
        }
        .open_snapshot(identity)
    }
}

fn native_projection_patch_envelope(
    commit_identity: TruthCommitIdentity,
    patch_identity: TruthPatchIdentity,
    snapshot_identity: TruthSnapshotIdentity,
    branch_identity: TruthBranchIdentity,
) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new(
            commit_identity,
            patch_identity,
            snapshot_identity,
            branch_identity,
        ),
        vec![BridgeCommittedPatchItem::with_target(
            "entity-1",
            BridgeCommittedPatchTarget::entity_field_path(
                AspectLocator::new(LocatorAuthority::Authoritative, aspect_key("status")),
                CanonicalFieldPath::single(field_key("lane")),
            ),
        )],
    )
    .expect("projection bridge fixture must build a native patch envelope")
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).expect("valid projection bridge aspect key")
}

fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value.to_owned()).expect("valid projection bridge field key")
}

fn projection_snapshot_identity() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        6, 2,
    ))
}

struct ProjectionBridgeSink;

impl InvalidationSink for ProjectionBridgeSink {
    fn deliver_invalidation(
        &self,
        delivery: BridgeSignalInvalidationDelivery,
        _lease: Option<&worth_runtime_bridge::facade::ExecutionResourceLease<'_>>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}
