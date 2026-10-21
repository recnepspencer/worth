use crate::runtime::{
    admit_graph_read_access_for_family, WorthQueryGraphReadAccessAdmissionPosture,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadRequiredCapabilityOwner,
};

use crate::runtime::tests::graph_read_access::support::graph_read_access_cost_model::{
    frontier_search_family, projection_only_family, workspace,
};

#[test]
fn inline_read_review_plan_is_same_artifact_attached_to_execution_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-six.inline-receipt",
    );
    let family = projection_only_family(&mut workspace, "phase-six-inline-receipt");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("read intent review should succeed");
    let planned = review
        .graph_read_access_plan()
        .expect("inline graph read should lower to admitted access plan");
    let planned_digest = planned.digest().to_string();
    let admission_digest = planned.admission().digest().to_string();
    let admitted = review
        .admit()
        .expect("inline graph read should admit with access plan");
    let binding_digest = admitted.execution_binding().binding_digest().to_string();
    let result = admitted
        .execute(request_execution)
        .expect("admitted inline read should execute");
    let receipt_plan = result
        .receipt()
        .graph_read_access_plan()
        .expect("receipt should expose graph read access plan");
    let consumption = result
        .receipt()
        .graph_read_access_plan_consumption()
        .expect("receipt should expose graph read access plan consumption");

    assert_eq!(receipt_plan.digest(), planned_digest);
    assert_eq!(consumption.admitted_plan_digest(), planned.digest());
    assert_eq!(consumption.admission_digest(), planned.admission().digest());
    assert_eq!(consumption.execution_binding_digest(), binding_digest);
    assert_eq!(
        consumption.execution_strategy(),
        planned.execution_strategy()
    );
    assert_eq!(consumption.execution_counters().executor_entry_count(), 1);
    assert_eq!(
        consumption.execution_counters().strategy_recompute_count(),
        0
    );
    assert_eq!(
        result
            .receipt()
            .graph_read_access_admission()
            .expect("receipt should expose graph read access admission")
            .digest(),
        admission_digest
    );
    assert_eq!(
        receipt_plan.posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
    );
}

#[test]
fn equivalent_read_fronts_produce_same_access_admission_digest() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut execute_workspace = workspace(
        request_execution,
        "graph-read-access.phase-six.execute-front",
    );
    let execute_family = projection_only_family(&mut execute_workspace, "phase-six-execute-front");
    let execute_result = execute_workspace
        .execute_read_family(request_execution, &execute_family)
        .expect("execute helper should execute through admission");

    let mut intent_workspace = workspace(
        request_execution,
        "graph-read-access.phase-six.intent-front",
    );
    let intent_family = projection_only_family(&mut intent_workspace, "phase-six-intent-front");
    let intent_result = intent_workspace
        .read_family_intent(&intent_family)
        .execute(request_execution)
        .expect("intent helper should execute through admission");

    assert_eq!(
        execute_result
            .receipt()
            .graph_read_access_admission()
            .expect("execute helper receipt should carry admission")
            .digest(),
        intent_result
            .receipt()
            .graph_read_access_admission()
            .expect("intent helper receipt should carry admission")
            .digest()
    );
}

#[test]
fn access_case_registry_covers_each_requirement_row_before_execution() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-six.case-registry",
    );
    let family = frontier_search_family(&mut workspace, "phase-six-case-registry");
    let admission = admit_graph_read_access_for_family(&family).expect("admission should derive");
    let requirement_kinds = admission
        .requirement_set()
        .rows()
        .iter()
        .map(|row| row.kind())
        .collect::<Vec<_>>();
    let registry_cases = admission.case_registry().cases();
    let case_kinds = registry_cases
        .iter()
        .map(|case| case.requirement_kind())
        .collect::<Vec<_>>();
    let registry_requirement_kinds = admission.case_registry().requirement_kinds();

    for requirement_kind in requirement_kinds {
        assert!(
            admission
                .case_registry()
                .case_for_requirement_kind(requirement_kind)
                .is_some(),
            "required graph access case must be present in registry"
        );
    }
    assert_eq!(
        registry_requirement_kinds,
        WorthQueryGraphReadAccessRequirementKind::all().to_vec()
    );
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::DirectionalAdjacency));
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::ReverseAdjacency));
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::PredicateSupport));
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::OrderingSupport));
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::TraversalWorkset));
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::VisitedSet));
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::DedupSet));
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::ProofSupport));
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::ResultBuffer));
    assert!(
        case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::MaterializationLifecycle)
    );
    assert!(case_kinds.contains(&&WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport));
}

#[test]
fn admission_explanation_exposes_requirements_and_support_without_executor_topology() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(request_execution, "graph-read-access.phase-six.explanation");
    let family = projection_only_family(&mut workspace, "phase-six-explanation");
    let review = workspace
        .read_family_intent(&family)
        .review()
        .expect("read intent review should succeed");
    let explanation = review
        .graph_read_access_explanation()
        .expect("explanation should derive");

    assert_eq!(
        explanation.selected_posture(),
        &WorthQueryGraphReadAccessAdmissionPosture::InlineIndexed
    );
    assert!(!explanation.requirement_rows().is_empty());
    assert_eq!(
        explanation.requirement_rows().len(),
        explanation.inventory_matches().len()
    );
    assert!(explanation
        .inventory_matches()
        .iter()
        .all(|inventory_match| {
            inventory_match.required_capability_owner()
                == &WorthQueryGraphReadRequiredCapabilityOwner::QueryRuntime
        }));
    assert!(explanation.denial().is_none());
}
