use std::sync::Arc;

use worth_foundational::facade::{
    AbsenceLaw, AspectBinding, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
    AspectIdentity, AspectKey, AspectMask, AuthoritativeAspectChangeKind, CanonicalFieldPath,
    FieldDeclaration, FieldKey, FieldRequirement, ProjectionMask, ScalarAspectType,
    StructAspectShape,
};
use worth_proof::TransitionOutcome;
use worth_signal::facade::{PartitionToken, SignalGraph};

use super::super::runtime_world_admission::RuntimeWorldCorrespondenceInspectionLedger;
use super::super::{
    BridgeSemanticCorrespondenceRegistration, BridgeSemanticDependencyCandidate,
    BridgeSemanticDependencyCandidateParts, BridgeSignalAspectTargetDeclaration,
};
use super::AdmittedSemanticDependencyRegistry;

fn target(graph: &mut SignalGraph) -> BridgeSignalAspectTargetDeclaration {
    let node = graph.node().build();
    let TransitionOutcome::Success(node) = graph.admit_installed_node(node) else {
        panic!("test target node is admitted by its real Signal graph");
    };
    BridgeSignalAspectTargetDeclaration::allocate(
        crate::facade::BridgeAspectRegistrationId::from_stable_name("runtime-world-index-test"),
        PartitionToken::new("runtime-world-index-test-partition"),
        node,
    )
}

fn candidate(slot: usize, generation: u64) -> BridgeSemanticDependencyCandidate {
    let field = FieldKey::new("name").expect("valid field key");
    let shape = StructAspectShape::new([FieldDeclaration::new(
        field.clone(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("valid field declaration")])
    .expect("valid aspect shape");
    let record = crate::facade::RelationalBridgeRecordIdentityParts::entity(0, 1, 1);
    BridgeSemanticDependencyCandidate::admit(BridgeSemanticDependencyCandidateParts {
        source_installation_identity: Arc::from(format!("index-installation:{slot}")),
        source_basis: Arc::from("index-basis"),
        source_runtime_authority: 700 + slot as u64,
        source_installation_generation: generation,
        source_authority_binding_identity: Arc::from(format!("index-binding:{slot}")),
        source_stage_identity: None,
        source_node_identity: Arc::from(format!("index-node:{slot}")),
        dependency_ordinal: slot,
        declared_graph_role: Arc::from("runtime-world-index-test"),
        graph_participation_identity: Arc::from("runtime-world-index-graph"),
        graph_adapter_identity: Arc::from("runtime-world-index-adapter"),
        source_record_identity: Some(record),
        observation_record_identity: Some(record),
        contract: AspectContract::struct_aspect(
            AspectKey::new("runtime-world-index").expect("valid aspect key"),
            AspectIdentity(77),
            AspectContractRevision(1),
            shape,
        ),
        projection_mask: AspectMask::<ProjectionMask>::new([CanonicalFieldPath::single(field)]),
        binding: AspectBinding::EntityField {
            field: FieldKey::new("profile").expect("valid binding field"),
        },
        locality: super::super::BridgeSemanticLocality::SourceRecord,
        relevant_changes: vec![AuthoritativeAspectChangeKind::FieldSet],
    })
    .expect("valid index candidate")
}

fn registration(
    candidate: BridgeSemanticDependencyCandidate,
    target: &BridgeSignalAspectTargetDeclaration,
) -> BridgeSemanticCorrespondenceRegistration {
    BridgeSemanticCorrespondenceRegistration::new(candidate, vec![target.clone()])
        .expect("valid index registration")
}

#[test]
fn currentness_index_records_one_direct_lookup() {
    let mut graph = SignalGraph::new();
    let target = target(&mut graph);
    for population in [1, 4096] {
        let registrations = (0..population)
            .map(|slot| registration(candidate(slot, 1), &target))
            .collect();
        let registry = AdmittedSemanticDependencyRegistry::freeze(registrations)
            .expect("population freezes into one owner registry");
        let inspection = RuntimeWorldCorrespondenceInspectionLedger::default();
        let probe = candidate(population - 1, 1);

        assert_eq!(
            registry.currentness_index().lookup(&probe, &inspection),
            Some(1)
        );
        let snapshot = inspection.snapshot();
        assert_eq!(snapshot.binding_index_lookups(), 1);
    }
}

#[test]
fn generation_maximum_missing_rebuild_and_graph_rebind_are_coherent() {
    let mut graph = SignalGraph::new();
    let target = target(&mut graph);
    let old = candidate(0, 1);
    let latest = candidate(0, 3);
    let missing = candidate(1, 1);
    let mut registry = AdmittedSemanticDependencyRegistry::freeze(vec![
        registration(old.clone(), &target),
        registration(latest.clone(), &target),
    ])
    .expect("generations freeze into one binding index");
    assert!(old.same_installation_binding_except_generation(&latest));
    assert!(!old.same_installation_binding_except_generation(&missing));
    let inspection = RuntimeWorldCorrespondenceInspectionLedger::default();

    assert_eq!(
        registry.currentness_index().lookup(&old, &inspection),
        Some(3)
    );
    assert_eq!(
        registry.currentness_index().lookup(&latest, &inspection),
        Some(3)
    );
    assert_eq!(
        registry.currentness_index().lookup(&missing, &inspection),
        None
    );
    assert_eq!(inspection.snapshot().binding_index_lookups(), 3);
    assert!(registry.rebuild_has_exact_parity());

    let lower = registry
        .admit_extension(&[registration(candidate(0, 2), &target)])
        .expect("a lower generation remains an admitted registration");
    assert_eq!(lower.commit(&mut registry), 1);
    assert_eq!(
        registry.currentness_index().lookup(&latest, &inspection),
        Some(3)
    );

    let higher = registry
        .admit_extension(&[registration(candidate(0, 4), &target)])
        .expect("a newer generation extends the registration");
    assert_eq!(higher.commit(&mut registry), 1);
    assert_eq!(
        registry.currentness_index().lookup(&latest, &inspection),
        Some(4)
    );
    assert!(registry.rebuild_has_exact_parity());

    registry.destroy_derived_indexes();
    assert!(!registry.rebuild_has_exact_parity());
    let rebuilt = registry
        .reconstruct_derived_indexes()
        .expect("destroyed derived indexes rebuild from authority");
    assert!(rebuilt.rebuild_has_exact_parity());
    assert_eq!(
        rebuilt.currentness_index().lookup(&latest, &inspection),
        Some(4)
    );
}

#[test]
fn binding_equivalence_retains_complete_contract_material() {
    let left = candidate(0, 1);
    let mut right = left.clone();
    right.contract = AspectContract::scalar(
        AspectKey::new("runtime-world-index").expect("valid aspect key"),
        AspectIdentity(77),
        AspectContractRevision(1),
        ScalarAspectType::Bool,
    );
    right.projection_mask = AspectMask::<ProjectionMask>::whole_aspect();

    assert!(!left.same_installation_binding_except_generation(&right));
}
