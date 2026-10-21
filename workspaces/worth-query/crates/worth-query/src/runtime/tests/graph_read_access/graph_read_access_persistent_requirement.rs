use crate::runtime::{
    WorthQueryGraphIndexInventoryMatchOutcome, WorthQueryGraphIndexLifecycleClass,
    WorthQueryGraphIndexLifecycleOwner, WorthQueryGraphIndexPosture,
    WorthQueryGraphIndexSupportState, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessComplexityContract, WorthQueryGraphReadAccessDenialKind,
    WorthQueryGraphReadAccessInvalidationBasis, WorthQueryGraphReadAccessRebuildBasis,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadPredicateFamily,
    WorthQueryGraphReadRequiredCapabilityOwner, WorthQueryRuntimeError,
};

use crate::runtime::tests::graph_read_access::support;

use support::graph_index_inventory::runtime_profiles::{
    profile_requiring_store_backed_graph_index, profile_without_graph_support,
};
use support::graph_read_access::persistent_requirements::{
    broad_persistent_family, persistent_predicate_family, persistent_requirement_workspace,
};

#[test]
fn persistent_store_requirement_is_declared_as_a_proof_not_a_materialization() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = persistent_requirement_workspace(
        request_execution,
        "graph-read-access.phase-ten.predicate-persistent",
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = persistent_predicate_family(&mut workspace, "phase-ten-predicate-persistent");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("persistent requirement review should derive");
    let admission = review
        .graph_read_access_admission()
        .expect("persistent requirement admission should derive");
    let declaration = admission
        .persistent_index_requirement()
        .expect("persistent requirement declaration should be visible");
    let receipt = declaration.receipt();

    assert_eq!(
        admission.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::Denied
    );
    assert_eq!(
        admission
            .denial()
            .expect("persistent requirement should deny execution")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::RequiredPersistentIndex
    );
    assert_eq!(
        declaration.required_owner(),
        &WorthQueryGraphReadRequiredCapabilityOwner::PersistentStore
    );
    assert_eq!(declaration.requirement_rows().len(), 1);
    assert_eq!(
        declaration.requirement_rows()[0].requirement_kind(),
        &WorthQueryGraphReadAccessRequirementKind::PredicateSupport
    );
    assert_eq!(
        declaration.requirement_rows()[0].required_owner(),
        &WorthQueryGraphReadRequiredCapabilityOwner::PersistentStore
    );
    assert_eq!(
        declaration.requirement_rows()[0].required_posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::PersistentIndexRequired
    );
    assert_eq!(
        declaration.requirement_rows()[0].match_outcome(),
        &WorthQueryGraphIndexInventoryMatchOutcome::ExactMatch
    );
    assert_eq!(
        declaration.requirement_rows()[0].support_state(),
        &WorthQueryGraphIndexSupportState::StoreOwnedUnavailable
    );
    assert_eq!(
        declaration.requirement_rows()[0].lifecycle_owner(),
        &WorthQueryGraphIndexLifecycleOwner::StoreOwned
    );
    assert_eq!(
        declaration.requirement_rows()[0].lifecycle_class(),
        &WorthQueryGraphIndexLifecycleClass::StoreOwnedRequired
    );
    assert_eq!(
        declaration.requirement_rows()[0].owning_milestone(),
        Some("worth-query-9.10-test-store-backed-index")
    );
    assert_eq!(
        declaration.requirement_rows()[0].rebuild_basis(),
        &WorthQueryGraphReadAccessRebuildBasis::SelectivityProof
    );
    assert_eq!(
        declaration.requirement_rows()[0].invalidation_basis(),
        &WorthQueryGraphReadAccessInvalidationBasis::AuthoritativeFieldDelta
    );
    assert_eq!(
        declaration.requirement_rows()[0].complexity_contract(),
        &WorthQueryGraphReadAccessComplexityContract::CandidatePredicateSupport
    );
    assert_eq!(declaration.counters().durable_artifact_count(), 0);
    assert_eq!(receipt.counters().blocked_allocation_count(), 1);
    assert_eq!(receipt.counters().durable_artifact_count(), 0);
    assert_eq!(receipt.declaration_digest(), declaration.digest());
    assert!(review.graph_read_access_plan().is_err());
}

#[test]
fn persistent_requirement_survives_over_budget_family_instead_of_async_magic() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = persistent_requirement_workspace(
        request_execution,
        "graph-read-access.phase-ten.broad-persistent",
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = broad_persistent_family(&mut workspace, "phase-ten-broad-persistent");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("broad persistent requirement review should derive")
        .graph_read_access_admission()
        .expect("broad persistent requirement admission should derive");

    assert_eq!(
        admission
            .denial()
            .expect("persistent requirement should deny")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::RequiredPersistentIndex
    );
    assert!(admission.persistent_index_requirement().is_some());
    assert_eq!(
        admission
            .persistent_index_requirement()
            .expect("declaration should exist")
            .requirement_rows()[0]
            .required_owner(),
        &WorthQueryGraphReadRequiredCapabilityOwner::PersistentStore
    );
}

#[test]
fn persistent_requirement_execution_denies_before_any_ephemeral_allocation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = persistent_requirement_workspace(
        request_execution,
        "graph-read-access.phase-ten.execute-denied",
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = persistent_predicate_family(&mut workspace, "phase-ten-execute-denied");
    let denial = workspace
        .read_family_intent(&family)
        .execute(request_execution)
        .expect_err("persistent requirement should stop execution");

    let WorthQueryRuntimeError::ReadCompositionDenied(read_denial) = denial else {
        panic!("expected read composition denial for persistent requirement");
    };
    let admission = read_denial
        .graph_read_access_admission()
        .expect("denial should carry admission proof");
    let counters = read_denial
        .graph_read_access_execution_counters()
        .expect("denial should carry pre-execution counters");
    let persistent_audit = read_denial
        .graph_read_persistent_artifact_audit()
        .expect("persistent denial should carry durable artifact audit");

    assert_eq!(counters.executor_entry_count(), 0);
    assert_eq!(counters.ephemeral_index_allocation_count(), 0);
    assert_eq!(persistent_audit.durable_artifact_create_attempt_count(), 0);
    assert_eq!(persistent_audit.durable_artifact_open_attempt_count(), 0);
    assert_eq!(persistent_audit.durable_artifact_write_attempt_count(), 0);
    assert_eq!(persistent_audit.declaration_only_stop_count(), 1);
    assert!(admission.persistent_index_requirement().is_some());
    assert_eq!(
        admission
            .persistent_index_requirement()
            .expect("persistent declaration should exist")
            .receipt()
            .counters()
            .durable_artifact_count(),
        0
    );
}

#[test]
fn persistent_requirement_rows_match_store_backed_inventory_rows_exactly() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = persistent_requirement_workspace(
        request_execution,
        "graph-read-access.phase-ten.inventory-exact",
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        ),
    );
    let family = persistent_predicate_family(&mut workspace, "phase-ten-inventory-exact");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("review should derive")
        .graph_read_access_admission()
        .expect("admission should derive");
    let declaration = admission
        .persistent_index_requirement()
        .expect("persistent declaration should exist");
    let inventory_match = admission
        .graph_index_inventory_match_report()
        .matches()
        .iter()
        .find(|row| {
            row.support_posture()
                == &WorthQueryGraphIndexPosture::RequiresStoreBackedPersistentIndex
        })
        .expect("inventory should contain the store-backed support row");
    let declaration_row = declaration
        .requirement_rows()
        .first()
        .expect("declaration should carry the exact row");

    assert_eq!(
        declaration.inventory_match_report_digest(),
        admission.graph_index_inventory_match_report().digest()
    );
    assert_eq!(
        declaration_row.requirement_row_digest(),
        inventory_match.requirement_row_digest()
    );
    assert_eq!(
        declaration_row.support_row_digest(),
        inventory_match.support_row_digest()
    );
}

#[test]
fn persistent_family_with_missing_inventory_row_fails_closed_without_declaration() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = persistent_requirement_workspace(
        request_execution,
        "graph-read-access.phase-ten.missing-persistent-row",
        profile_without_graph_support(WorthQueryGraphReadAccessRequirementKind::PredicateSupport),
    );
    let family = persistent_predicate_family(&mut workspace, "phase-ten-missing-persistent-row");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("review should derive")
        .graph_read_access_admission()
        .expect("admission should derive");
    let contract = admission.graph_read_family_index_contract();
    let missing_match = admission
        .graph_index_inventory_match_report()
        .matches()
        .iter()
        .find(|row| {
            row.requirement_kind() == &WorthQueryGraphReadAccessRequirementKind::PredicateSupport
        })
        .expect("predicate support requirement should be represented");

    assert_eq!(
        admission
            .denial()
            .expect("missing support should deny")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::UnsupportedGraphIndexSupport
    );
    assert!(admission.persistent_index_requirement().is_none());
    assert_eq!(contract.persistent_requirement_digest(), None);
    assert_eq!(
        missing_match.outcome(),
        &WorthQueryGraphIndexInventoryMatchOutcome::MissingSupportRow
    );
}

#[test]
fn near_miss_persistent_support_is_not_treated_as_exact_requirement_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = persistent_requirement_workspace(
        request_execution,
        "graph-read-access.phase-ten.near-miss-predicate",
        profile_requiring_store_backed_graph_index(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
        )
        .with_graph_index_supported_predicate_family(
            WorthQueryGraphReadAccessRequirementKind::PredicateSupport,
            WorthQueryGraphReadPredicateFamily::Equality,
        ),
    );
    let family = persistent_predicate_family(&mut workspace, "phase-ten-near-miss-predicate");
    let admission = workspace
        .read_family_intent(&family)
        .review()
        .expect("review should derive")
        .graph_read_access_admission()
        .expect("admission should derive");
    let mismatch = admission
        .graph_index_inventory_match_report()
        .matches()
        .iter()
        .find(|row| {
            row.requirement_kind() == &WorthQueryGraphReadAccessRequirementKind::PredicateSupport
        })
        .expect("predicate support requirement should be represented");

    assert_eq!(
        mismatch.outcome(),
        &WorthQueryGraphIndexInventoryMatchOutcome::PredicateMismatch
    );
    assert_eq!(
        admission
            .denial()
            .expect("near-miss support should deny")
            .kind(),
        &WorthQueryGraphReadAccessDenialKind::UnsupportedGraphIndexSupport
    );
    assert!(admission.persistent_index_requirement().is_none());
}
