use std::sync::Arc;
use std::sync::Mutex;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::request;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::version_map;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::DecisionProvider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::OBSERVE_CALLS;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set_fixture::cleanup;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set_fixture::managed_decision_graph_run_with_provider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set_fixture::staged;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactLocator;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactRequest;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetDenialKind;
use worth_query_installation::facade::WorthQueryDecisionFactFamily;
use worth_query_installation::facade::WorthQueryDecisionFactKind;

#[test]
fn invalid_completeness_and_kind_deny_before_provider_contact() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let kinds = [
            WorthQueryDecisionFactKind::ObservedValue,
            WorthQueryDecisionFactKind::DomainStructuralProof,
        ];
        let versions = Arc::new(Mutex::new(version_map(2)));
        let families = kinds
            .iter()
            .enumerate()
            .map(|(index, kind)| {
                WorthQueryDecisionFactFamily::new(format!("family-{index}"), kind.clone()).unwrap()
            })
            .collect();
        let (mut running, graph) = managed_decision_graph_run_with_provider(
            DecisionProvider {
                versions: Arc::clone(&versions),
            },
            families,
            resource_request,
        );
        let staged = staged(execution, &mut running, &graph);
        let reads = staged.read_authority();
        let omission = reads
            .capture_decision_read_set(
                [request(0, &kinds[0])],
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .err()
            .expect("every required family must be represented");
        assert_eq!(
            omission.kind(),
            WorthQueryDecisionReadSetDenialKind::IncompleteRequiredFamilies
        );
        let mismatch = WorthQueryDecisionFactRequest::new(
            "family-0",
            WorthQueryDecisionFactLocator::predicate("locator-0").unwrap(),
        )
        .unwrap();
        let failure = reads
            .capture_decision_read_set(
                [mismatch],
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .err()
            .expect("installed family kind cannot be substituted");
        assert_eq!(
            failure.kind(),
            WorthQueryDecisionReadSetDenialKind::FamilyKindMismatch
        );
        assert_eq!(observations(&versions), 0);
        staged.abort();
        cleanup(running);
    });
}

#[test]
fn exact_family_count_and_duplicate_discovery_are_prevalidated_canonically() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let kind = WorthQueryDecisionFactKind::ObservedValue;
        let family = WorthQueryDecisionFactFamily::new("family-0", kind.clone())
            .unwrap()
            .with_exact_fact_count(2)
            .unwrap();
        let versions = Arc::new(Mutex::new(version_map(1)));
        let (mut running, graph) = managed_decision_graph_run_with_provider(
            DecisionProvider {
                versions: Arc::clone(&versions),
            },
            vec![family],
            resource_request,
        );
        let staged = staged(execution, &mut running, &graph);
        let reads = staged.read_authority();
        let one = request(0, &kind);
        let failure = reads
            .capture_decision_read_set(
                [one.clone(), one],
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .err()
            .expect("duplicate discovery cannot satisfy a two-fact family");
        assert_eq!(
            failure.kind(),
            WorthQueryDecisionReadSetDenialKind::IncompleteRequiredFacts
        );
        assert_eq!(observations(&versions), 0);
        staged.abort();
        cleanup(running);
    });
}

#[test]
fn duplicate_discovery_for_one_fact_calls_the_provider_once() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let kind = WorthQueryDecisionFactKind::ObservedValue;
        let versions = Arc::new(Mutex::new(version_map(1)));
        let (mut running, graph) = managed_decision_graph_run_with_provider(
            DecisionProvider {
                versions: Arc::clone(&versions),
            },
            vec![WorthQueryDecisionFactFamily::new("family-0", kind.clone()).unwrap()],
            resource_request,
        );
        let staged = staged(execution, &mut running, &graph);
        let reads = staged.read_authority();
        let one = request(0, &kind);
        let receipt = reads
            .capture_decision_read_set(
                [one.clone(), one],
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .expect("set semantics should canonicalize duplicate discovery");
        assert_eq!(receipt.fact_count(), 1);
        assert_eq!(receipt.counters().requested_facts(), 1);
        assert_eq!(receipt.counters().provider_calls(), 1);
        assert_eq!(observations(&versions), 1);
        staged.abort();
        cleanup(running);
    });
}

#[test]
fn bounded_family_accepts_attempt_exact_sets_only_within_installed_ceiling() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let kind = WorthQueryDecisionFactKind::ObservedValue;
        let family = WorthQueryDecisionFactFamily::new("family-0", kind.clone())
            .unwrap()
            .with_bounded_fact_count(2)
            .unwrap();
        let versions = Arc::new(Mutex::new(version_map(3)));
        let (mut running, graph) = managed_decision_graph_run_with_provider(
            DecisionProvider {
                versions: Arc::clone(&versions),
            },
            vec![family],
            resource_request,
        );
        let staged = staged(execution, &mut running, &graph);
        let reads = staged.read_authority();
        let empty = reads
            .capture_decision_read_set(
                std::iter::empty(),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .expect("bounded families permit an attempt with no realized facts");
        assert_eq!(empty.fact_count(), 0);
        let one = reads
            .capture_decision_read_set(
                [bounded_request(0)],
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .expect("one exact observed fact is within the installed ceiling");
        assert_eq!(one.fact_count(), 1);
        let two = reads
            .capture_decision_read_set(
                [bounded_request(0), bounded_request(1)],
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .expect("two exact observed facts reach the installed ceiling");
        assert_eq!(two.fact_count(), 2);
        let observations_before_denial = observations(&versions);
        let denial = reads
            .capture_decision_read_set(
                [bounded_request(0), bounded_request(1), bounded_request(2)],
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .err()
            .expect("the installed ceiling must deny before provider contact");
        assert_eq!(
            denial.kind(),
            WorthQueryDecisionReadSetDenialKind::DecisionFactBudgetExceeded
        );
        assert_eq!(observations(&versions), observations_before_denial);
        staged.abort();
        cleanup(running);
    });
}

pub fn bounded_request(index: usize) -> WorthQueryDecisionFactRequest {
    WorthQueryDecisionFactRequest::new(
        "family-0",
        WorthQueryDecisionFactLocator::observed(format!("locator-{index}")).unwrap(),
    )
    .unwrap()
}

pub fn observations(versions: &Arc<Mutex<std::collections::BTreeMap<String, u64>>>) -> u64 {
    versions
        .lock()
        .unwrap()
        .get(OBSERVE_CALLS)
        .copied()
        .unwrap_or_default()
}
