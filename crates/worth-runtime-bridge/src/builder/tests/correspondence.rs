use std::sync::Arc;

use worth_foundational::facade::{
    AbsenceLaw, AspectBinding, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
    AspectIdentity, AspectKey, AspectLocator, AspectMask, AuthoritativeAspectChangeKind,
    CanonicalFieldPath, FieldDeclaration, FieldKey, FieldRequirement, LocatorAuthority,
    ScalarAspectType, StructAspectShape,
};
use worth_proof::TransitionOutcome;
use worth_signal::facade::{Aspect, SignalGraph, MAX_ASPECTS};

use super::support::{TestSink, TestSource};
use crate::facade::{
    BridgeAspectChangePrecision, BridgeAspectChangeWideningCause, BridgeAspectRegistration,
    BridgeAspectRegistrationId, BridgeAuthoritativeSourceProvenance, BridgeBuildErrorKind,
    BridgeCommittedPatchEnvelope, BridgeCommittedPatchEnvelopeIdentity, BridgeCommittedPatchItem,
    BridgeCommittedPatchTarget, BridgeMappingId, BridgeMappingRegistration, BridgeProducerMetadata,
    BridgeSemanticAspectChange, BridgeSemanticCorrespondenceRegistration,
    BridgeSemanticDependencyCandidate, BridgeSemanticLocality, BridgeSignalAspectTargetDeclaration,
    CoarseRoutingMode, MappingSelector, RuntimeBridge, RuntimeBridgeBuilder,
    SignalInvalidationScope, SliceWideningPolicy, SnapshotReadContract, SubscriptionSliceKind,
    TruthDeltaSurfaceKind, TruthPatchScope, TruthPatchTargetSelector,
};
use crate::relational_identity::RelationalBridgeRecordIdentityParts;
use crate::truth_identity_fixtures::{truth_branch, truth_commit, truth_patch, truth_snapshot};

#[test]
fn managed_record_mapping_family_preserves_delivered_record_locality_without_widening() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mapping = widened_mapping();
    let aspect_mapping = BridgeAspectRegistration::new(
        BridgeAspectRegistrationId::admit_bridge_owned("profile-name"),
        mapping.truth_scope().clone(),
        mapping.snapshot_read_contract().clone(),
        TruthDeltaSurfaceKind::EntityField,
        SubscriptionSliceKind::SignalField,
        SliceWideningPolicy::Disallow,
    );
    let dependency = semantic_dependencies::managed_record_dependency("query:managed");
    let runtime = runtime_with_aspect_mapping(
        mapping,
        aspect_mapping,
        vec![registration(dependency.clone(), vec![target(&graph, node)])],
    );

    let TransitionOutcome::Success(installed) =
        runtime.install_semantic_correspondence(dependency, &graph)
    else {
        panic!("managed record should use the mapping family without semantic widening");
    };
    assert_eq!(installed.admission_counters().exact_matches(), 1);
    assert_eq!(installed.admission_counters().entity_widened_matches(), 0);
}

#[test]
fn foreign_runtime_and_cloned_graph_require_distinct_recovery_paths() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let installed_runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        installed_runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("installed correspondence");
    };
    let aspect = correspondence.targets().next().unwrap().aspect();
    let original_before = graph.node_aspect_version(node).unwrap().get(aspect);

    let foreign_runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    assert!(matches!(
        foreign_runtime.deliver_installed_correspondence_envelope(
            &correspondence,
            &mut graph,
            &field_change_envelope(),
        ),
        TransitionOutcome::Stale(_)
    ));
    assert_eq!(
        graph.node_aspect_version(node).unwrap().get(aspect),
        original_before
    );

    let mut cloned_graph = graph.clone();
    let clone_before = cloned_graph.node_aspect_version(node).unwrap().get(aspect);
    assert!(matches!(
        installed_runtime.deliver_installed_correspondence_envelope(
            &correspondence,
            &mut cloned_graph,
            &field_change_envelope(),
        ),
        TransitionOutcome::RebindRequired(_)
    ));
    assert_eq!(
        cloned_graph.node_aspect_version(node).unwrap().get(aspect),
        clone_before
    );
}

#[test]
fn allocation_exhaustion_is_typed_and_leaves_existing_slots_intact() {
    let graph = SignalGraph::new();
    let mut graph = graph;
    let node = graph.node().build();
    let registrations = (0..MAX_ASPECTS)
        .map(|slot| {
            registration(
                dependency(&format!("query:{slot}")),
                vec![target(&graph, node)],
            )
        })
        .chain(std::iter::once(registration(
            dependency("query:overflow"),
            vec![target(&graph, node)],
        )))
        .collect();
    let runtime = runtime(exact_mapping(), registrations);
    for slot in 0..MAX_ASPECTS {
        let correspondence =
            runtime.install_semantic_correspondence(dependency(&format!("query:{slot}")), &graph);
        assert!(matches!(correspondence, TransitionOutcome::Success(_)));
    }
    let overflow = runtime.install_semantic_correspondence(dependency("query:overflow"), &graph);
    let TransitionOutcome::Denied(denial) = overflow else {
        panic!("slot overflow must deny");
    };
    assert_eq!(
        denial.kind(),
        crate::facade::BridgeCorrespondenceDenialKind::CapacityExhausted
    );
    assert_eq!(denial.counters().capacity_denials(), 1);
    assert_eq!(denial.counters().allocation_keys_examined(), MAX_ASPECTS);
    assert_eq!(denial.counters().targets_admitted(), 0);
    let rebuild = runtime
        .rebuild_correspondence_allocation_index()
        .expect("capacity denial must leave rebuildable installed allocations");
    assert_eq!(rebuild.authoritative_allocation_records(), MAX_ASPECTS);
    assert_eq!(rebuild.rebuilt_allocation_keys(), MAX_ASPECTS);
    assert!(rebuild.exact_index_parity());
}

#[test]
fn equal_numeric_slots_are_scoped_by_node_and_graph_basis() {
    let mut graph = SignalGraph::new();
    let first = graph.node().build();
    let second = graph.node().build();
    let exact = |node| exact_target(&graph, node, Aspect::new(0));
    let runtime = runtime(
        exact_mapping(),
        vec![
            registration(dependency("query:first"), vec![exact(first)]),
            registration(dependency("query:second"), vec![exact(second)]),
        ],
    );
    assert!(runtime
        .install_semantic_correspondence(dependency("query:first"), &graph)
        .is_success());
    assert!(runtime
        .install_semantic_correspondence(dependency("query:second"), &graph)
        .is_success());
}

#[test]
fn derived_only_signal_slot_cannot_be_promoted_into_truth_correspondence() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let derived_only = Aspect::new(7);
    assert!(graph
        .admit_installed_aspect(node, derived_only)
        .is_success());
    let derived_before = graph.node_aspect_version(node).unwrap().get(derived_only);

    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("truth correspondence should install independently");
    };
    assert!(correspondence
        .targets()
        .all(|target| target.aspect() != derived_only));
    assert!(runtime
        .deliver_installed_correspondence_envelope(
            &correspondence,
            &mut graph,
            &field_change_envelope(),
        )
        .is_success());
    assert_eq!(
        graph.node_aspect_version(node).unwrap().get(derived_only),
        derived_before
    );
}

#[test]
fn removed_entity_coarse_lane_cannot_authorize_many_to_one_correspondence() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let widened_runtime = runtime(
        widened_mapping(),
        vec![
            registration(
                dependency("query:first"),
                vec![exact_target(&graph, node, Aspect::new(0))],
            ),
            registration(
                dependency("query:second"),
                vec![exact_target(&graph, node, Aspect::new(0))],
            ),
        ],
    );
    for dependency in [dependency("query:first"), dependency("query:second")] {
        assert!(matches!(
            widened_runtime.install_semantic_correspondence(dependency, &graph),
            TransitionOutcome::Denied(denial)
                if denial.kind()
                    == crate::facade::BridgeCorrespondenceDenialKind::MappingSemanticMismatch
        ));
    }

    let exact_runtime = runtime(
        exact_mapping(),
        vec![
            registration(
                dependency("query:first"),
                vec![exact_target(&graph, node, Aspect::new(0))],
            ),
            registration(
                dependency("query:second"),
                vec![exact_target(&graph, node, Aspect::new(0))],
            ),
        ],
    );
    assert!(exact_runtime
        .install_semantic_correspondence(dependency("query:first"), &graph)
        .is_success());
    assert!(matches!(
        exact_runtime.install_semantic_correspondence(dependency("query:second"), &graph),
        TransitionOutcome::Denied(denial)
            if denial.kind()
                == crate::facade::BridgeCorrespondenceDenialKind::SharedSlotRequiresDeclaredWidening
    ));
}

#[test]
fn duplicate_fan_out_denies_atomically_without_allocation_residue() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let duplicate = exact_target(&graph, node, Aspect::new(0));
    let denial = BridgeSemanticCorrespondenceRegistration::new(
        dependency("query:one"),
        vec![duplicate.clone(), duplicate],
    )
    .expect_err("duplicate target set must deny before runtime construction");
    assert_eq!(
        denial.kind(),
        crate::facade::BridgeCorrespondenceDenialKind::DuplicateTarget
    );
    assert_eq!(denial.counters().targets_admitted(), 0);
}

#[test]
fn widened_source_cannot_flow_through_exact_correspondence() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("exact correspondence");
    };
    let aspect = correspondence.targets().next().unwrap().aspect();
    let before = graph.node_aspect_version(node).unwrap().get(aspect);
    let TransitionOutcome::Denied(denial) = runtime.deliver_installed_correspondence_envelope(
        &correspondence,
        &mut graph,
        &field_change_envelope_with_precision(BridgeAspectChangePrecision::DeclaredWidening),
    ) else {
        panic!("widened source must not satisfy exact correspondence");
    };
    assert_eq!(
        denial.kind(),
        crate::facade::BridgeCorrespondenceDenialKind::MappingSemanticMismatch
    );
    assert_eq!(denial.counters().correspondence_lookups(), 1);
    assert_eq!(denial.counters().truth_targets_admitted(), 0);
    assert_eq!(denial.counters().signal_seeds_emitted(), 0);
    assert_eq!(graph.node_aspect_version(node).unwrap().get(aspect), before);
}

#[test]
fn runtime_world_admission_is_bound_to_the_issuing_bridge_runtime() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let runtime = runtime(
        exact_mapping(),
        vec![registration(
            dependency("query:one"),
            vec![target(&graph, node)],
        )],
    );
    let TransitionOutcome::Success(correspondence) =
        runtime.install_semantic_correspondence(dependency("query:one"), &graph)
    else {
        panic!("installed correspondence");
    };

    let admitted = runtime
        .runtime_world_correspondence_port()
        .admit_installed_basis(&correspondence)
        .expect("the issuing runtime admits its installed basis");
    assert_eq!(
        admitted.source_installation_generation(),
        correspondence.basis().source_installation_generation()
    );
    let repeated = runtime
        .runtime_world_correspondence_port()
        .admit_installed_basis(&correspondence)
        .expect("the same installed correspondence reuses its admission identity");
    assert_eq!(admitted.admission_identity(), repeated.admission_identity());
    assert_eq!(admitted, repeated);

    let foreign_runtime = runtime.fork_managed_request_lane();
    assert!(matches!(
        foreign_runtime
            .runtime_world_correspondence_port()
            .admit_installed_basis(&correspondence),
        Err(crate::facade::RuntimeWorldCorrespondenceAdmissionDenial::ForeignBridgeRuntime { .. })
    ));
}

mod atomic_batch;
mod conditional_compatibility;
mod delivery_cost;
mod graph_cohesion;
mod outcome_topology;
mod partition_and_precision;
mod registration_drift;
mod runtime_world;
mod semantic_dependencies;
mod semantic_fixture;
mod semantic_invalidation;
mod slot_search;
mod source_contact;

use semantic_fixture::*;
