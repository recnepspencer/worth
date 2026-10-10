use worth_foundational::facade::{
    aspects, AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectLocator,
    AspectValue, CanonicalFieldPath, FieldKey, LocatorAuthority, ScalarAspectType,
    StructAspectValue,
};
use worth_runtime_bridge::facade::{
    materialize_bridge_row_set, AspectKeySelector, BridgeCommittedPatchEnvelope,
    BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem, BridgeCommittedPatchTarget,
    BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration, BridgeRuntimePolicy,
    BridgeSignalInvalidationDelivery, BridgeSnapshotReadError, BridgeSourceAdapter,
    BridgeSourceCapability, BridgeSourceCapabilitySet, BridgeTruthViewSelector, CoarseRoutingMode,
    CommittedPatchSource, InvalidationSink, MappingSelector, RelationalBridgeSnapshotIdentityParts,
    RelationalBridgeSourceError, RelationalCommittedPatchRequest, RuntimeBridge,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SnapshotReadContract,
    SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord, SnapshotReadRequest,
    SnapshotReadSource, SourceDeclaration, SourceDeclarationIdentity, TruthBranchHeadSource,
    TruthBranchIdentity, TruthCommitIdentity, TruthPatchIdentity, TruthPatchScope,
    TruthPatchTargetSelector, TruthSnapshotIdentity, TruthSnapshotReader,
};

use super::super::super::{
    ProjectMaterializedFacts, ProjectionConsumptionSource, ProjectionSourceFamily,
};
use super::support::{admitted, binding};

#[test]
fn bridge_row_set_preserves_complete_struct_values_through_consumption() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = bridge_runtime();
    let contract = runtime
        .admit_source(source_declaration())
        .expect("registered struct source should admit");
    let observation = runtime
        .materialize_source_packet(&contract, read_packet(), resource_request)
        .expect("struct source packet should materialize");
    let row_set = materialize_bridge_row_set(&observation, resource_request)
        .expect("bridge row set should build");
    let consumption = admitted(
        ProjectionConsumptionSource::from_bridge_truth_view_row_set(&row_set),
        binding(&["profile"]),
        ProjectMaterializedFacts::declare().derived_field_path(
            crate::projection_consumption::projection_fact_field_path_from_segments([field_key(
                "profile",
            )]),
        ),
    )
    .bind_contract();

    let consumed = consumption
        .extract_from_bridge_truth_view_row_set(&row_set)
        .expect("bridge struct should be consumable");
    let fact = &consumed.derived_fields()[0];

    assert_eq!(fact.as_struct().unwrap(), &profile_value());
    assert_eq!(
        fact.source_family(),
        ProjectionSourceFamily::BridgeTruthViewRowSet
    );
    assert_eq!(
        fact.field_path().canonical_field_path(),
        Some(&CanonicalFieldPath::single(field_key("profile")))
    );
    assert_eq!(fact.projection_authority(), consumption.contract_digest());
}

fn bridge_runtime() -> RuntimeBridge {
    let source = StructSource;
    RuntimeBridgeBuilder::new()
        .with_policy(BridgeRuntimePolicy::default())
        .with_relational_source(source.clone())
        .with_source_adapter(source.clone())
        .with_truth_branch_head_source(source)
        .with_signal_sink(StructSink)
        .register_source(source_declaration())
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("mapping:profile-struct"),
            TruthPatchScope::new(
                MappingSelector::exact("entity-1"),
                AspectKeySelector::exact(aspect_key("profile")),
                TruthPatchTargetSelector::entity_field(field_key("name")),
            ),
            profile_contract(),
            SignalInvalidationScope::from_stable_name("signal:profile-struct"),
            CoarseRoutingMode::Direct,
        ))
        .build()
        .expect("bridge struct runtime should build")
}

fn source_declaration() -> SourceDeclaration {
    SourceDeclaration::new(
        SourceDeclarationIdentity::from_stable_name("source:profile-struct"),
        BridgeTruthViewSelector::branch_snapshot(
            TruthBranchIdentity::from_relational_branch_id("main"),
            snapshot_identity(),
        ),
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ]),
    )
}

fn read_packet() -> SnapshotReadPacket {
    SnapshotReadPacket::new(vec![
        SnapshotReadRequest::for_coarse(
            "entity-1",
            SnapshotReadContract::scalar(aspect_key("identity.id"), ScalarAspectType::String),
        ),
        SnapshotReadRequest::for_coarse("entity-1", profile_contract()),
    ])
}

fn profile_contract() -> SnapshotReadContract {
    SnapshotReadContract::new(AspectContract::struct_aspect(
        aspect_key("profile"),
        AspectIdentity(41),
        AspectContractRevision(1),
        aspects()
            .struct_fields()
            .required("name", ScalarAspectType::String)
            .required("rank", ScalarAspectType::UInt32)
            .finish()
            .unwrap(),
    ))
}

fn profile_value() -> StructAspectValue {
    StructAspectValue::new([
        (field_key("name"), AspectValue::String("Ada".into())),
        (field_key("rank"), AspectValue::UInt32(7)),
    ])
    .unwrap()
}

#[derive(Clone)]
struct StructSource;

impl CommittedPatchSource for StructSource {
    fn load_committed_patch(
        &self,
        request: RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(patch_envelope(
            request.commit_identity().clone(),
            TruthPatchIdentity::from_relational_patch_position(41),
            TruthBranchIdentity::from_relational_branch_id("main"),
        ))
    }
}

impl SnapshotReadSource for StructSource {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        if identity == &snapshot_identity() {
            Ok(Box::new(StructSnapshotReader))
        } else {
            Err(RelationalBridgeSourceError::new(
                worth_runtime_bridge::facade::RelationalBridgeSourceErrorTag::ExternalSourceFailure,
                "unknown struct snapshot",
            ))
        }
    }
}

impl TruthBranchHeadSource for StructSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        Ok(patch_envelope(
            TruthCommitIdentity::from_relational_commit_id(42),
            TruthPatchIdentity::from_relational_patch_position(42),
            branch_identity.clone(),
        ))
    }
}

impl BridgeSourceAdapter for StructSource {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        BridgeSourceCapabilitySet::new(vec![
            BridgeSourceCapability::SnapshotRead,
            BridgeSourceCapability::BranchRead,
        ])
    }

    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        SnapshotReadSource::open_snapshot(self, identity, _execution)
    }
}

struct StructSnapshotReader;

impl TruthSnapshotReader for StructSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        snapshot_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        _execution: worth_runtime_bridge::facade::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        Ok(SnapshotReadPacketResult::new(
            snapshot_identity(),
            request
                .reads()
                .iter()
                .map(|read| match read.aspect_key().as_str() {
                    "identity.id" => {
                        SnapshotReadRecord::for_request(read, AspectValue::String("task-1".into()))
                    }
                    "profile" => SnapshotReadRecord::for_request(read, profile_value()),
                    other => panic!("unexpected bridge struct read `{other}`"),
                })
                .collect(),
        ))
    }
}

struct StructSink;

impl InvalidationSink for StructSink {
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

fn patch_envelope(
    commit: TruthCommitIdentity,
    patch: TruthPatchIdentity,
    branch: TruthBranchIdentity,
) -> BridgeCommittedPatchEnvelope {
    BridgeCommittedPatchEnvelope::new(
        BridgeCommittedPatchEnvelopeIdentity::new(commit, patch, snapshot_identity(), branch),
        vec![BridgeCommittedPatchItem::with_target(
            "entity-1",
            BridgeCommittedPatchTarget::entity_field_path(
                AspectLocator::new(LocatorAuthority::Authoritative, aspect_key("profile")),
                CanonicalFieldPath::single(field_key("name")),
            ),
        )],
    )
    .unwrap()
}

fn snapshot_identity() -> TruthSnapshotIdentity {
    TruthSnapshotIdentity::from_relational_snapshot(RelationalBridgeSnapshotIdentityParts::new(
        41, 1,
    ))
}

fn aspect_key(value: &str) -> AspectKey {
    AspectKey::new(value).unwrap()
}

fn field_key(value: &str) -> FieldKey {
    FieldKey::new(value).unwrap()
}
