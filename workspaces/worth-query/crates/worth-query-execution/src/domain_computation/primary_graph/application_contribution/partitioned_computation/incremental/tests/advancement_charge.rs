//! Retained partitions replay the full computation's work and structural span.
use super::*;

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
    assert_eq!(retained.bill.charged_work(), full_cost.0 - 1);
    assert_eq!(
        retained.bill.charged_work(),
        unretained.bill.charged_work(),
        "carried and computed refusals have the same exact bill"
    );
}
