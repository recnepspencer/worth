use crate::runtime::WorthQueryGraphReadAccessRequirementKind;
use crate::runtime::{
    derive_graph_read_cost_evidence, estimate_graph_read_access_cost,
    estimate_graph_read_access_cost_with_planning_observation,
    explain_graph_read_access_requirements_for_family, WorthQueryGraphReadAccessCostEstimate,
    WorthQueryGraphReadBudget, WorthQueryGraphReadBudgetClassKind,
    WorthQueryGraphReadComplexityContractKind, WorthQueryGraphReadCostEstimateStatusKind,
    WorthQueryGraphReadInlineEphemeralAllowanceKind, WorthQueryGraphReadResultPressure,
};

use crate::runtime::tests::graph_read_access::support::graph_read_access_cost_model::{
    assert_exact_bucket_contribution, bucket_sum, dense_traversal_family, frontier_search_family,
    intermediate_pressure_family, reordered_simple_traversal_family, simple_traversal_family,
    workspace,
};

#[test]
fn dense_boolean_traversal_exceeds_inline_ephemeral_budget_conservatively() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-five.dense-budget",
    );
    let family = dense_traversal_family(&mut workspace, "phase-five-dense-budget");
    let requirements = explain_graph_read_access_requirements_for_family(&family)
        .expect("requirements should derive");

    let evidence = derive_graph_read_cost_evidence(&requirements);
    let estimate = estimate_graph_read_access_cost(&requirements, evidence);
    let budget = WorthQueryGraphReadBudget::inline_ephemeral_default();
    let budget_check = budget.check_supported_cost(&estimate);

    assert_eq!(
        estimate.status().kind(),
        &WorthQueryGraphReadCostEstimateStatusKind::UnknownConservative
    );
    assert_eq!(
        estimate.complexity_contract().kind(),
        &WorthQueryGraphReadComplexityContractKind::BroadTraversalCandidate
    );
    assert_eq!(
        budget_check.class().kind(),
        &WorthQueryGraphReadBudgetClassKind::ExceedsInlineEphemeralBudget
    );
    assert_eq!(
        budget_check.inline_ephemeral_allowance().kind(),
        &WorthQueryGraphReadInlineEphemeralAllowanceKind::Rejected
    );
    assert_eq!(
        budget_check.cost_estimate_digest(),
        estimate.digest().as_str()
    );
}

#[test]
fn equivalent_access_requirements_produce_identical_cost_estimate_digests() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-five.equivalence",
    );
    let first = simple_traversal_family(&mut workspace, "phase-five-equivalence-a");
    let second = reordered_simple_traversal_family(&mut workspace, "phase-five-equivalence-b");
    let first_requirements = explain_graph_read_access_requirements_for_family(&first)
        .expect("first requirements should derive");
    let second_requirements = explain_graph_read_access_requirements_for_family(&second)
        .expect("second requirements should derive");

    let first_evidence = derive_graph_read_cost_evidence(&first_requirements);
    let second_evidence = derive_graph_read_cost_evidence(&second_requirements);
    let first_estimate = estimate_graph_read_access_cost(&first_requirements, first_evidence);
    let second_estimate = estimate_graph_read_access_cost(&second_requirements, second_evidence);

    assert_eq!(first_requirements.digest(), second_requirements.digest());
    assert_eq!(
        first_estimate.attribution_rows(),
        second_estimate.attribution_rows()
    );
    assert_eq!(
        first_estimate.digest().as_str(),
        second_estimate.digest().as_str()
    );
}

#[test]
fn memory_estimate_names_each_relevant_access_structure_bucket() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-five.memory-buckets",
    );
    let family = frontier_search_family(&mut workspace, "phase-five-memory-buckets");
    let requirements = explain_graph_read_access_requirements_for_family(&family)
        .expect("requirements should derive");
    assert_eq!(
        requirements
            .rows()
            .iter()
            .find(|row| { row.kind() == &WorthQueryGraphReadAccessRequirementKind::ResultBuffer })
            .and_then(|row| row.result_pressure()),
        Some(&WorthQueryGraphReadResultPressure::CollectionWide)
    );
    let estimate = estimate_graph_read_access_cost(
        &requirements,
        derive_graph_read_cost_evidence(&requirements),
    );
    assert_memory_totals_equal_attribution_rows(&estimate);
    assert_frontier_search_bucket_contract(&estimate);
}

fn assert_memory_totals_equal_attribution_rows(estimate: &WorthQueryGraphReadAccessCostEstimate) {
    let memory = estimate.supported().memory();
    assert_eq!(
        memory.adjacency_bytes(),
        bucket_sum(estimate, |row| row.adjacency_bytes())
    );
    assert_eq!(
        memory.reverse_adjacency_bytes(),
        bucket_sum(estimate, |row| row.reverse_adjacency_bytes())
    );
    assert_eq!(
        memory.frontier_bytes(),
        bucket_sum(estimate, |row| row.frontier_bytes())
    );
    assert_eq!(
        memory.visited_bytes(),
        bucket_sum(estimate, |row| row.visited_bytes())
    );
    assert_eq!(
        memory.dedup_bytes(),
        bucket_sum(estimate, |row| row.dedup_bytes())
    );
    assert_eq!(
        memory.predicate_bytes(),
        bucket_sum(estimate, |row| row.predicate_bytes())
    );
    assert_eq!(
        memory.ordering_bytes(),
        bucket_sum(estimate, |row| row.ordering_bytes())
    );
    assert_eq!(
        memory.proof_bytes(),
        bucket_sum(estimate, |row| row.proof_bytes())
    );
    assert_eq!(
        memory.result_bytes(),
        bucket_sum(estimate, |row| row.result_bytes())
    );
}

fn assert_frontier_search_bucket_contract(estimate: &WorthQueryGraphReadAccessCostEstimate) {
    use WorthQueryGraphReadAccessRequirementKind as Kind;

    for (kind, bytes) in [
        (Kind::DirectionalAdjacency, 512),
        (Kind::ReverseAdjacency, 512),
        (Kind::TraversalWorkset, 512),
        (Kind::VisitedSet, 384),
        (Kind::DedupSet, 384),
        (Kind::PredicateSupport, 512),
        (Kind::OrderingSupport, 512),
        (Kind::ProofSupport, 256),
        (Kind::ResultBuffer, 2048),
    ] {
        assert_exact_bucket_contribution(
            estimate,
            kind.clone(),
            |memory| match kind {
                Kind::DirectionalAdjacency => memory.adjacency_bytes(),
                Kind::ReverseAdjacency => memory.reverse_adjacency_bytes(),
                Kind::TraversalWorkset => memory.frontier_bytes(),
                Kind::VisitedSet => memory.visited_bytes(),
                Kind::DedupSet => memory.dedup_bytes(),
                Kind::PredicateSupport => memory.predicate_bytes(),
                Kind::OrderingSupport => memory.ordering_bytes(),
                Kind::ProofSupport => memory.proof_bytes(),
                Kind::ResultBuffer => memory.result_bytes(),
                _ => unreachable!("frontier bucket matrix names only memory-owning rows"),
            },
            bytes,
        );
    }
}

#[test]
fn proof_memory_is_counted_without_consuming_inline_index_capacity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-five.proof-memory",
    );
    let family = frontier_search_family(&mut workspace, "phase-five-proof-memory");
    let requirements = explain_graph_read_access_requirements_for_family(&family)
        .expect("requirements should derive");
    let estimate = estimate_graph_read_access_cost(
        &requirements,
        derive_graph_read_cost_evidence(&requirements),
    );
    let memory = estimate.supported().memory();
    let index_bytes = memory.adjacency_bytes()
        + memory.reverse_adjacency_bytes()
        + memory.frontier_bytes()
        + memory.visited_bytes()
        + memory.dedup_bytes()
        + memory.predicate_bytes()
        + memory.ordering_bytes();

    assert_eq!(memory.index_bytes(), index_bytes);
    assert_eq!(
        memory.total_bytes(),
        index_bytes + memory.proof_bytes() + memory.result_bytes()
    );
    assert_eq!(
        WorthQueryGraphReadBudget::bounded(
            index_bytes,
            memory.result_bytes(),
            estimate.intrinsic().intermediate_set_size(),
        )
        .check_supported_cost(&estimate)
        .class()
        .kind(),
        &WorthQueryGraphReadBudgetClassKind::InlineEphemeralCandidate
    );
    assert_eq!(
        WorthQueryGraphReadBudget::bounded(
            index_bytes - 1,
            memory.result_bytes(),
            estimate.intrinsic().intermediate_set_size(),
        )
        .check_supported_cost(&estimate)
        .class()
        .kind(),
        &WorthQueryGraphReadBudgetClassKind::ExceedsInlineEphemeralBudget
    );
}

#[test]
fn intermediate_set_pressure_marks_broad_even_when_index_bytes_fit() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-five.intermediate-broadness",
    );
    let family = intermediate_pressure_family(&mut workspace, "phase-five-intermediate-broadness");
    let requirements = explain_graph_read_access_requirements_for_family(&family)
        .expect("requirements should derive");
    let estimate = estimate_graph_read_access_cost(
        &requirements,
        derive_graph_read_cost_evidence(&requirements),
    );

    assert!(
        estimate.supported().index_bytes()
            <= WorthQueryGraphReadBudget::inline_ephemeral_default()
                .max_inline_index_bytes()
                .expect("this case selects the explicit finite ephemeral strategy")
    );
    assert_eq!(
        estimate.complexity_contract().kind(),
        &WorthQueryGraphReadComplexityContractKind::BroadTraversalCandidate
    );
    assert_eq!(
        WorthQueryGraphReadBudget::bounded(
            estimate.supported().index_bytes(),
            estimate.supported().result_bytes(),
            estimate.intrinsic().intermediate_set_size() - 1,
        )
        .check_supported_cost(&estimate)
        .class()
        .kind(),
        &WorthQueryGraphReadBudgetClassKind::ExceedsInlineEphemeralBudget
    );
    assert_eq!(
        WorthQueryGraphReadBudget::inline_ephemeral_default()
            .check_supported_cost(&estimate)
            .class()
            .kind(),
        &WorthQueryGraphReadBudgetClassKind::InlineEphemeralCandidate
    );
}

#[test]
fn cost_estimation_is_planning_pure_not_execution_observation() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workspace(
        request_execution,
        "graph-read-access.phase-five.planning-purity",
    );
    let family = frontier_search_family(&mut workspace, "phase-five-planning-purity");
    let requirements = explain_graph_read_access_requirements_for_family(&family)
        .expect("requirements should derive");
    let observed = estimate_graph_read_access_cost_with_planning_observation(
        &requirements,
        derive_graph_read_cost_evidence(&requirements),
    );
    let estimate = observed.estimate();

    assert_eq!(estimate.counters().edge_scan_count(), 0);
    assert_eq!(estimate.counters().access_buffer_allocation_count(), 0);
    assert_eq!(observed.planning_observation().edge_read_count(), 0);
    assert_eq!(
        observed
            .planning_observation()
            .access_buffer_allocation_count(),
        0
    );
    assert_eq!(
        estimate.counters().requirement_row_count(),
        requirements.rows().len()
    );
    assert!(estimate.intrinsic().edge_touches() > 0);
    assert!(estimate.supported().index_bytes() > 0);
}
