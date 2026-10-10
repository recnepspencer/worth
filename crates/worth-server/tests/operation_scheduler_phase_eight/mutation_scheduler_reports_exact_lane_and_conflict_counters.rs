//! mutation scheduler reports exact lane and conflict counters.
use super::*;

#[test]
fn mutation_scheduler_reports_exact_lane_and_conflict_counters() {
    let server = scheduler_server();
    let seed_plan = lower_product_mutation_plan(&server, "session-alpha", "", "task-seed");
    let stale_basis = seed_plan
        .query_handoff()
        .workspace()
        .with_advancement(|phase| {
            seed_plan
                .query_handoff()
                .workspace()
                .snapshot_identity(&phase)
        })
        .unwrap()
        .unwrap()
        .terminal_projection_for_reporting()
        .to_string();
    let executed = server
        .operation_scheduler()
        .schedule_batch([
            seed_plan,
            lower_product_mutation_plan(&server, "session-alpha", &stale_basis, "task-stale"),
            lower_product_mutation_plan(&server, "session-alpha", "", "task-closed"),
        ])
        .expect("product mutation plans should schedule")
        .execute();

    assert_eq!(executed.counters().admitted_mutation_slot_count(), 1);
    assert_eq!(executed.counters().completed_mutation_slot_count(), 1);
    assert_eq!(executed.counters().stale_basis_stop_count(), 1);
    assert_eq!(executed.counters().queue_closed_slot_count(), 1);
    match executed.outcomes()[1]
        .failure_posture()
        .expect("second slot should fail stale")
    {
        WorthServerSchedulerFailurePosture::StaleMutationBasis {
            expected_basis_digest,
            observed_basis_digest,
        } => {
            assert_eq!(expected_basis_digest, &stale_basis);
            assert_ne!(observed_basis_digest, &stale_basis);
        }
        other => panic!("expected stale mutation basis failure, got {other:?}"),
    }
    match executed.outcomes()[2]
        .failure_posture()
        .expect("third slot should close behind the stale failure")
    {
        WorthServerSchedulerFailurePosture::OrderedLaneClosed {
            scheduler_lane,
            failed_slot_ordinal,
        } => {
            assert_eq!(scheduler_lane, "product-draft:session-alpha:product-draft");
            assert_eq!(*failed_slot_ordinal, 1);
        }
        other => panic!("expected ordered lane closure, got {other:?}"),
    }
}
