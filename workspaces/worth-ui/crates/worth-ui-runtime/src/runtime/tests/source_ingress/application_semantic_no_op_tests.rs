use crate::facade::WorthUiApplicationReplacementOutcome;
use crate::runtime::tests::active_application_session_test_support::{
    admit_candidate_catalog, admit_first_candidate_catalog_row_with_viewport_width,
    component_candidate_submission, source_backed_component_session,
};
use crate::runtime::tests::appearance_component_session_test_support::{
    source_backed_two_node_appearance_session, two_node_appearance_candidate_submission,
    validation_background_role, APPEARANCE_NODE_A, APPEARANCE_NODE_B,
};
use crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission;
use crate::runtime::{
    WorthUiFrameBoundary, WorthUiNoOpProvenancePosture, WorthUiNoOpQueryPosture,
    WorthUiSourceProvider, WorthUiWatchedCandidateSubmission, WorthUiWatcherEvent,
};

const NO_OP_STORM_COUNT: usize = 2_000;

#[test]
fn equivalent_candidate_is_an_authority_preserving_semantic_no_op() {
    let mut session = source_backed_component_session();
    prime_allocation_truth(&mut session);
    let predecessor = session.inspect_runtime();

    let receipt = replace_with_equivalent_candidate(&mut session, "semantic-no-op-once");

    assert_ne!(receipt.candidate_generation(), receipt.active_generation());
    assert_eq!(
        receipt.active_generation(),
        predecessor.generation_identity()
    );
    assert_eq!(receipt.equivalence().changed_region_count(), 0);
    assert_eq!(
        receipt.work().exact_region_comparison_count(),
        receipt.equivalence().exact_region_comparison_count(),
        "the no-op work receipt reports only comparisons performed by the regional proof"
    );
    assert_eq!(receipt.work().candidate_region_construction_count(), 0);
    assert_eq!(receipt.work().activation_publication_count(), 0);
    assert_eq!(receipt.work().scheduler_transition_count(), 0);
    assert_eq!(
        receipt.provenance_posture(),
        WorthUiNoOpProvenancePosture::PriorAdmittedMappingPreserved
    );
    assert_eq!(
        receipt.query_posture(),
        WorthUiNoOpQueryPosture::ActiveBindingPreserved
    );
    assert_eq!(session.inspect_runtime(), predecessor);
}

#[test]
#[ignore = "long replacement storms run in hostile certification"]
fn long_replacement_storm_equivalent_candidates_never_publish_or_change_active_truth() {
    let mut session = source_backed_component_session();
    prime_allocation_truth(&mut session);
    let predecessor = session.inspect_runtime();

    for step in 0..NO_OP_STORM_COUNT {
        let receipt = replace_with_equivalent_candidate(&mut session, &format!("no-op-{step}"));
        assert_eq!(
            receipt.active_generation(),
            predecessor.generation_identity()
        );
        assert_eq!(receipt.work().activation_publication_count(), 0);
    }

    assert_eq!(session.inspect_runtime(), predecessor);
}

#[test]
fn watcher_order_and_formatting_drift_preserve_the_admitted_mapping_without_publication() {
    let mut session = source_backed_component_session();
    prime_allocation_truth(&mut session);
    let admitted = session.inspect_runtime();

    let reordered = incidental_submission(&session, "reordered", true, false);
    let outcome = replace(&mut session, reordered);
    let reload_cost = match &outcome {
        WorthUiApplicationReplacementOutcome::SemanticNoOp(receipt) => receipt
            .reload_cost()
            .expect("semantic no-op carries the work that proved equivalence"),
        WorthUiApplicationReplacementOutcome::Activated(_) => {
            panic!("provider insertion order must not publish")
        }
    };
    assert_eq!(
        reload_cost.context().unwrap().active_plan_digest(),
        reload_cost.context().unwrap().candidate_plan_digest()
    );
    let no_op = outcome
        .semantic_no_op()
        .expect("provider insertion order is not executable meaning");
    assert_eq!(
        no_op.provenance_posture(),
        WorthUiNoOpProvenancePosture::PriorAdmittedMappingPreserved
    );
    assert_eq!(session.inspect_runtime(), admitted);

    let reformatted = incidental_submission(&session, "reformatted", false, true);
    let outcome = replace(&mut session, reformatted);
    assert!(outcome.semantic_no_op().is_some());
    assert_eq!(session.inspect_runtime(), admitted);
}

#[test]
fn equivalent_appearance_attachment_remains_a_semantic_no_op() {
    let role = validation_background_role("theme.appearance_consumer");
    let mut session = source_backed_two_node_appearance_session(&role);
    prime_two_node_appearance_allocation(&mut session, &role);
    let predecessor = session.inspect_runtime();
    let submission = two_node_appearance_candidate_submission(
        &session,
        "two-node-appearance-current",
        &role,
        APPEARANCE_NODE_A,
    );

    let outcome = replace(&mut session, submission);

    assert!(outcome.semantic_no_op().is_some());
    assert_eq!(session.inspect_runtime(), predecessor);
}

#[test]
fn moving_an_equal_demand_attachment_activates_the_successor_graph() {
    let role = validation_background_role("theme.appearance_consumer");
    let mut session = source_backed_two_node_appearance_session(&role);
    prime_two_node_appearance_allocation(&mut session, &role);
    let submission = two_node_appearance_candidate_submission(
        &session,
        "two-node-appearance-current",
        &role,
        APPEARANCE_NODE_B,
    );

    let outcome = replace(&mut session, submission);

    assert!(outcome.activation().is_some());
    let graph = session.graph();
    let attachment_by_node = graph
        .snapshot()
        .nodes()
        .iter()
        .filter_map(|node| {
            let authored = node.declaration_identity().authored_semantic_name();
            (authored == APPEARANCE_NODE_A || authored == APPEARANCE_NODE_B).then(|| {
                (
                    authored.to_owned(),
                    node.appearance_role_attachment()
                        .map(|attachment| attachment.role().as_str().to_owned()),
                )
            })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(attachment_by_node.get(APPEARANCE_NODE_A), Some(&None));
    assert_eq!(
        attachment_by_node.get(APPEARANCE_NODE_B),
        Some(&Some(role.role().as_str().to_owned()))
    );
}

#[test]
fn executable_change_cannot_be_attached_as_a_provenance_refresh() {
    let mut session = source_backed_component_session();
    prime_allocation_truth(&mut session);
    let prior = session.inspect_runtime();

    let changed = component_candidate_submission(
        &session,
        "changed-executable-component",
        "workspace.component.active_session_candidate",
    );
    let outcome = replace(&mut session, changed);
    assert!(outcome.semantic_no_op().is_none());
    assert!(outcome.activation().is_some());
    assert_ne!(session.inspect_runtime(), prior);
}

#[test]
fn allocation_value_change_activates_even_when_the_executable_plan_is_identical() {
    let mut session = source_backed_component_session();
    let predecessor_rows = prime_allocation_truth(&mut session);
    let prior = session.inspect_runtime();
    let submission = component_candidate_submission(
        &session,
        "changed-viewport-allocation",
        "workspace.component.active_session_current",
    );
    let prepared = session
        .prepare_replacement(submission)
        .expect("allocation-only candidate prepares");
    let mut prepared = prepared;
    let catalog = admit_first_candidate_catalog_row_with_viewport_width(&mut prepared, 144.0);
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("allocation-only candidate lowers");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("allocation-only candidate stages");
    let boundary = WorthUiFrameBoundary::safe_to_activate(
        session.inspect_runtime().frame_epoch(),
        session.host_session_identity(),
    );
    let outcome = session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .expect("allocation-only candidate reaches publication");
    let activation = outcome
        .activation()
        .expect("changed viewport meaning requires allocation publication");
    let catalog = activation.allocation_catalog_successor();

    assert_eq!(
        activation.plan_decision().kind(),
        crate::runtime::WorthUiExecutablePlanDecisionKind::ExactSemanticNoOp
    );
    assert_eq!(catalog.predecessor_rows(), predecessor_rows);
    assert_eq!(catalog.successor_rows(), predecessor_rows);
    assert_eq!(catalog.carried_rows(), predecessor_rows - 1);
    assert_eq!(catalog.transitions().len(), 1);
    assert!(catalog.transitions().iter().all(|transition| {
        transition.disposition()
            == crate::runtime::exports::UiAllocationCatalogRowDisposition::Replanned
    }));
    assert_eq!(catalog.counters().submitted_row_visits(), 1);
    assert_eq!(catalog.counters().carried_row_visits(), 0);
    assert_eq!(catalog.counters().derived_index_row_visits(), 2);
    assert_eq!(catalog.counters().derived_index_membership_mutations(), 2);
    assert_eq!(catalog.counters().derived_index_owner_mutations(), 4);
    assert_ne!(session.inspect_runtime(), prior);
}

#[test]
fn removing_every_active_allocation_row_publishes_an_empty_successor() {
    let mut session = source_backed_component_session();
    let prime = component_candidate_submission(
        &session,
        "prime-removal-only-allocation",
        "workspace.component.active_session_current",
    );
    let initial = replace(&mut session, prime)
        .into_activation()
        .expect("initial allocation truth publishes");
    let predecessor = initial.allocation_catalog_successor();
    let removed_roots = predecessor
        .transitions()
        .iter()
        .map(|transition| transition.root())
        .collect::<Vec<_>>();
    assert!(!removed_roots.is_empty());

    let source_name = "remove-all-allocation-roots";
    let submission = lower_file_submission(
        WorthUiSourceProvider::in_memory(source_name).with_file(
            "app/main.wui",
            "token theme.removal_only = \"theme.removal_only\";",
        ),
        [WorthUiWatcherEvent::provider_revision(source_name)],
        session.capabilities(),
    );
    let prepared = session
        .prepare_replacement(submission)
        .expect("non-allocating candidate prepares");
    let delta = prepared
        .admit_candidate_allocation_catalog_delta(Vec::new(), removed_roots)
        .expect("candidate graph admits exact active-root removals");
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("removal-only candidate lowers");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("removal-only candidate stages");
    let boundary = WorthUiFrameBoundary::safe_to_activate(
        session.inspect_runtime().frame_epoch(),
        session.host_session_identity(),
    );
    let activation = session
        .activate_prepared_replacement(pending, delta, boundary, None)
        .expect("removal-only successor publishes")
        .into_activation()
        .expect("removing allocation truth changes executable meaning");
    let successor = activation.allocation_catalog_successor();

    assert_eq!(successor.predecessor_rows(), predecessor.successor_rows());
    assert_eq!(successor.successor_rows(), 0);
    assert_eq!(successor.carried_rows(), 0);
    assert_eq!(successor.counters().submitted_row_visits(), 0);
    assert_eq!(successor.counters().carried_row_visits(), 0);
    assert!(successor.transitions().iter().all(|transition| {
        transition.disposition()
            == crate::runtime::exports::UiAllocationCatalogRowDisposition::Removed
    }));
}

fn replace_with_equivalent_candidate(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    source_name: &str,
) -> crate::runtime::WorthUiSemanticNoOpReceipt {
    let submission = component_candidate_submission(
        session,
        source_name,
        "workspace.component.active_session_current",
    );
    match replace(session, submission) {
        WorthUiApplicationReplacementOutcome::SemanticNoOp(receipt) => receipt.receipt().clone(),
        WorthUiApplicationReplacementOutcome::Activated(receipt) => {
            panic!(
                "equivalent executable meaning must not publish: plan={:?}, allocation={:?}",
                receipt.plan_decision(),
                receipt.plan_swap().committed_allocation().counters()
            )
        }
    }
}

fn prime_allocation_truth(session: &mut crate::facade::WorthUiActiveApplicationSession) -> usize {
    let submission = component_candidate_submission(
        session,
        "prime-active-allocation",
        "workspace.component.active_session_current",
    );
    let outcome = replace(session, submission);
    let activation = outcome
        .into_activation()
        .expect("initial mounted allocation truth requires publication");
    assert_eq!(
        activation.plan_decision().kind(),
        crate::runtime::WorthUiExecutablePlanDecisionKind::ExactSemanticNoOp,
        "plan equality cannot erase the first committed allocation transition"
    );
    activation.allocation_catalog_successor().successor_rows()
}

fn prime_two_node_appearance_allocation(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
) {
    let submission = two_node_appearance_candidate_submission(
        session,
        "two-node-appearance-current",
        role,
        APPEARANCE_NODE_A,
    );
    let activation = replace(session, submission)
        .into_activation()
        .expect("initial appearance allocation truth requires publication");
    assert_eq!(
        activation.plan_decision().kind(),
        crate::runtime::WorthUiExecutablePlanDecisionKind::ExactSemanticNoOp
    );
}

fn replace(
    session: &mut crate::facade::WorthUiActiveApplicationSession,
    submission: WorthUiWatchedCandidateSubmission,
) -> WorthUiApplicationReplacementOutcome {
    let prepared = session
        .prepare_replacement(submission)
        .expect("equivalent candidate prepares");
    let mut prepared = prepared;
    let catalog = admit_candidate_catalog(&mut prepared);
    let lowered = session
        .lower_prepared_replacement(*prepared)
        .expect("equivalent candidate lowers completely");
    let pending = session
        .stage_prepared_replacement(lowered)
        .expect("equivalent candidate stages completely");
    let boundary = WorthUiFrameBoundary::safe_to_activate(
        session.inspect_runtime().frame_epoch(),
        session.host_session_identity(),
    );
    session
        .activate_prepared_replacement(pending, catalog, boundary, None)
        .expect("equivalent candidate reaches the plan decision")
}

fn incidental_submission(
    session: &crate::facade::WorthUiActiveApplicationSession,
    source_name: &str,
    reverse_events: bool,
    reformatted: bool,
) -> WorthUiWatchedCandidateSubmission {
    let current = if reformatted {
        "\n component workspace.component.active_session_current {\n region workspace.region.primary { sizing workspace.sizing.mosaic_support; }\n }\n"
    } else {
        "component workspace.component.active_session_current { region workspace.region.primary { sizing workspace.sizing.mosaic_support; } }"
    };
    let provider = WorthUiSourceProvider::in_memory(source_name).with_file("app/main.wui", current);
    let events = if reverse_events {
        [
            WorthUiWatcherEvent::write_completed("app/main.wui"),
            WorthUiWatcherEvent::modified("app/main.wui"),
        ]
    } else {
        [
            WorthUiWatcherEvent::modified("app/main.wui"),
            WorthUiWatcherEvent::write_completed("app/main.wui"),
        ]
    };
    lower_file_submission(provider, events, session.capabilities())
}
