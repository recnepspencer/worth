//! A public registered source and one exact semantic dependency.
use std::sync::Arc;
use worth_foundational::facade::*;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::{NodeId, PartitionToken, SignalGraph};
struct BaselineSource;
impl CommittedPatchSource for BaselineSource {
    fn authoritative_source_profile(&self) -> Option<BridgeAuthoritativeSourceProfile> {
        Some(BridgeAuthoritativeSourceProfile::new(99, "relational-adapter:99").unwrap())
    }
    fn runtime_world_baseline_graph_role(&self) -> Option<&str> {
        Some("primary")
    }
    fn load_committed_patch(
        &self,
        _: RelationalCommittedPatchRequest,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, RelationalBridgeSourceError> {
        panic!("baseline metadata must not execute a source effect")
    }
}
impl SnapshotReadSource for BaselineSource {
    fn open_snapshot(
        &self,
        _: &TruthSnapshotIdentity,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        panic!("baseline metadata must not open a truth reader")
    }
}
impl InvalidationSink for BaselineSource {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
        _: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        panic!("baseline metadata must not invalidate Signal")
    }
}
fn contract() -> AspectContract {
    AspectContract::struct_aspect(
        AspectKey::new("profile").unwrap(),
        AspectIdentity(31),
        AspectContractRevision(4),
        StructAspectShape::new([FieldDeclaration::new(
            FieldKey::new("name").unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap()])
        .unwrap(),
    )
}
pub(super) fn dependency() -> BridgeSemanticDependencyCandidate {
    BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
        source_installation_identity: Arc::from("bridge-test-installation:1"),
        source_basis: Arc::from("bridge-correspondence:1"),
        source_runtime_authority: 1,
        source_installation_generation: 1,
        source_authority_binding_identity: Arc::from("bridge-test-binding:1:query:one"),
        source_stage_identity: None,
        source_node_identity: Arc::from("query:one"),
        dependency_ordinal: 0,
        declared_graph_role: Arc::from("model"),
        graph_participation_identity: Arc::from("bridge-test-graph-authority:1"),
        graph_adapter_identity: Arc::from("bridge-test-graph-adapter"),
        source_record_identity: Some(RelationalBridgeRecordIdentityParts::entity(0, 1, 1)),
        observation_record_identity: Some(RelationalBridgeRecordIdentityParts::entity(0, 1, 1)),
        contract: contract(),
        projection_mask: AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(
            FieldKey::new("name").unwrap(),
        )]),
        binding: AspectBinding::EntityField {
            field: FieldKey::new("profile").unwrap(),
        },
        locality: BridgeSemanticLocality::SourceRecord,
        relevant_changes: vec![AuthoritativeAspectChangeKind::FieldSet],
    })
    .unwrap()
}
pub(super) fn runtime(
    dependency: BridgeSemanticDependencyCandidate,
    graph: &SignalGraph,
    node: NodeId,
) -> RuntimeBridge {
    let scope = TruthPatchScope::for_entity_field(
        MappingSelector::exact("relational-record:entity:0:1:1"),
        AspectKey::new("profile").unwrap(),
        FieldKey::new("name").unwrap(),
    );
    let read = SnapshotReadContract::new(contract());
    let aspect = BridgeAspectRegistrationId::from_stable_name("profile-name");
    let target = BridgeSignalAspectTargetDeclaration::allocate(
        aspect.clone(),
        PartitionToken::new("bridge-main"),
        graph.admit_installed_node(node).unwrap(),
    );
    RuntimeBridgeBuilder::new()
        .with_committed_patch_source(BaselineSource)
        .with_snapshot_read_source(BaselineSource)
        .with_signal_sink(BaselineSource)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("profile-name"),
            scope.clone(),
            read.clone(),
            SignalInvalidationScope::from_stable_name("signal.profile.name"),
            CoarseRoutingMode::Direct,
        ))
        .register_aspect_mapping(BridgeAspectRegistration::new(
            aspect,
            scope,
            read,
            TruthDeltaSurfaceKind::EntityField,
            SubscriptionSliceKind::SignalField,
            SliceWideningPolicy::Disallow,
        ))
        .register_semantic_correspondence(
            BridgeSemanticCorrespondenceRegistration::new(dependency, vec![target]).unwrap(),
        )
        .build()
        .unwrap()
}
