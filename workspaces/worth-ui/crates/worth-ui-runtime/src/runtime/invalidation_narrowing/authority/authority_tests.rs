use crate::runtime::tests::activation_staging_test_support::activation_staging_inputs;

#[path = "viewport_tests.rs"]
mod viewport_tests;

fn authority_context(
    basis: crate::evidence::UiMeasurementBasis,
    neighborhood: crate::evidence::UiAllocationNeighborhood,
) -> super::authority::UiAllocationInvalidationAdmissionContext {
    let planning_basis =
        crate::runtime::WorthUiAllocationPlanningBasis::new(basis, neighborhood, None);
    super::authority::UiAllocationInvalidationAdmissionContext::from_planning_basis(&planning_basis)
}

pub(super) fn admitted_runtime_authority(
    _label: &str,
) -> (
    crate::runtime::WorthUiRuntime,
    crate::graph::UiGraphNodeIdentity,
    crate::runtime::UiAllocationCandidate,
) {
    let (runtime, root, _, candidate) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_production_catalog_activation();
    (runtime, root, candidate)
}

#[test]
fn fresh_runtime_denies_narrowing_when_no_existing_target_authority_exists() {
    let inputs = activation_staging_inputs();
    let (mut runtime, _) = inputs.into_runtime_and_pending();
    let root = crate::graph::UiGraphNodeIdentity::new(1);

    let completion = runtime.execute_framework_turn(|turn| {
        turn.interaction(|source| {
            source
                .admit_and_submit(
                    root,
                    crate::runtime::WorthUiTransientInteractionState::TextInput,
                )
                .expect("interaction source admits independently of target narrowing");
        });
    });

    let crate::runtime::WorthUiFrameworkTurnCompletion::AllocationInvalidationNarrowingDenied {
        rejection,
    } = completion
    else {
        panic!("fresh runtime must deny rather than mint target authority");
    };
    assert_eq!(
        rejection.denial(),
        crate::runtime::UiAllocationInvalidationNarrowingDenial::GraphTargetNotAdmitted {
            ordinal: 0,
        }
    );
}

#[test]
fn admitted_candidate_does_not_publish_active_narrowing_authority() {
    let inputs = activation_staging_inputs();
    let (mut runtime, pending) = inputs.into_runtime_and_pending();
    let (basis, snapshot, selected) =
        crate::runtime::tests::allocation_planning_test_support::admitted_planning_admission(
            "candidate-is-not-active",
            "operator:stack",
        );
    let root = basis.graph_node_identity();
    let input = runtime
        .admit_planning_lane_input(&pending, &snapshot, basis, &selected)
        .expect("planning input admits");
    assert!(runtime.plan_allocation(input).is_admitted());

    let completion = runtime.execute_framework_turn(|turn| {
        turn.interaction(|source| {
            source
                .admit_and_submit(
                    root,
                    crate::runtime::WorthUiTransientInteractionState::TextInput,
                )
                .expect("interaction admits independently");
        });
    });
    assert!(matches!(
        completion,
        crate::runtime::WorthUiFrameworkTurnCompletion::AllocationInvalidationNarrowingDenied { .. }
    ));
}

#[test]
fn ordinary_frame_narrows_only_through_existing_graph_authority() {
    let (mut runtime, root, _) = admitted_runtime_authority("existing-phase6-authority");

    let completion = runtime.execute_framework_turn(|turn| {
        turn.interaction(|source| {
            source
                .admit_and_submit(
                    root,
                    crate::runtime::WorthUiTransientInteractionState::TextInput,
                )
                .expect("admitted interaction enters the ordinary frame");
        });
    });

    let narrowed = completion
        .narrowed_plan()
        .expect("existing graph authority permits narrowing");
    let counters = completion
        .planning_counters()
        .expect("ordinary plan publishes structural counters");
    assert_eq!(counters.admitted_ingress_width(), 1);
    assert_eq!(counters.invalidation_breadth(), 1);
    assert_eq!(counters.selected_neighborhood_breadth(), 1);
    assert_eq!(counters.policy_family_count(), 1);
    assert_eq!(counters.policy_classification_count(), 1);
    assert_eq!(narrowed.counters().graph_target_lookups(), 1);
    assert_eq!(narrowed.counters().emitted_targets(), 1);
    let crate::runtime::UiAllocationInvalidationTarget::Graph(target) =
        narrowed.narrowed_invalidations()[0].target()
    else {
        panic!("interaction must retain graph target authority");
    };
    assert_eq!(target.graph_node_identities(), &[root]);
    assert_eq!(narrowed.counters().authority_probes(), 1);
}

#[test]
fn repeated_scope_admission_replaces_stale_authority() {
    use crate::runtime::tests::allocation_planning_test_support::{
        admitted_measurement_basis_with_font_seed, planning_graph_authority,
    };

    let mut index = super::authority::UiAllocationInvalidationAuthority::default();
    let (snapshot, selected) = planning_graph_authority("phase6-capacity", "operator:stack");
    for generation_index in 0..=64_u64 {
        let basis = admitted_measurement_basis_with_font_seed("phase6-capacity", generation_index);
        let neighborhood = selected
            .admit_allocation_neighborhood(&snapshot, &basis)
            .expect("graph authority admits");
        install_graph_authority_fixture(&mut index, authority_context(basis, neighborhood));
    }
}

#[test]
fn successor_projection_retires_targets_absent_from_a_different_scope() {
    use crate::runtime::tests::allocation_planning_test_support::admitted_planning_admission;

    let mut projection = super::authority::UiAllocationInvalidationAuthority::default();
    let (first_basis, first_graph, first_selected) =
        admitted_planning_admission("phase6-first-scope", "operator:stack");
    let first_root = first_basis.graph_node_identity();
    let first_neighborhood = first_selected
        .admit_allocation_neighborhood(&first_graph, &first_basis)
        .expect("first scope admits");
    install_graph_authority_fixture(
        &mut projection,
        authority_context(first_basis, first_neighborhood),
    );
    assert!(projection
        .graph_target(first_root)
        .unwrap()
        .target
        .is_some());

    let (second_basis, second_graph, second_selected) =
        admitted_planning_admission("phase6-second-scope", "operator:grid");
    let second_neighborhood = second_selected
        .admit_allocation_neighborhood(&second_graph, &second_basis)
        .expect("second scope admits");
    install_graph_authority_fixture(
        &mut projection,
        authority_context(second_basis, second_neighborhood),
    );

    assert!(projection
        .graph_target(first_root)
        .unwrap()
        .target
        .is_none());
}

#[test]
fn identical_existing_authority_replays_to_identical_narrowing() {
    fn run(label: &str) -> String {
        let (mut runtime, root, _) = admitted_runtime_authority(label);
        let completion = runtime.execute_framework_turn(|turn| {
            turn.interaction(|source| {
                source
                    .admit_and_submit(
                        root,
                        crate::runtime::WorthUiTransientInteractionState::TextInput,
                    )
                    .expect("interaction admits");
            });
        });
        format!(
            "{:?}",
            completion.narrowed_plan().expect("narrowing succeeds")
        )
    }

    assert_eq!(run("phase6-replay"), run("phase6-replay"));
}

#[test]
fn owner_native_host_mapping_counts_source_index_and_membership_probes() {
    use crate::runtime::tests::allocation_planning_test_support::admitted_planning_admission;

    let (basis, graph, selected) =
        admitted_planning_admission("phase6-host-mapping", "operator:stack");
    let host_result = basis
        .evidence_inputs()
        .iter()
        .find_map(crate::evidence::MeasurementEvidenceInput::as_host_measurement_result)
        .expect("fixture carries admitted host evidence");
    let witness = host_result.authority_witness();
    let root = basis.graph_node_identity();
    let neighborhood = selected
        .admit_allocation_neighborhood(&graph, &basis)
        .expect("graph owner admits neighborhood");
    let mut authority = super::authority::UiAllocationInvalidationAuthority::default();
    install_graph_authority_fixture(&mut authority, authority_context(basis, neighborhood));

    let lookup = authority
        .host_target(witness)
        .expect("matching owner evidence admits");
    assert_eq!(lookup.probes, 1);
    assert_eq!(lookup.target_count(), 1);
    assert_eq!(
        lookup.materialize_target().unwrap().graph_node_identities(),
        &[root]
    );
}

#[test]
fn existing_graph_node_outside_active_neighborhood_cannot_be_minted_as_target() {
    use crate::runtime::tests::allocation_planning_test_support::admitted_planning_admission;

    let (basis, graph, selected) =
        admitted_planning_admission("phase6-non-member", "operator:stack");
    let neighborhood = selected
        .admit_allocation_neighborhood(&graph, &basis)
        .expect("graph owner admits neighborhood");
    let mut authority = super::authority::UiAllocationInvalidationAuthority::default();
    install_graph_authority_fixture(&mut authority, authority_context(basis, neighborhood));

    let lookup = authority
        .graph_target(crate::graph::UiGraphNodeIdentity::new(u64::MAX))
        .expect("absence is a typed lookup result");
    assert_eq!(lookup.probes, 1);
    assert!(lookup.target.is_none());
}

#[test]
fn query_mapping_consumes_admitted_projection_receipt_and_neighborhood_proof() {
    use crate::runtime::tests::allocation_planning_test_support::{
        admitted_query_measurement_basis, planning_graph_authority,
    };

    let basis = admitted_query_measurement_basis("phase6-query-mapping");
    assert!(basis.is_admitted());
    let receipt = basis
        .evidence_inputs()
        .iter()
        .find_map(crate::evidence::MeasurementEvidenceInput::as_settled_query_fact)
        .expect("fixture carries projection-consumption evidence");
    let source_key =
        crate::evidence::measurement::basis::UiQueryAllocationSourceKey::SettledSnapshot(
            receipt.key().clone(),
        );
    let root = basis.graph_node_identity();
    let (graph, selected) = planning_graph_authority("phase6-query-mapping", "operator:stack");
    let neighborhood = selected
        .admit_allocation_neighborhood(&graph, &basis)
        .expect("graph owner admits Query-backed neighborhood");
    let mut authority = super::authority::UiAllocationInvalidationAuthority::default();
    install_graph_authority_fixture(&mut authority, authority_context(basis, neighborhood));

    let lookup = authority
        .settled_query_target(&source_key)
        .expect("settled Query mapping admits");
    assert_eq!(lookup.probes, 3);
    assert_eq!(lookup.target.unwrap().graph_node_identities(), &[root]);
}

#[test]
fn ordinary_turn_selects_and_atomically_commits_the_phase7_transaction() {
    let (mut runtime, root, widened_root, _) = crate::runtime::tests::production_catalog_activation_test_support::runtime_with_production_catalog_activation();
    let completion = runtime.execute_framework_turn(|turn| {
        turn.interaction(|source| {
            source
                .admit_and_submit(
                    root,
                    crate::runtime::WorthUiTransientInteractionState::TextInput,
                )
                .expect("interaction admits");
            source
                .admit_and_submit(
                    widened_root,
                    crate::runtime::WorthUiTransientInteractionState::TextInput,
                )
                .expect("widened-neighborhood interaction admits");
        });
    });
    let selection = completion
        .replan_selection()
        .unwrap_or_else(|| panic!("ordinary turn selects locality: {completion:?}"));
    assert_eq!(selection.ordered_neighborhoods().len(), 2);
    assert_eq!(selection.counters().replacement_consequence_reads(), 2);
    assert_eq!(selection.counters().graph_index_probes(), 2);
    assert_eq!(
        selection.overlap_disposition(),
        crate::graph::UiReplanOverlapDisposition::PairwiseDisjoint
    );
    assert_eq!(
        selection.root_posture(),
        crate::graph::UiReplanRootPosture::CountedRootWiden {
            reason: crate::graph::UiReplanWidenReason::SharedAncestorRequirement,
        }
    );
    let replay_selection = selection.clone();
    let crate::runtime::UiAllocationReplanTransactionOutcome::Committed(committed) = completion
        .replan_transaction()
        .expect("ordinary turn publishes its transaction")
    else {
        panic!("ordinary selected set must commit atomically");
    };
    assert_eq!(committed.receipts().len(), 2);
    assert_eq!(committed.counters().selected_neighborhoods(), 2);
    assert_eq!(committed.counters().committed_receipts(), 2);
    assert_eq!(committed.transaction().ordered_neighborhoods().len(), 2);
    assert_eq!(committed.evidence().committed_receipt_count(), 2);
    assert_eq!(
        committed.evidence().overlap_disposition(),
        crate::graph::UiReplanOverlapDisposition::PairwiseDisjoint
    );
    let lowering = committed.receipts()[0]
        .lowering_input()
        .expect("current committed receipt lowers");
    assert_eq!(lowering.receipt(), &committed.receipts()[0]);
    assert_eq!(lowering.report(), committed.receipts()[0].report());
    assert_eq!(lowering.transaction(), committed.transaction());
    assert_eq!(
        lowering.frame_epoch(),
        committed.transaction().frame_epoch()
    );
    drop(completion);
    assert!(matches!(
        runtime.replay_admitted_transaction_for_test(&replay_selection),
        crate::runtime::UiAllocationReplanTransactionOutcome::Replayed(_)
    ));

    let (successor_basis, successor_graph, successor_selected) =
        crate::runtime::tests::allocation_planning_test_support::admitted_planning_admission(
            "phase7-successor-authority",
            "operator:grid",
        );
    let successor_neighborhood = successor_selected
        .admit_allocation_neighborhood(&successor_graph, &successor_basis)
        .expect("successor graph authority admits");
    install_graph_authority_fixture(
        &mut runtime.allocation_invalidation_index.borrow_mut(),
        authority_context(successor_basis, successor_neighborhood),
    );
    assert!(matches!(
        runtime.replay_admitted_transaction_for_test(&replay_selection),
        crate::runtime::UiAllocationReplanTransactionOutcome::Denied(
            crate::runtime::UiAllocationReplanTransactionCommitDenial::AdmittedGenerationSetChanged
        )
    ));
}
fn install_graph_authority_fixture(
    authority: &mut super::UiAllocationInvalidationAuthority,
    context: super::UiAllocationInvalidationAdmissionContext,
) {
    let transition = authority.graph_replan.seal_activation_transition(vec![(
        crate::evidence::UiAllocationNeighborhoodScope::from_neighborhood(&context.neighborhood),
        context.neighborhood.identity().clone(),
        context.neighborhood.graph_snapshot_authority_digest(),
        context.planning_identity_digest(),
        context.graph_replan_admission(),
    )]);
    assert!(authority
        .graph_replan
        .apply_activation_transition(&transition));
    authority.install_fixture_context(context.commit());
    authority.rebuild_indexes();
}
