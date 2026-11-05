//! Arithmetic/prefix oracles, not native result or source-custody issuance.
use std::sync::Arc;
use std::collections::BTreeSet;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::Call;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::CallVisit;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::ComputationCallAccounting;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::ComputationCallCharge;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationRead;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::Node;
use worth_query_execution::court_doors_for_test::domain_computation::request_execution::QueryRequestExecution;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tree::build;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tree::summary;
use crate::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tests;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::operation_reader::computation_calls::sequence::tree::visit;
pub fn calls() -> Vec<(ComputationRead, Call)> {
    [(9, 2), (1, 1), (8, 3), (2, 2), (11, 1), (3, 2)]
        .into_iter()
        .enumerate()
        .map(|(i, (demand, consumed))| {
            (
                ComputationRead::ItemKey(worth_execution::PartitionItemId(i as u64)),
                (
                    ComputationCallCharge::new_for_test(
                        Some(ComputationCallAccounting::new_for_test(demand, consumed)),
                        Arc::from([]),
                    ),
                    Some(i as u64 + 1),
                ),
            )
        })
        .collect()
}
#[test]
fn call_range_demand_and_consumption_match_independent_sequential_prefixes() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget as Budget;
    let calls = calls();
    let tree = funded_tree(&calls);
    for start in 0..calls.len() {
        for end in start + 1..=calls.len() {
            for remaining in 0..40 {
                let mut sequential = Budget::bounded_for_test(remaining);
                let fits = calls[start..end].iter().all(|(_, (charge, _))| {
                    let cost = (*charge.accounting_for_test()).unwrap();
                    sequential
                        .carry_for_test((*cost.demand_for_test()), (*cost.consumed_for_test()))
                });
                let sum = summary(&tree, &(start..end)).unwrap();
                let mut ranged = Budget::bounded_for_test(remaining);
                assert_eq!(
                    ranged.carry_for_test(
                        (*(*sum.accounting_for_test()).unwrap().demand_for_test()),
                        (*(*sum.accounting_for_test()).unwrap().consumed_for_test())
                    ),
                    fits,
                    "{start}..{end}, remaining{remaining}"
                );
                if fits {
                    assert_eq!(ranged.remaining_for_test(), sequential.remaining_for_test());
                }
                assert_eq!(
                    (*sum.declared_for_test()),
                    Some(
                        calls[start..end]
                            .iter()
                            .map(|(_, (_, declared))| declared.unwrap())
                            .sum::<u64>()
                    )
                );
            }
        }
    }
}
#[test]
fn call_range_refusal_descends_to_the_first_actual_unfitting_call() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget as Budget;
    let calls = calls();
    let tree = funded_tree(&calls);
    for remaining in 0..40 {
        let mut full = Budget::bounded_for_test(remaining);
        let stop = calls.iter().position(|(_, (charge, _))| {
            let charge = (*charge.accounting_for_test()).unwrap();
            if full.can_afford_for_test((*charge.demand_for_test())) {
                assert!(full.consume_for_test((*charge.consumed_for_test())));
                false
            } else {
                true
            }
        });
        let mut ranged = Budget::bounded_for_test(remaining);
        let actual = visit(
            &tree,
            &(0..calls.len()),
            &BTreeSet::new(),
            &mut |entry| -> Result<bool, usize> {
                match entry {
                    CallVisit::Range(_, sum) => {
                        let cost = (*sum.accounting_for_test()).unwrap();
                        Ok(ranged
                            .carry_for_test((*cost.demand_for_test()), (*cost.consumed_for_test())))
                    }
                    CallVisit::Leaf(at, _, charge, _) => {
                        let charge = (*charge.accounting_for_test()).unwrap();
                        if !ranged.can_afford_for_test((*charge.demand_for_test())) {
                            return Err(at);
                        }
                        assert!(ranged.consume_for_test((*charge.consumed_for_test())));
                        Ok(true)
                    }
                }
            },
        )
        .err();
        assert_eq!(actual, stop, "remaining {remaining}");
        assert_eq!(
            (ranged.remaining_for_test(), ranged.exceeded_for_test()),
            (full.remaining_for_test(), full.exceeded_for_test())
        );
    }
}

pub fn funded_tree(calls: &[(ComputationRead, Call)]) -> Arc<Node> {
    use worth_query_execution::court_doors_for_test::domain_computation::request_execution::test_placement::test_policy;
    use worth_query_execution::court_doors_for_test::domain_computation::request_execution::QueryRequestExecution;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
    let request = live_scope();
    let execution = QueryRequestExecution::open_for_test(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 24),
        )),
        &request,
    );
    build(0, calls, &execution).unwrap().unwrap()
}
