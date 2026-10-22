//! Arithmetic/prefix oracles, not native result or source-custody issuance.
use super::*;
pub(super) fn calls() -> Vec<(ComputationRead, Call)> {
    [(9, 2), (1, 1), (8, 3), (2, 2), (11, 1), (3, 2)]
        .into_iter()
        .enumerate()
        .map(|(i, (demand, consumed))| {
            let mut work = WorthQueryInvariantProjectionWork::default();
            for _ in 0..consumed {
                work.record_field();
            }
            (
                ComputationRead::ItemKey(worth_execution::PartitionItemId(i as u64)),
                (
                    ComputationCallCharge {
                        accounting: Some(ComputationCallAccounting {
                            work,
                            demand,
                            consumed,
                        }),
                        reached: Arc::from([]),
                    },
                    Some(i as u64 + 1),
                ),
            )
        })
        .collect()
}
#[test]
fn call_range_demand_and_consumption_match_independent_sequential_prefixes() {
    use crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget as Budget;
    let calls = calls();
    let tree = funded_tree(&calls);
    for start in 0..calls.len() {
        for end in start + 1..=calls.len() {
            for remaining in 0..40 {
                let mut sequential = Budget::bounded(remaining);
                let fits = calls[start..end].iter().all(|(_, (charge, _))| {
                    let cost = charge.accounting.unwrap();
                    sequential.carry(cost.demand, cost.consumed)
                });
                let sum = summary(&tree, &(start..end)).unwrap();
                assert_eq!(
                    sum.accounting.unwrap().work.field_reads(),
                    calls[start..end]
                        .iter()
                        .map(|(_, (charge, _))| charge.accounting.unwrap().work.field_reads())
                        .sum::<usize>(),
                    "descriptive reader work also survives range carriage"
                );
                let mut ranged = Budget::bounded(remaining);
                assert_eq!(
                    ranged.carry(
                        sum.accounting.unwrap().demand,
                        sum.accounting.unwrap().consumed
                    ),
                    fits,
                    "{start}..{end}, remaining{remaining}"
                );
                if fits {
                    assert_eq!(ranged.remaining(), sequential.remaining());
                }
                assert_eq!(
                    sum.declared,
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
    use crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget as Budget;
    let calls = calls();
    let tree = funded_tree(&calls);
    for remaining in 0..40 {
        let mut full = Budget::bounded(remaining);
        let stop = calls.iter().position(|(_, (charge, _))| {
            let charge = charge.accounting.unwrap();
            if full.can_afford(charge.demand) {
                assert!(full.consume(charge.consumed));
                false
            } else {
                true
            }
        });
        let mut ranged = Budget::bounded(remaining);
        let actual = visit(
            &tree,
            &(0..calls.len()),
            &BTreeSet::new(),
            &mut |entry| -> Result<bool, usize> {
                match entry {
                    CallVisit::Range(_, sum) => {
                        let cost = sum.accounting.unwrap();
                        Ok(ranged.carry(cost.demand, cost.consumed))
                    }
                    CallVisit::Leaf(at, _, charge, _) => {
                        let charge = charge.accounting.unwrap();
                        if !ranged.can_afford(charge.demand) {
                            return Err(at);
                        }
                        assert!(ranged.consume(charge.consumed));
                        Ok(true)
                    }
                }
            },
        )
        .err();
        assert_eq!(actual, stop, "remaining {remaining}");
        assert_eq!(
            (ranged.remaining(), ranged.exceeded()),
            (full.remaining(), full.exceeded())
        );
    }
}

fn funded_tree(calls: &[(ComputationRead, Call)]) -> Arc<Node> {
    use crate::domain_computation::primary_graph::{
        application_contribution::{test_policy, QueryRequestExecution},
        tests::fixture::live_scope,
    };
    let request = live_scope();
    let execution = QueryRequestExecution::open(
        worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(test_policy(
            std::num::NonZeroUsize::MIN,
            Some(1 << 24),
        )),
        &request,
    );
    build(0, calls, &execution).unwrap().unwrap()
}
