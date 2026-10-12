//! Retained partitions replay the full computation's work and structural span.
use super::*;

#[test]
fn a_carried_stop_after_a_recomputed_slot_has_the_full_runs_bill() {
    let _observation = QueryObservation::install_on_this_thread();
    for width in [1, 2] {
        let world = installed_authorization_world(true);
        let installed = installed(StatusRead::Gather(0), sum);
        *installed.owner.steps.lock().unwrap() = Some([vec![1; 20], vec![1]]);
        let request = live_scope();
        let policy = crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(std::num::NonZeroUsize::new(width).unwrap(), 1 << 30);
        let billed = |prior, ceiling| {
            projected_attempt::attempt_billed(&world, &installed, prior, &request, RuntimeWorldExecutionPlacement::Leased { authority: super::super::super::super::request_execution::query_test_execution_authority_for_test(), policy }, Some(ceiling))
        };
        let first = billed(
            Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
            u64::MAX,
        );
        let prior = prior_of(first, true);
        let typed = prior
            .retained
            .as_ref()
            .unwrap()
            .typed
            .downcast_ref::<super::super::retained::RetainedPartitions<Parity, Number, u64>>()
            .unwrap();
        assert_eq!(
            typed
                .partitions
                .values()
                .map(|p| p.key.0)
                .collect::<Vec<_>>(),
            [0, 1],
            "the recomputed even slot precedes the carried odd slot"
        );
        let retained = billed(Some(prior), 16);
        let full = billed(None, 16);
        assert_eq!(
            retained.gathered,
            [0],
            "only the first canonical slot is recomputed"
        );
        assert!(full.outcome.is_err());
        assert_eq!(
            retained.outcome, full.outcome,
            "the later carried slot has the full run's typed stop"
        );
        assert_eq!(
            (retained.bill.charged_work(), retained.bill.charged_span()),
            (full.bill.charged_work(), full.bill.charged_span()),
            "recomputed prefix then carried stop has exactly the full run's bill"
        );
    }
}

#[test]
fn a_public_nested_map_cost_cannot_be_retained_as_owner_steps() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let request = live_scope();
    let policy = crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(std::num::NonZeroUsize::MIN, 1 << 30);
    let billed = || {
        projected_attempt::attempt_billed(
            &world,
            &installed,
            Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
            &request,
            RuntimeWorldExecutionPlacement::Serial(policy),
            Some(u64::MAX),
        )
    };
    let plain = billed();
    installed
        .owner
        .nested
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let nested = billed();
    assert_eq!(
        nested.outcome.as_ref().unwrap().0,
        plain.outcome.as_ref().unwrap().0
    );
    assert_eq!(
        nested.bill.charged_work(),
        plain.bill.charged_work() + 2 * (2 + 3),
        "two real nested maps add their declared work to their calling kernels"
    );
    assert_eq!(
        nested.bill.charged_span(),
        plain.bill.charged_span() + 3,
        "nested independent branches add their critical path to the calling branch"
    );
    assert!(
        nested.sealed.unwrap().is_none(),
        "nested work cannot be replayed from owner advance calls: nothing is retained"
    );
}

#[test]
fn carrying_every_partition_has_the_full_bill_and_the_same_one_unit_short_stop() {
    let _observation = QueryObservation::install_on_this_thread();
    let world = installed_authorization_world(true);
    let installed = installed(StatusRead::Gather(1), sum);
    let request = live_scope();
    let policy = crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(std::num::NonZeroUsize::MIN, 1 << 30);
    let billed = |prior, ceiling| {
        projected_attempt::attempt_billed(
            &world,
            &installed,
            prior,
            &request,
            RuntimeWorldExecutionPlacement::Serial(policy),
            Some(ceiling),
        )
    };
    let full = billed(
        Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
        u64::MAX,
    );
    let full_cost = (full.bill.charged_work(), full.bill.charged_span());
    let carried = billed(Some(prior_of(full, false)), u64::MAX);
    assert_eq!(
        (carried.bill.charged_work(), carried.bill.charged_span()),
        full_cost,
        "reuse never changes the advancement's work or span"
    );
    assert_eq!(carried.runs, [(Run::Incremental, None)]);
    assert!(
        carried.gathered.is_empty(),
        "the proof carries every partition"
    );
    let prior = prior_of(carried, false);
    let retained = billed(Some(prior), full_cost.0 - 1);
    let unretained = billed(None, full_cost.0 - 1);
    assert_eq!(
        retained.outcome, unretained.outcome,
        "reuse preserves the typed stop one unit below the full-run bill"
    );
    assert!(
        unretained.outcome.is_err(),
        "one unit below the full bill must refuse"
    );
    assert_eq!(
        retained.bill.charged_span(),
        unretained.bill.charged_span(),
        "the one-unit-short stop preserves span"
    );
    assert_eq!(retained.bill.charged_work(), full_cost.0 - 1);
    assert_eq!(
        retained.bill.charged_work(),
        unretained.bill.charged_work(),
        "carried and computed refusals have the same exact bill"
    );
}

#[test]
fn a_ceiling_inside_carried_checkpoint_steps_has_the_full_runs_exact_stop() {
    use super::super::super::super::request_execution::{
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    };
    let _observation = QueryObservation::install_on_this_thread();
    for width in [1, 2] {
        let previous = place(Placement::Leased(
            std::num::NonZeroUsize::new(width).unwrap(),
        ));
        for steps in [vec![1; 20], vec![3, 5, 7, 9]] {
            let world = installed_authorization_world(true);
            let installed = installed(StatusRead::Gather(1), sum);
            // The even partition is carried, while the status-reading odd one
            // is gathered again. Each kernel must encounter the same local
            // ceiling before canonical settlement accepts or discards it.
            *installed.owner.steps.lock().unwrap() = Some([vec![1], steps.clone()]);
            let request = live_scope();
            let policy = crate::domain_computation::primary_graph::application_contribution::request_execution::test_policy(std::num::NonZeroUsize::MIN, 1 << 30);
            let billed = |prior, ceiling| {
                projected_attempt::attempt_billed(
                    &world,
                    &installed,
                    prior,
                    &request,
                    RuntimeWorldExecutionPlacement::Serial(policy),
                    Some(ceiling),
                )
            };
            let first = billed(
                Some(ComputationPrior::new(edition(), Err(Cause::FirstRun), None)),
                u64::MAX,
            );
            let retained = billed(Some(prior_of(first, true)), 16);
            let full = billed(None, 16);
            assert_eq!(
                retained.gathered,
                [1],
                "the refusal crosses mixed carried and recomputed slots"
            );
            assert!(full.outcome.is_err());
            assert_eq!(
                retained.outcome, full.outcome,
                "carried checkpoints preserve the typed canonical stop"
            );
            assert_eq!(
                (retained.bill.charged_work(), retained.bill.charged_span()),
                (full.bill.charged_work(), full.bill.charged_span()),
                "a refusal inside carried work preserves the accepted checkpoint work and span"
            );
            if steps[0] == 3 {
                assert!(
                    full.bill.charged_work() < 16,
                    "an unequal next step refuses below the ceiling"
                );
            }
        }
        place(previous);
    }
}
