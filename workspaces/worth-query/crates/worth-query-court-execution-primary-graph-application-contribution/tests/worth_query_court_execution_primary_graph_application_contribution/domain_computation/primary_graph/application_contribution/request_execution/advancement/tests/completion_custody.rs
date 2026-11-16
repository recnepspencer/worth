//! Effectful installed entry returns its real refusal without resetting controls.
use super::*;
use super::panic_custody::ResultDrop;
use std::cell::Cell;
use worth_query_admission::facade::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope};
use worth_query_execution::facade::domain_computation::{
    advancement_observations_on_this_thread_for_test, WorthQueryExecutionObservation,
};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

#[test]
fn installed_completion_keeps_the_returned_refusal_and_its_single_owner() {
    let _observer = WorthQueryExecutionObservation::install_on_this_thread();
    let clock =
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system();
    let world = installed_authorization_world(true, clock.clone());
    advancement_observations_on_this_thread_for_test();
    for accounted in [false, true] {
        let policy = if accounted {
            worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::MIN, None, None),
            )
        } else {
            worth_foundational::ExecutionRequestPolicy::for_workers(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                NonZeroUsize::MIN,
            )
        };
        let root = QueryRequestExecution::open_control_for_test(
            worth_runtime_world::facade::RuntimeWorldExecutionPlacement::Serial(policy),
            worth_execution::CancellationToken::new(),
            None,
        );
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(None, cancellation.token(), clock.clone());
        let drops = Cell::new(0);
        root.execution_request()
            .unwrap()
            .run_native_completion(|lease| {
                let returned = world
                    .application
                    .with_funded_application_completion(
                        &scope,
                        worth_execution::ExecutionAllocationPolicy::Execution(
                            lease.expect("the Query request lends its actual lease"),
                        ),
                        |_| {
                            cancellation.cancel();
                            Err::<(), _>(ResultDrop {
                                drops: &drops,
                                panic: false,
                            })
                        },
                    )
                    .expect("the returned refusal survives late settlement")
                    .expect_err("completion cannot turn the callback refusal into success");
                assert!(std::ptr::eq(returned.drops, &drops));
                assert_eq!(
                    drops.get(),
                    0,
                    "the caller owns the original returned refusal"
                );
                drop(returned);
                assert_eq!(drops.get(), 1);
            })
            .expect("the outer completion preserves its own returned value");
        let records = advancement_observations_on_this_thread_for_test();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].refusal, None);
        assert_eq!(records[0].report.has_accounting(), accounted);
        let refused = world.application.with_funded_application_completion(
            &scope,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            |_| panic!("retention cannot clear the original cancellation"),
        );
        assert_eq!(
            refused,
            Err(WorthQueryAdvancementDenial::Interrupted(
                WorthQueryManagedComputationInterruption::Cancelled,
            ))
        );
        advancement_observations_on_this_thread_for_test();
    }
}

#[test]
fn installed_completion_refuses_zero_work_before_entering_the_callback() {
    let _observer = WorthQueryExecutionObservation::install_on_this_thread();
    let clock =
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system();
    let world = installed_authorization_world(true, clock.clone());
    let scope = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let _restore = Restore(
        place(Placement::Serial),
        bound(Some(ExecutionBudget::new(NonZeroUsize::MIN, None, Some(0)))),
    );
    let refused = world.application.with_funded_application_completion(
        &scope,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        |_| panic!("zero work cannot enter an effectful callback"),
    );
    assert_eq!(
        refused,
        Err(WorthQueryAdvancementDenial::Resource(
            Resource::WorkExhausted
        ))
    );
}
