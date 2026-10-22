//! Acquisition is recorded inside the shipped owned dispatcher, not in a racy kernel.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::request_execution::{
    place_managed_computations_on_this_thread_for_test as place,
    WorthQueryExecutionPlacementForTest as Placement,
};
thread_local! { static ORDERS: std::cell::RefCell<Vec<Vec<usize>>> = const { std::cell::RefCell::new(Vec::new()) }; }
pub(in super::super) fn record(order: Vec<usize>) {
    ORDERS.with(|orders| orders.borrow_mut().push(order));
}
#[test]
fn certified_placement_perturbs_the_actual_owned_wave() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
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
                        .into_slots()
                        .pop()
                        .unwrap()
                        .into_value::<(u64, bool)>()
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
