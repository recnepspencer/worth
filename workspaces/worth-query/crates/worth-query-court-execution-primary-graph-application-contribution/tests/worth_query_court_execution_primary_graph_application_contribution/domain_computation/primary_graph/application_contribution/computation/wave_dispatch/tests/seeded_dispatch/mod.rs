//! Acquisition is recorded inside the shipped owned dispatcher, not in a racy kernel.

use std::num::NonZeroUsize;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::computation::wave_dispatch::tests::seeded_dispatch::ORDERS;
use crate::domain_computation::primary_graph::application_contribution::computation::wave_dispatch::tests::member_result;
use crate::domain_computation::primary_graph::application_contribution::computation::wave_dispatch::tests::run;
use crate::domain_computation::primary_graph::application_contribution::computation::wave_dispatch::tests::*;
use worth_query_execution::facade::domain_computation::place_managed_computations_on_this_thread_for_test as place;
use worth_query_execution::facade::domain_computation::WorthQueryExecutionPlacementForTest as Placement;

#[test]
fn certified_placement_perturbs_the_actual_owned_wave() {
    let _observation = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let mut orders = Vec::new();
    for width in [2, 1] {
        let mut histories = Vec::new();
        for seed in [0, 1] {
            place(Placement::Certified {
                workers: NonZeroUsize::new(width).unwrap(),
                seed,
            });
            ORDERS.with(|orders| orders.borrow_mut().clear());
            let (settled, report) = run(
                Some(width),
                vec![member_result(0, false), member_result(1, false)],
            );
            let values = settled
                .into_iter()
                .map(|value| {
                    value
                        .unwrap()
                        .into_slots_for_test()
                        .pop()
                        .unwrap()
                        .into_value_for_test::<(u64, bool)>()
                })
                .collect::<Vec<_>>();
            assert_eq!(report.accounted_cost(), None);
            histories.push(values);
            orders.push(ORDERS.with(|orders| orders.borrow_mut().pop().unwrap()));
        }
        assert_eq!(
            histories[0], histories[1],
            "a dispatch permutation preserves canonical result"
        );
        assert_eq!(histories[0], vec![(0, false), (1, false)]);
    }
    assert_ne!(
        orders[0], orders[1],
        "different certified seeds change the actual owned-wave acquisition order"
    );
    assert_eq!(
        orders[2], orders[3],
        "a one-worker serial lease ignores dispatch seeds: one named cell"
    );
}
