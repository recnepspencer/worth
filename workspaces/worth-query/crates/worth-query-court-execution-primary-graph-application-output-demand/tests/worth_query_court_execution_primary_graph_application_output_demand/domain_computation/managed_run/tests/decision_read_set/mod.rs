use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::DecisionProvider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::all_fact_kinds;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::families;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::requests;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set::version_map;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use crate::domain_computation::managed_run::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactComparisonAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactComparisonEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactEvidenceView;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactLocator;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactProvider;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactRequest;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactRequestView;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFailure;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionPlanView;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionToken;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionTokenAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionView;
use worth_query_installation::facade::WorthQueryDecisionFactFamily;
use worth_query_installation::facade::WorthQueryDecisionFactKind;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set_fixture::cleanup;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set_fixture::managed_decision_graph_run_with_provider;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::decision_read_set_fixture::staged;
#[test]
fn all_fact_families_capture_canonically_and_compare_without_false_conflicts() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let kinds = all_fact_kinds();
        let families = families(&kinds);
        let versions = Arc::new(Mutex::new(version_map(kinds.len())));
        let (mut running, graph) = managed_decision_graph_run_with_provider(
            DecisionProvider {
                versions: Arc::clone(&versions),
            },
            families,
            resource_request,
        );
        let staged = staged(execution, &mut running, &graph);
        {
            let reads = staged.read_authority();
            let requests = requests(&kinds);
            let first = reads
                .capture_decision_read_set(
                    requests.clone(),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    None,
                )
                .expect("all installed fact families should capture");
            let second = reads
                .capture_decision_read_set(
                    requests.into_iter().rev(),
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                    None,
                )
                .expect("discovery order must not affect capture");
            assert_eq!(first.identity(), second.identity());
            assert_eq!(first.fact_count(), kinds.len());
            let outcome = reads
                .compare_decision_read_set(&phase.borrowed_request_for_test(), first)
                .expect("unchanged provider facts should compare");
            let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) = outcome else {
                panic!("unchanged facts must remain fresh");
            };
            assert_eq!(fresh.counters().compared_facts(), kinds.len());
            assert_eq!(fresh.counters().false_conflicts(), 0);
        }
        staged.abort();
        cleanup(running);
    });
}
#[test]
fn every_relevant_family_stales_independently_while_unrelated_axes_remain_fresh() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let kinds = all_fact_kinds();
        let versions = Arc::new(Mutex::new(version_map(kinds.len())));
        let (mut running, graph) = managed_decision_graph_run_with_provider(
            DecisionProvider {
                versions: Arc::clone(&versions),
            },
            families(&kinds),
            resource_request,
        );
        let staged = staged(execution, &mut running, &graph);
        {
            let reads = staged.read_authority();
            for changed_index in 0..kinds.len() {
                let unchanged = reads
                    .capture_decision_read_set(
                        requests(&kinds),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                        None,
                    )
                    .expect("complete decision facts should capture");
                {
                    let mut versions = versions.lock().unwrap();
                    for unrelated in [
                        "unrelated-entity",
                        "unrelated-aspect",
                        "unrelated-partition",
                        "unrelated-artifact-family",
                    ] {
                        versions.insert(unrelated.to_owned(), changed_index as u64 + 2);
                    }
                }
                let WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) = reads
                    .compare_decision_read_set(&phase.borrowed_request_for_test(), unchanged)
                    .unwrap()
                else {
                    panic!("unrelated semantic drift must remain fresh");
                };
                assert_eq!(fresh.counters().false_conflicts(), 0);

                let changed = reads
                    .capture_decision_read_set(
                        requests(&kinds),
                        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                        None,
                    )
                    .expect("complete decision facts should recapture");
                versions
                    .lock()
                    .unwrap()
                    .insert(format!("locator-{changed_index}"), 2);
                let WorthQueryDecisionReadSetFreshnessOutcome::Stale(stale) = reads
                    .compare_decision_read_set(&phase.borrowed_request_for_test(), changed)
                    .unwrap()
                else {
                    panic!("each relevant family must independently stale the receipt");
                };
                assert_eq!(stale.stale_fact_count(), 1);
                versions
                    .lock()
                    .unwrap()
                    .insert(format!("locator-{changed_index}"), 1);
            }
        }
        staged.abort();
        cleanup(running);
    });
}
