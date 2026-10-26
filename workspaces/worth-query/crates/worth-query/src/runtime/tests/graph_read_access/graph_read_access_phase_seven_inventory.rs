use crate::runtime::{
    worth_query_graph_index_inventory, WorthQueryGraphIndexInventoryMatchOutcome,
    WorthQueryGraphIndexPosture, WorthQueryGraphIndexSupportState,
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessDenialKind,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadRequiredCapabilityOwner,
};

use crate::runtime::tests::graph_read_access::support;

use support::graph_index_inventory::assertions::{
    missing_match_for_requirement, requirement_row_digest_for_kind, support_match_for_kind,
};
use support::graph_index_inventory::read_families::{
    predicate_collection_family, traversal_collection_family,
};
use support::graph_index_inventory::runtime_profiles::{
    default_graph_support_workspace, profile_requiring_graph_access_capability_registration,
    profile_requiring_store_backed_graph_index, profile_with_graph_support_temporarily_unavailable,
    profile_without_graph_support, workspace_with_graph_support,
};

#[test]
fn graph_index_inventory_digest_is_stable_for_the_same_runtime_assembly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let first = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-seven.inventory.first",
    );
    let second = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-seven.inventory.second",
    );
    let first_report = first.graph_index_inventory();
    let second_report = second.graph_index_inventory();
    let default_report = worth_query_graph_index_inventory();

    assert_eq!(first_report.digest(), second_report.digest());
    assert_eq!(first_report.digest(), default_report.digest());
    assert_eq!(
        first_report.rows().len(),
        WorthQueryGraphReadAccessRequirementKind::all().len() + 2
    );
    for requirement_kind in WorthQueryGraphReadAccessRequirementKind::all() {
        assert!(
            first_report
                .row_for_requirement_kind(requirement_kind)
                .is_some(),
            "missing graph index inventory row for {requirement_kind:?}"
        );
    }
}

#[test]
fn graph_index_inventory_digest_changes_when_runtime_assembly_changes() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let default_workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-seven.inventory.default",
    );
    let omitted_workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-seven.inventory.omitted",
        profile_without_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );

    assert_ne!(
        default_workspace.graph_index_inventory().digest(),
        omitted_workspace.graph_index_inventory().digest()
    );
}

#[test]
fn verified_graph_index_rows_carry_certified_support_state_and_required_bases() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let inventory = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-seven.certified",
    )
    .graph_index_inventory();

    for row in inventory
        .rows()
        .iter()
        .filter(|row| row.posture() == &WorthQueryGraphIndexPosture::Verified)
    {
        assert!(row.support_state().certifies_verified_support());
        assert!(!row.rebuild_basis().as_str().is_empty());
        assert!(!row.invalidation_basis().as_str().is_empty());
        assert!(!row.complexity_contract().as_str().is_empty());
    }
}

#[test]
fn admission_consumes_runtime_bound_graph_index_inventory_match_report_before_planning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = default_graph_support_workspace(
        request_execution,
        "graph-read-access.phase-seven.admission",
    );
    let family = predicate_collection_family(&mut workspace, "phase-seven-admission");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("review should be inspectable");
    let admission = review
        .graph_read_access_admission()
        .expect("admission should be derivable");
    let support = review
        .graph_index_support()
        .expect("support report should be derivable from review");
    let plan = review
        .graph_read_access_plan()
        .expect("inline read should produce admitted plan");

    assert_eq!(
        support.digest(),
        admission.graph_index_inventory_match_report().digest()
    );
    assert_eq!(support.digest(), plan.graph_index_support().digest());
    assert_eq!(
        support.inventory_digest(),
        admission.graph_index_inventory().digest()
    );
    assert_eq!(
        support.requirement_set_digest().render_hex(),
        admission.requirement_set().digest().render_support_hex()
    );
    assert_eq!(
        support.counters().matched_requirement_count(),
        admission.requirement_set().rows().len()
    );
    assert_eq!(support.counters().generic_missing_index_count(), 0);
    assert_eq!(
        admission.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
    );
}

#[test]
fn missing_runtime_support_localizes_to_exact_requirement_row_and_denies_admission() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-seven.missing-directional",
        profile_without_graph_support(
            WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
        ),
    );
    let family = traversal_collection_family(&mut workspace, "phase-seven-missing-directional");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("review should derive through public read intent");
    let admission = review
        .graph_read_access_admission()
        .expect("admission should derive denied posture");
    let missing_requirement_digest = requirement_row_digest_for_kind(
        admission.requirement_set(),
        WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency,
    );
    let missing_match = missing_match_for_requirement(
        admission.graph_index_inventory_match_report().matches(),
        &missing_requirement_digest,
    );

    assert_eq!(
        missing_match.requirement_kind(),
        &WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency
    );
    assert_eq!(
        missing_match.resolved_admission_posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::Denied
    );
    assert_eq!(
        admission
            .denial()
            .expect("admission should be denied")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::UnsupportedGraphIndexSupport
    );
}

#[test]
fn temporary_runtime_support_unavailability_maps_to_async_materialization_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-seven.unavailable-predicate",
        profile_with_graph_support_temporarily_unavailable(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = predicate_collection_family(&mut workspace, "phase-seven-unavailable-predicate");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("review should derive")
        .graph_read_access_admission()
        .expect("admission should derive");
    let predicate_match = support_match_for_kind(
        admission.graph_index_inventory_match_report().matches(),
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        WorthQueryGraphIndexPosture::TemporarilyUnavailable,
    );

    assert_eq!(
        predicate_match.resolved_admission_posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::AsyncMaterializationRequired
    );
    assert_eq!(
        admission
            .denial()
            .expect("admission should be denied")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::RequiredAsyncMaterialization
    );
}

#[test]
fn capability_registration_required_support_maps_to_registration_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-seven.registration-predicate",
        profile_requiring_graph_access_capability_registration(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = predicate_collection_family(&mut workspace, "phase-seven-registration-predicate");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("review should derive")
        .graph_read_access_admission()
        .expect("admission should derive");
    let predicate_match = support_match_for_kind(
        admission.graph_index_inventory_match_report().matches(),
        WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        WorthQueryGraphIndexPosture::RequiresAccessCapabilityRegistration,
    );

    assert_eq!(
        predicate_match.required_capability_owner(),
        &WorthQueryGraphReadRequiredCapabilityOwner::DomainRegistration
    );
    assert_eq!(
        admission
            .denial()
            .expect("admission should be denied")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::RequiredAccessCapabilityRegistration
    );
}

#[test]
fn store_backed_required_support_is_visible_without_pretending_runtime_support_exists() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace_with_graph_support(
        request_execution,
        "graph-read-access.phase-seven.store-backed-predicate",
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let inventory = workspace.graph_index_inventory();
    let predicate_row = inventory
        .row_for_requirement_kind(&WorthQueryGraphReadAccessRequirementKind::PredicateSupport)
        .expect("predicate support row should exist");
    let family = predicate_collection_family(&mut workspace, "phase-seven-store-backed-predicate");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("review should derive")
        .graph_read_access_admission()
        .expect("admission should derive");

    assert_eq!(
        predicate_row.posture(),
        &WorthQueryGraphIndexPosture::RequiresStoreBackedPersistentIndex
    );
    assert_eq!(
        predicate_row.support_state(),
        &WorthQueryGraphIndexSupportState::StoreOwnedUnavailable
    );
    assert_eq!(
        admission.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::Denied
    );
    assert_eq!(
        admission
            .denial()
            .expect("admission should be denied")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::RequiredPersistentIndex
    );
    assert!(admission
        .graph_index_inventory_match_report()
        .matches()
        .iter()
        .any(
            |row| row.outcome() == &WorthQueryGraphIndexInventoryMatchOutcome::ExactMatch
                && row.support_posture()
                    == &WorthQueryGraphIndexPosture::RequiresStoreBackedPersistentIndex
        ));
}
