use super::*;

pub(super) fn runtime(
    mapping: BridgeMappingRegistration,
    registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
) -> RuntimeBridge {
    let aspect_mapping = aspect_mapping(&mapping);
    runtime_with_aspect_mapping(mapping, aspect_mapping, registrations)
}

pub(super) fn runtime_with_source_widening(
    mapping: BridgeMappingRegistration,
    cause: BridgeAspectChangeWideningCause,
    registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
) -> RuntimeBridge {
    let aspect_mapping = aspect_mapping(&mapping).with_declared_source_widening(cause);
    runtime_with_aspect_mapping(mapping, aspect_mapping, registrations)
}

pub(super) fn runtime_with_aspect_mapping(
    mapping: BridgeMappingRegistration,
    aspect_mapping: BridgeAspectRegistration,
    registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
) -> RuntimeBridge {
    let builder = RuntimeBridgeBuilder::new()
        .with_relational_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect_mapping);
    registrations
        .into_iter()
        .fold(builder, |builder, registration| {
            builder.register_semantic_correspondence(registration)
        })
        .build()
        .expect("correspondence runtime")
}

pub(super) fn runtime_with_delivery_source(
    mapping: BridgeMappingRegistration,
    envelope: BridgeCommittedPatchEnvelope,
    registrations: Vec<BridgeSemanticCorrespondenceRegistration>,
) -> RuntimeBridge {
    let aspect_mapping = aspect_mapping(&mapping);
    let builder = RuntimeBridgeBuilder::new()
        .with_committed_patch_source(CorrespondenceEnvelopeSource(envelope))
        .with_snapshot_read_source(TestSource)
        .with_signal_sink(TestSink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect_mapping);
    registrations
        .into_iter()
        .fold(builder, |builder, registration| {
            builder.register_semantic_correspondence(registration)
        })
        .build()
        .expect("source-driven correspondence runtime")
}

struct CorrespondenceEnvelopeSource(BridgeCommittedPatchEnvelope);

impl crate::facade::CommittedPatchSource for CorrespondenceEnvelopeSource {
    fn authoritative_source_profile(
        &self,
    ) -> Option<crate::facade::BridgeAuthoritativeSourceProfile> {
        Some(
            crate::facade::BridgeAuthoritativeSourceProfile::new(99, "relational-adapter:99")
                .expect("valid correspondence source profile"),
        )
    }

    fn load_committed_patch(
        &self,
        _request: crate::facade::RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeCommittedPatchEnvelope, crate::facade::RelationalBridgeSourceError> {
        Ok(self.0.clone())
    }
}

pub(super) fn registration(
    dependency: BridgeSemanticDependencyCandidate,
    targets: Vec<BridgeSignalAspectTargetDeclaration>,
) -> BridgeSemanticCorrespondenceRegistration {
    BridgeSemanticCorrespondenceRegistration::new(dependency, targets)
        .expect("fixture correspondence registration is valid")
}

pub(super) fn exact_mapping() -> BridgeMappingRegistration {
    mapping(
        MappingSelector::exact("relational-record:entity:0:1:1"),
        TruthPatchTargetSelector::entity_field(FieldKey::new("name".to_string()).unwrap()),
    )
}

pub(super) fn sibling_field_mapping() -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("profile-status"),
        TruthPatchScope::new(
            MappingSelector::exact("relational-record:entity:0:1:1"),
            crate::mapping::AspectKeySelector::exact(aspect_key()),
            TruthPatchTargetSelector::entity_field(FieldKey::new("status").unwrap()),
        ),
        SnapshotReadContract::new(contract_with_extra_field()),
        SignalInvalidationScope::admit_bridge_owned("signal.profile.status"),
        CoarseRoutingMode::Direct,
    )
}

pub(super) fn widened_mapping() -> BridgeMappingRegistration {
    mapping(
        MappingSelector::any(),
        TruthPatchTargetSelector::entity_field(FieldKey::new("name".to_string()).unwrap()),
    )
}

pub(super) fn mapping(
    entity_selector: MappingSelector,
    target: TruthPatchTargetSelector,
) -> BridgeMappingRegistration {
    BridgeMappingRegistration::new(
        BridgeMappingId::admit_bridge_owned("profile-name"),
        TruthPatchScope::new(
            entity_selector,
            crate::mapping::AspectKeySelector::exact(aspect_key()),
            target,
        ),
        SnapshotReadContract::new(contract()),
        SignalInvalidationScope::admit_bridge_owned("signal.profile.name"),
        CoarseRoutingMode::Direct,
    )
}

pub(super) fn aspect_mapping(mapping: &BridgeMappingRegistration) -> BridgeAspectRegistration {
    let widened = matches!(
        mapping.truth_scope().entity_selector(),
        MappingSelector::Any
    );
    BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
        mapping.truth_scope().clone(),
        mapping.snapshot_read_contract().clone(),
        TruthDeltaSurfaceKind::EntityField,
        if widened {
            SubscriptionSliceKind::RegisteredCoarseWidening
        } else {
            SubscriptionSliceKind::SignalField
        },
        if widened {
            SliceWideningPolicy::RegisteredEntityCoarseWidening
        } else {
            SliceWideningPolicy::Disallow
        },
    )
}

pub(super) fn dependency(source_identity: &str) -> BridgeSemanticDependencyCandidate {
    super::semantic_dependencies::dependency(source_identity)
}

pub(super) fn target(
    graph: &worth_signal::facade::SignalGraph,
    node: worth_signal::facade::NodeId,
) -> BridgeSignalAspectTargetDeclaration {
    let worth_proof::TransitionOutcome::Success(node) = graph.admit_installed_node(node) else {
        panic!("fixture target requires an installed Signal node");
    };
    BridgeSignalAspectTargetDeclaration::allocate(
        BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
        worth_signal::facade::PartitionToken::new("bridge-main"),
        node,
    )
}

pub(super) fn exact_target(
    graph: &worth_signal::facade::SignalGraph,
    node: worth_signal::facade::NodeId,
    aspect: worth_signal::facade::Aspect,
) -> BridgeSignalAspectTargetDeclaration {
    let worth_proof::TransitionOutcome::Success(node_capability) = graph.admit_installed_node(node)
    else {
        panic!("fixture target requires an installed Signal node");
    };
    let worth_proof::TransitionOutcome::Success(aspect_capability) =
        graph.admit_installed_aspect(node, aspect)
    else {
        panic!("fixture target requires an installed Signal aspect");
    };
    BridgeSignalAspectTargetDeclaration::exact(
        BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
        worth_signal::facade::PartitionToken::new("bridge-main"),
        node_capability,
        aspect_capability,
    )
    .expect("node and aspect fixture capabilities share exact graph authority")
}

mod envelopes;
pub(super) use envelopes::{
    field_change_envelope, field_change_envelope_for_source, field_change_envelope_for_source_role,
    field_change_envelope_with_precision, field_change_envelope_with_scope,
    unidentified_whole_aspect_envelope, whole_aspect_change_envelope,
};

mod envelope_width;
pub(super) use envelope_width::field_change_envelope_with_width;

pub(super) fn contract() -> AspectContract {
    contract_at_revision(4)
}

pub(super) fn contract_at_revision(revision: u64) -> AspectContract {
    let shape = StructAspectShape::new([FieldDeclaration::new(
        FieldKey::new("name".to_string()).unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap()])
    .unwrap();
    AspectContract::struct_aspect(
        aspect_key(),
        AspectIdentity(31),
        AspectContractRevision(revision),
        shape,
    )
}

pub(super) fn contract_with_extra_field() -> AspectContract {
    let shape = StructAspectShape::new([
        FieldDeclaration::new(
            FieldKey::new("name".to_string()).unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap(),
        FieldDeclaration::new(
            FieldKey::new("status".to_string()).unwrap(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .unwrap(),
    ])
    .unwrap();
    AspectContract::struct_aspect(
        aspect_key(),
        AspectIdentity(31),
        AspectContractRevision(4),
        shape,
    )
}

fn aspect_key() -> AspectKey {
    AspectKey::new("profile").unwrap()
}
pub(super) fn field_path() -> CanonicalFieldPath {
    CanonicalFieldPath::single(FieldKey::new("name".to_string()).unwrap())
}
