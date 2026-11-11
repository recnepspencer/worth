//! Real World commit-to-dispatch support for causal tests.

use worth_relational::facade::transactions::RecordRef;

use worth_query_execution::court_doors_for_test::domain_computation::application_aftermath::external_effect::causal_event::WorthQueryAdmittedExternalDispatchAttempt;
use crate::domain_computation::primary_graph::tests::recoverable_commit_support::recoverable_application_world;
use crate::domain_computation::primary_graph::tests::recoverable_commit_support::two_recoverable_application_commits;

pub fn commit_observe_and_admit_fixture(
    seed: u8,
) -> (
    WorthQueryAdmittedExternalDispatchAttempt,
    worth_relational::facade::history::RelationalCommitReceipt,
    RecordRef,
    u64,
) {
    let (world, receipt) = recoverable_application_world(seed, &format!("dispatch-fixture-{seed}"));
    let observation = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &receipt)
        })
        .expect("the dispatch fixture opens its observation call")
        .expect("the exact World commit remains owner-readable")
        .expect("the real operation co-committed an outbox");
    let commit = observation.commit_reference().clone();
    let record_ref = observation.record_ref().clone();
    let relational_runtime_instance_id = observation.relational_runtime_instance_id();
    let admitted = world
        .application
        .admit_external_dispatch_attempt_for_test(observation)
        .expect("the owning runtime admits its World-bound observation");
    (admitted, commit, record_ref, relational_runtime_instance_id)
}

pub fn commit_observe_and_admit_twice_fixture(
    seed: u8,
) -> (
    WorthQueryAdmittedExternalDispatchAttempt,
    WorthQueryAdmittedExternalDispatchAttempt,
) {
    let (world, receipt) = recoverable_application_world(seed, &format!("dispatch-twice-{seed}"));
    let first = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &receipt)
        })
        .expect("the dispatch fixture opens its observation call")
        .expect("first exact owner read succeeds")
        .expect("the real operation co-committed an outbox");
    let second = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &receipt)
        })
        .expect("the dispatch fixture opens its observation call")
        .expect("second exact owner read succeeds")
        .expect("the real operation co-committed an outbox");
    (
        world
            .application
            .admit_external_dispatch_attempt_for_test(first)
            .expect("first physical attempt is admitted"),
        world
            .application
            .admit_external_dispatch_attempt_for_test(second)
            .expect("second physical attempt is admitted"),
    )
}

pub fn commit_distinct_records_and_admit_fixture() -> (
    WorthQueryAdmittedExternalDispatchAttempt,
    WorthQueryAdmittedExternalDispatchAttempt,
    RecordRef,
    RecordRef,
) {
    let (world, first_receipt, second_receipt) = two_recoverable_application_commits(231, 232);
    let first = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &first_receipt)
        })
        .expect("the dispatch fixture opens its observation call")
        .expect("the retained first commit remains owner-readable")
        .expect("the first operation co-committed an outbox");
    let second = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &second_receipt)
        })
        .expect("the dispatch fixture opens its observation call")
        .expect("the second commit remains owner-readable")
        .expect("the second operation co-committed an outbox");
    let first_ref = first.record_ref().clone();
    let second_ref = second.record_ref().clone();
    let first = world
        .application
        .admit_external_dispatch_attempt_for_test(first)
        .expect("the first World publication admits dispatch");
    let second = world
        .application
        .admit_external_dispatch_attempt_for_test(second)
        .expect("the second World publication admits dispatch");
    (first, second, first_ref, second_ref)
}

/// Admitted real owner evidence and the clock installed when its runtime was published.
pub fn commit_observe_and_admit_with_clock_fixture(seed: u8) -> (
    WorthQueryAdmittedExternalDispatchAttempt,
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::authorization_time::AuthorizationTimeController,
){
    let (world, receipt) = recoverable_application_world(seed, "no-effect-observation-time");
    let observation = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &receipt)
        })
        .unwrap()
        .unwrap()
        .unwrap();
    let admitted = world
        .application
        .admit_external_dispatch_attempt_for_test(observation)
        .unwrap();
    (admitted, world.authorization_time)
}
