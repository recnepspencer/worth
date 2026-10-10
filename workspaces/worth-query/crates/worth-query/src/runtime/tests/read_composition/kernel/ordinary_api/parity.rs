use super::super::super::support::*;
use super::composed_fixtures::{
    local_named_direct_read, local_named_scoped_read, local_named_template_read,
};
use super::fixtures::{
    anchored_manager_graph_read, current_bounded_manager_relationship_context,
    current_manager_relationship_context, local_identity_collection_read, local_identity_read,
    local_manager_relationship_read,
};
use crate::ordinary::read::{
    admit_read_context_declaration, current, declare, WorthQueryReadOutcome,
};
use crate::runtime::{
    WorthQueryReadBuilder, WorthQueryReadFamily, WorthQueryReadGraphFamily,
    WorthQueryReadOperatorFamily,
};

macro_rules! assert_ordinary_internal_parity {
    ($execution:expr, $author:path, $context:expr, $workspace_name:literal) => {{
        let declaration = declare($author).expect("ordinary declaration should build");
        let mut workspace = read_runtime($execution)
            .workspace($workspace_name)
            .expect("ordinary workspace should open");
        let completion = declaration
            .using($context)
            .run($execution, &mut workspace)
            .into_result()
            .expect("ordinary read should execute");
        let ordinary_context_receipt = completion.context_receipt().clone();
        let ordinary_journey_counters = *completion.journey_counters();
        let ordinary = completion.into_result();

        let oracle_intent = $author(WorthQueryReadBuilder::declaration())
            .expect("internal oracle declaration should build");
        let admitted_context = admit_read_context_declaration(&oracle_intent, $context.into())
            .expect("internal oracle context should admit");
        let (oracle_authority, oracle_planning_authority, oracle_context_receipt) =
            admitted_context.into_parts();
        let oracle_read_graph = oracle_intent
            .plan(oracle_planning_authority)
            .expect("internal oracle should plan");
        let oracle_plan_digest = oracle_read_graph
            .execution_plan()
            .query()
            .plan_digest()
            .evidence_identity();
        let oracle_family =
            WorthQueryReadFamily::new_kernel_only("declared_read", oracle_read_graph);
        let oracle = workspace
            .read_family_intent_in_graph_read_authority(&oracle_family, &oracle_authority)
            .execute($execution)
            .expect("internal phase-chain oracle should execute");

        assert_eq!(
            ordinary.receipt().execution_plan_evidence_identity(),
            &oracle_plan_digest
        );
        assert_eq!(ordinary_context_receipt, oracle_context_receipt);
        assert_eq!(
            ordinary_journey_counters.context_admission_attempt_count(),
            1
        );
        assert_eq!(ordinary_journey_counters.planning_attempt_count(), 1);
        assert_eq!(ordinary_journey_counters.planning_completed_count(), 1);
        assert_eq!(
            ordinary_journey_counters.lower_runtime_execution_attempt_count(),
            1
        );
        assert_eq!(
            ordinary_journey_counters.lower_runtime_execution_completed_count(),
            1
        );
        assert_eq!(ordinary.receipt(), oracle.receipt());
        assert_eq!(ordinary, oracle);
        ordinary
    }};
}

#[test]
fn ordinary_read_matches_internal_phase_chain_result_and_receipt_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_identity_read).expect("ordinary declaration should build");
    let declaration_identity = declaration.identity().as_str().to_string();
    let ordinary = assert_ordinary_internal_parity!(
        request_execution,
        local_identity_read,
        current(),
        "ordinary-read-detail-parity"
    );

    assert_ne!(declaration_identity, ordinary.receipt().read_graph_digest());
    assert!(!ordinary
        .receipt()
        .execution_plan_evidence_identity()
        .as_str()
        .is_empty());
    assert!(ordinary.receipt().graph_read_access_plan().is_some());
}

#[test]
fn ordinary_collection_preserves_family_and_matches_internal_phase_chain() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let result = assert_ordinary_internal_parity!(
        request_execution,
        local_identity_collection_read,
        current(),
        "ordinary-read-collection-parity"
    );

    assert_eq!(
        result.receipt().graph_family(),
        &WorthQueryReadGraphFamily::Collection
    );
    assert_eq!(result.rows().len(), 1);
    assert_eq!(
        result.receipt().breadth().planned_traversal_clause_count(),
        0
    );
}

#[test]
fn ordinary_graph_read_preserves_traversal_and_matches_internal_phase_chain() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let result = assert_ordinary_internal_parity!(
        request_execution,
        anchored_manager_graph_read,
        current_bounded_manager_relationship_context(),
        "ordinary-read-graph-parity"
    );

    assert_eq!(
        result.receipt().graph_family(),
        &WorthQueryReadGraphFamily::Collection
    );
    assert!(result
        .receipt()
        .operator_families()
        .contains(&WorthQueryReadOperatorFamily::Traversal));
    assert_eq!(
        result.receipt().breadth().planned_traversal_clause_count(),
        1
    );
    assert_eq!(
        result.receipt().breadth().planned_traversal_depth_limit(),
        2
    );
}

#[test]
fn ordinary_composed_read_matches_internal_phase_chain_without_scalarizing_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let result = assert_ordinary_internal_parity!(
        request_execution,
        local_manager_relationship_read,
        current_manager_relationship_context(),
        "ordinary-read-composed-parity"
    );

    assert_eq!(
        result.receipt().graph_family(),
        &WorthQueryReadGraphFamily::Detail
    );
    assert!(!result.receipt().built_in_operator_coverage().is_empty());
    assert_eq!(
        result.receipt().breadth().planned_traversal_clause_count(),
        1
    );
}

#[test]
fn scope_and_template_composition_execute_through_the_same_admitted_chain() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let direct = assert_ordinary_internal_parity!(
        request_execution,
        local_named_direct_read,
        current(),
        "ordinary-read-declarative-composition-parity"
    );
    let scoped = assert_ordinary_internal_parity!(
        request_execution,
        local_named_scoped_read,
        current(),
        "ordinary-read-declarative-composition-parity"
    );
    let templated = assert_ordinary_internal_parity!(
        request_execution,
        local_named_template_read,
        current(),
        "ordinary-read-declarative-composition-parity"
    );

    assert_eq!(direct, scoped);
    assert_eq!(direct, templated);
    assert_eq!(
        direct.receipt().execution_plan_evidence_identity(),
        scoped.receipt().execution_plan_evidence_identity()
    );
    assert_eq!(
        direct.receipt().execution_plan_evidence_identity(),
        templated.receipt().execution_plan_evidence_identity()
    );
}

#[test]
fn ordinary_read_exposes_success_without_phase_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = declare(local_identity_read).expect("ordinary declaration should build");
    let mut workspace = read_runtime(request_execution)
        .workspace("ordinary-read-outcome")
        .expect("ordinary workspace should open");

    match declaration
        .using(current())
        .run(request_execution, &mut workspace)
    {
        WorthQueryReadOutcome::Completed(completion) => {
            assert!(!completion.result().receipt().query_digest().is_empty());
            assert_eq!(
                completion
                    .context_receipt()
                    .counters()
                    .graph_authority_admitted_count(),
                1
            );
        }
        WorthQueryReadOutcome::Stopped(stop) => {
            panic!("ordinary read unexpectedly stopped: {:?}", stop.source())
        }
    }
}
