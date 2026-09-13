use std::sync::Arc;

use worth_foundational::facade::{
    AspectBinding, AspectKey, AspectMask, AuthoritativeAspectChangeKind, FieldKey, ProjectionMask,
    ScalarAspectType,
};
use worth_relational::facade::{
    bridge::RuntimeBridgeRelationalSource, runtime::RelationalRuntimeApi,
};
use worth_runtime_bridge::facade::{
    AdmittedRuntimeWorldCorrespondenceBasis, BridgeAspectRegistration, BridgeAspectRegistrationId,
    BridgeDeliveryReceipt, BridgeMappingId, BridgeMappingRegistration,
    BridgeSemanticCorrespondenceRegistration, BridgeSemanticDependencyCandidate,
    BridgeSemanticDependencyCandidateParts, BridgeSemanticLocality,
    BridgeSignalAspectTargetDeclaration, CoarseRoutingMode, InvalidationSink, MappingSelector,
    RuntimeBridgeBuilder, SignalBridgeSinkError, SignalInvalidationScope, SliceWideningPolicy,
    SnapshotReadContract, SubscriptionSliceKind, TruthDeltaSurfaceKind, TruthPatchScope,
    TruthPatchTargetSelector,
};
use worth_signal::facade::{SignalGraph, SignalRuntime};

use super::{admit_current, CompositeBasisAdmissionDenial};
use crate::basis::compare_exact;
use crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract;

#[path = "admission_tests/commit.rs"]
mod commit;

#[derive(Clone)]
struct AdmissionSink;

impl InvalidationSink for AdmissionSink {
    fn deliver_invalidation(
        &self,
        delivery: worth_runtime_bridge::facade::BridgeSignalInvalidationDelivery,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub(super) struct ComponentFixture {
    pub(super) relational: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    pub(super) relational_port: worth_relational::facade::branch::RelationalBranchBasisPort,
    pub(super) signal: worth_signal::facade::branch::AdmittedSignalBranchBasis,
    pub(super) signal_port: worth_signal::facade::branch::SignalBranchBasisPort<(), (), ()>,
    pub(super) correspondence: AdmittedRuntimeWorldCorrespondenceBasis,
    pub(super) correspondence_port: worth_runtime_bridge::facade::RuntimeWorldCorrespondencePort,
    pub(super) _relational_runtime: Arc<worth_relational::facade::runtime::RelationalRuntime>,
    pub(super) _signal_runtime: SignalRuntime<(), (), (), (), ()>,
}

pub(super) fn component_fixture() -> ComponentFixture {
    let relational_runtime = Arc::new(RelationalRuntimeApi::builder().build());
    let relational_port = relational_runtime.owner_component_services().basis_port();
    let relational_identity = relational_runtime.main_branch_identity();
    let (_, relational) = relational_runtime
        .observe_branch(&relational_identity)
        .expect("real Relational owner admits its main basis");

    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let worth_proof::TransitionOutcome::Success(node) = graph.admit_installed_node(node) else {
        panic!("real Signal owner admits its installed node");
    };
    let aspect_key = AspectKey::new("profile").expect("valid aspect key");
    let field_key = FieldKey::new("name").expect("valid field key");
    let snapshot_contract =
        SnapshotReadContract::scalar(aspect_key.clone(), ScalarAspectType::String);
    let dependency_contract = snapshot_contract.aspect_contract().clone();
    let mapping = BridgeMappingRegistration::new(
        BridgeMappingId::from_stable_name("profile-name"),
        TruthPatchScope::for_target(
            MappingSelector::exact("relational-record:entity:0:1:1"),
            aspect_key.clone(),
            TruthPatchTargetSelector::authoritative_aspect(),
        ),
        snapshot_contract.clone(),
        SignalInvalidationScope::from_stable_name("signal.profile.name"),
        CoarseRoutingMode::Direct,
    );
    let aspect_registration = BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::from_stable_name("profile-name"),
        mapping.truth_scope().clone(),
        snapshot_contract,
        TruthDeltaSurfaceKind::AuthoritativeAspect,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    );
    let target = BridgeSignalAspectTargetDeclaration::allocate(
        BridgeAspectRegistrationId::from_stable_name("profile-name"),
        worth_signal::facade::PartitionToken::new("bridge-main"),
        node,
    );
    let source = RuntimeBridgeRelationalSource::for_graph_role(
        Arc::clone(&relational_runtime),
        "runtime-world-test",
    )
    .expect("real Relational owner provides the Bridge source");
    let source_profile = source.authoritative_source_profile();
    let dependency =
        BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
            source_installation_identity: Arc::from("relational-installation:1"),
            source_basis: Arc::from("relational-main"),
            source_runtime_authority: source_profile.runtime_instance_id(),
            source_installation_generation: 1,
            source_authority_binding_identity: Arc::from("relational-owner-binding:1"),
            source_stage_identity: None,
            source_node_identity: Arc::from("runtime-world-test"),
            dependency_ordinal: 0,
            declared_graph_role: Arc::from("runtime-world-test"),
            graph_participation_identity: Arc::from("runtime-world-graph"),
            graph_adapter_identity: Arc::from("relational-bridge-adapter"),
            source_record_identity: Some(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            ),
            observation_record_identity: Some(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
            ),
            contract: dependency_contract,
            projection_mask: AspectMask::<ProjectionMask>::whole_aspect(),
            binding: AspectBinding::EntityField { field: field_key },
            locality: BridgeSemanticLocality::SourceRecord,
            relevant_changes: vec![AuthoritativeAspectChangeKind::FieldSet],
        })
        .expect("real Bridge owner admits its semantic dependency");
    let registration =
        BridgeSemanticCorrespondenceRegistration::new(dependency.clone(), vec![target])
            .expect("real Bridge registration is coherent");
    let bridge = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(AdmissionSink)
        .register_mapping(mapping)
        .register_aspect_mapping(aspect_registration)
        .register_semantic_correspondence(registration)
        .build()
        .expect("real Bridge owner builds the installed correspondence");
    let installed = {
        let mut binding = bridge
            .bind_signal_graph(&mut graph)
            .expect("Bridge binds the real Signal graph");
        match binding.install_semantic_correspondence(dependency) {
            worth_proof::TransitionOutcome::Success(installed) => installed,
            other => {
                panic!("real Bridge owner admits its registered correspondence: {other:?}")
            }
        }
    };
    let correspondence_port = bridge.runtime_world_correspondence_port();
    let correspondence = correspondence_port
        .admit_installed_basis(&installed)
        .expect("Bridge Runtime World port admits its installed basis");

    let mut signal_runtime = SignalRuntime::builder(graph).with_kernel_defaults().build();
    let signal_port = signal_runtime
        .owner_component_services()
        .expect("real Signal owner issues its basis service")
        .basis_port();
    let signal = signal_runtime
        .observe_signal_branch_basis(signal_runtime.current_branch())
        .expect("real Signal owner admits its current basis");

    ComponentFixture {
        relational,
        relational_port,
        signal,
        signal_port,
        correspondence,
        correspondence_port,
        _relational_runtime: relational_runtime,
        _signal_runtime: signal_runtime,
    }
}

#[test]
fn foreign_owner_equal_descriptor_cannot_substitute_during_composite_admission() {
    let fixture = component_fixture();
    let mut foreign_signal_runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    let foreign_signal_port = foreign_signal_runtime
        .owner_component_services()
        .expect("foreign real Signal owner issues its basis service")
        .basis_port();
    let transported_signal = fixture.signal.clone();
    assert_eq!(
        fixture.signal.descriptor(),
        transported_signal.descriptor(),
        "the transported artifact has an exactly equal serializable descriptor"
    );

    let identities =
        RuntimeWorldOwnerConstructionContract::new().expect("World owner construction");
    let owner = identities.owner_identity();
    let foreign_identities =
        RuntimeWorldOwnerConstructionContract::new().expect("foreign World owner construction");
    let foreign_owner = foreign_identities.owner_identity();
    assert_ne!(owner, foreign_owner);

    let denial = admit_current(
        identities.issuer(),
        &fixture.relational_port,
        &foreign_signal_port,
        &fixture.correspondence_port,
        fixture.relational.clone(),
        transported_signal,
        fixture.correspondence.clone(),
    )
    .expect_err("a foreign Signal owner must reject the equal-looking basis");
    assert!(matches!(
        denial,
        CompositeBasisAdmissionDenial::Signal(
            worth_signal::facade::branch::SignalBranchBasisReadmissionDenial::OwnerMismatch { .. }
        )
    ));

    let admitted = admit_current(
        identities.issuer(),
        &fixture.relational_port,
        &fixture.signal_port,
        &fixture.correspondence_port,
        fixture.relational.clone(),
        fixture.signal.clone(),
        fixture.correspondence.clone(),
    )
    .expect("the Signal owner port admits the live basis");
    let foreign_world_admission = admit_current(
        foreign_identities.issuer(),
        &fixture.relational_port,
        &fixture.signal_port,
        &fixture.correspondence_port,
        fixture.relational.clone(),
        fixture.signal.clone(),
        fixture.correspondence.clone(),
    )
    .expect("the second World owner admits through the real Signal owner port");
    let repeated_admission = admit_current(
        identities.issuer(),
        &fixture.relational_port,
        &fixture.signal_port,
        &fixture.correspondence_port,
        fixture.relational.clone(),
        fixture.signal.clone(),
        fixture.correspondence.clone(),
    )
    .expect("a second owner admission remains a valid World operation");

    assert_eq!(admitted.owner_identity(), owner);
    assert_eq!(admitted, admitted.clone());
    assert_eq!(
        admitted.signal_basis().descriptor(),
        foreign_world_admission.signal_basis().descriptor(),
        "foreign World admissions carry equal Signal descriptors"
    );
    assert_eq!(
        admitted.relational_basis(),
        foreign_world_admission.relational_basis()
    );
    assert_eq!(
        admitted.correspondence_basis(),
        foreign_world_admission.correspondence_basis()
    );
    assert_ne!(
        admitted, foreign_world_admission,
        "World owner identity, not a Signal descriptor, defines composite equivalence"
    );
    assert_ne!(admitted.identity(), foreign_world_admission.identity());
    assert!(compare_exact(&admitted, &foreign_world_admission).is_err());
    assert_eq!(admitted.identity(), repeated_admission.identity());
    assert_eq!(admitted, repeated_admission);
    assert!(compare_exact(&admitted, &admitted.clone()).is_ok());
    assert!(compare_exact(&admitted, &repeated_admission).is_ok());

    let relational_pin = crate::retention::ExactComponentPinRequest::relational(
        &admitted,
        crate::retention::ComponentBasisDependencyClass::AdmittedObservation,
    );
    let signal_pin = crate::retention::ExactComponentPinRequest::signal(
        &admitted,
        crate::retention::ComponentBasisDependencyClass::AdmittedObservation,
    );
    assert_eq!(relational_pin.owner(), owner);
    assert_eq!(signal_pin.owner(), owner);
    assert_eq!(
        relational_pin.key(),
        crate::retention::ExactComponentBasisKey::Relational(
            admitted.relational_basis().admission_identity().clone(),
        )
    );
    assert_eq!(
        signal_pin.key(),
        crate::retention::ExactComponentBasisKey::Signal(
            admitted.signal_basis().admission_identity().clone(),
        )
    );

    let current_branch = fixture._signal_runtime.current_branch();
    let distinct_signal = fixture
        ._signal_runtime
        .observe_signal_branch_basis(current_branch)
        .expect("the real Signal owner admits a second current basis");
    assert_eq!(fixture.signal.descriptor(), distinct_signal.descriptor());
    assert_eq!(
        fixture.signal.admission_identity(),
        distinct_signal.admission_identity(),
        "the exact live Signal basis is canonical within its owner"
    );
    let distinct_signal_admission = admit_current(
        identities.issuer(),
        &fixture.relational_port,
        &fixture.signal_port,
        &fixture.correspondence_port,
        fixture.relational.clone(),
        distinct_signal,
        fixture.correspondence.clone(),
    )
    .expect("the real Signal port admits the distinct current basis");
    assert_eq!(admitted.identity(), distinct_signal_admission.identity());
    assert_eq!(admitted, distinct_signal_admission);
}

#[test]
fn foreign_relational_owner_cannot_substitute_an_equal_looking_basis() {
    let fixture = component_fixture();
    let foreign_runtime = RelationalRuntimeApi::builder().build();
    let foreign_identity = foreign_runtime.main_branch_identity();
    let (_, foreign_relational) = foreign_runtime
        .observe_branch(&foreign_identity)
        .expect("the foreign real Relational owner admits its main basis");
    let identities =
        RuntimeWorldOwnerConstructionContract::new().expect("World owner construction");

    let denial = admit_current(
        identities.issuer(),
        &fixture.relational_port,
        &fixture.signal_port,
        &fixture.correspondence_port,
        foreign_relational,
        fixture.signal,
        fixture.correspondence,
    )
    .expect_err("a foreign Relational owner must be rejected before composition");
    assert!(matches!(
        denial,
        CompositeBasisAdmissionDenial::Relational(
            worth_relational::facade::branch::RelationalBranchBasisDenial::ForeignRuntime { .. }
        )
    ));
}
