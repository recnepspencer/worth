//! A refused basis read fails its slot and closes the ordered mutation lane.
use super::*;

#[test]
fn refused_basis_advancement_fails_the_slot_and_closes_the_lane() {
    use worth_query_host::facade::primary_graph::{
        bound_advancement_requests_on_this_thread_for_test as bound,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    };
    let server = scheduler_server();
    let first = lower_product_mutation_plan(&server, "session-alpha", "", "task-refused");
    let basis = first
        .query_handoff()
        .workspace()
        .with_advancement(|phase| first.query_handoff().workspace().snapshot_identity(&phase))
        .unwrap()
        .unwrap()
        .terminal_projection_for_reporting()
        .to_string();
    let batch = server
        .operation_scheduler()
        .schedule_batch([
            lower_product_mutation_plan(&server, "session-alpha", &basis, "task-refused"),
            lower_product_mutation_plan(&server, "session-alpha", "", "task-closed"),
        ])
        .unwrap();
    struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
            bound(self.1);
        }
    }
    let _restore = Restore(
        place(Placement::Leased(std::num::NonZeroUsize::MIN)),
        bound(Some(worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            64 * 1024 * 1024,
            0,
        ))),
    );
    let executed = batch.execute();
    assert_eq!(executed.counters().admitted_mutation_slot_count(), 0);
    assert_eq!(executed.counters().completed_mutation_slot_count(), 0);
    assert_eq!(executed.counters().queue_closed_slot_count(), 1);
    match executed.outcomes()[0].failure_posture().unwrap() {
        WorthServerSchedulerFailurePosture::IsolatedRuntimeFailure { runtime_failure } => {
            // The zero-work case stops the outer advancement. A custody
            // refusal inside snapshot_identity has a separate typed stage.
            assert!(
                matches!(runtime_failure,
                    worth_server::WorthServerSchedulerRuntimeFailure::BasisAdvancementDenied {
                        denial: worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Resource(
                            worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted
                        ), ..
                    }),
                "{runtime_failure:?}"
            );
        }
        other => panic!("expected refused read, got {other:?}"),
    }
    assert!(matches!(executed.outcomes()[1].failure_posture().unwrap(),
        WorthServerSchedulerFailurePosture::OrderedLaneClosed { scheduler_lane, failed_slot_ordinal }
        if scheduler_lane == "product-draft:session-alpha:product-draft" && *failed_slot_ordinal == 0));
}
