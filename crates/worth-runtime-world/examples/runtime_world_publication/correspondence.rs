use super::relational::{self, CargoRecords};
use worth_foundational::facade::*;
use worth_proof::TransitionOutcome;
use worth_relational::facade::bridge::RuntimeBridgeRelationalSource;
use worth_runtime_bridge::facade::*;
use worth_signal::facade::{Aspect, NodeId, PartitionToken, SignalGraph};

/// The example uses bound correspondence and sealed owner services. Accidental
/// entry into the distinct generic delivery path must fail, never acknowledge.
struct UnboundDelivery;
impl InvalidationSink for UnboundDelivery {
    fn deliver_invalidation(
        &self,
        _: BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Err(SignalBridgeSinkError::new(
            "example requires exact graph-bound delivery",
        ))
    }
}
pub fn install(
    records: &CargoRecords,
    graph: &mut SignalGraph,
    source_node: NodeId,
) -> (RuntimeBridge, BridgeInstalledSemanticCorrespondence) {
    let source =
        RuntimeBridgeRelationalSource::for_graph_role(records.runtime.clone(), "cargo-routing")
            .unwrap();
    let profile = source.authoritative_source_profile();
    let cargo = records.grain;
    let record = RelationalBridgeRecordIdentityParts::entity(
        cargo.partition_value(),
        cargo.local_slot_value(),
        cargo.generation_value(),
    );
    let candidate =
        BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
            source_installation_identity: "publication-example-v1".into(),
            source_basis: "cargo-schema-v1".into(),
            source_runtime_authority: profile.runtime_instance_id(),
            source_installation_generation: 1,
            source_authority_binding_identity: profile.adapter_identity().into(),
            source_stage_identity: None,
            source_node_identity: "cargo-tonnes".into(),
            dependency_ordinal: 0,
            declared_graph_role: "cargo-routing".into(),
            graph_participation_identity: "publication-example-routing".into(),
            graph_adapter_identity: profile.adapter_identity().into(),
            source_record_identity: Some(record),
            observation_record_identity: Some(record),
            contract: relational::contract(),
            projection_mask: AspectMask::whole_aspect(),
            binding: AspectBinding::EntityField {
                field: relational::field(),
            },
            locality: BridgeSemanticLocality::SourceRecord,
            relevant_changes: vec![AuthoritativeAspectChangeKind::FieldSet],
        })
        .expect("example declaration: semantic dependency");
    let scope = TruthPatchScope::for_target(
        MappingSelector::exact(record.terminal_projection_for_reporting()),
        relational::key(),
        TruthPatchTargetSelector::authoritative_aspect(),
    );
    let read = SnapshotReadContract::new(relational::contract());
    let aspect = BridgeAspectRegistrationId::from_stable_name("cargo-tonnes");
    let TransitionOutcome::Success(node) = graph.admit_installed_node(source_node) else {
        panic!("example declaration: source node")
    };
    let TransitionOutcome::Success(slot) =
        graph.admit_installed_aspect(source_node, Aspect::new(0))
    else {
        panic!("example declaration: source aspect")
    };
    let target = BridgeSignalAspectTargetDeclaration::exact(
        aspect.clone(),
        PartitionToken::new("cargo"),
        node,
        slot,
    )
    .unwrap();
    let bridge = RuntimeBridge::builder()
        .with_relational_source(source)
        .with_signal_sink(UnboundDelivery)
        .register_mapping(BridgeMappingRegistration::new(
            BridgeMappingId::from_stable_name("cargo"),
            scope.clone(),
            read.clone(),
            SignalInvalidationScope::from_stable_name("route"),
            CoarseRoutingMode::Direct,
        ))
        .register_aspect_mapping(BridgeAspectRegistration::new(
            aspect,
            scope,
            read,
            TruthDeltaSurfaceKind::AuthoritativeAspect,
            SubscriptionSliceKind::SignalField,
            SliceWideningPolicy::Disallow,
        ))
        .register_semantic_correspondence(
            BridgeSemanticCorrespondenceRegistration::new(candidate.clone(), vec![target]).unwrap(),
        )
        .build()
        .expect("example declaration: real Bridge");
    let installed = match bridge
        .bind_signal_graph(graph)
        .expect("example declaration: same graph binding")
        .install_semantic_correspondence(candidate)
    {
        TransitionOutcome::Success(installed) => installed,
        other => panic!("example declaration: correspondence {other:?}"),
    };
    (bridge, installed)
}
