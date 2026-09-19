//! Real World commit-to-dispatch support for causal tests.

use worth_relational::facade::transactions::RecordRef;

use crate::domain_computation::application_aftermath::external_effect::WorthQueryAdmittedExternalDispatchAttempt;
use crate::domain_computation::primary_graph::{
    recoverable_application_world, two_recoverable_application_commits,
};

pub(in crate::domain_computation) fn commit_observe_and_admit_fixture(
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
        .observe_committed_dispatch_outbox(&receipt)
        .expect("the exact World commit remains owner-readable")
        .expect("the real operation co-committed an outbox");
    let commit = observation.commit_reference().clone();
    let record_ref = observation.record_ref().clone();
    let relational_runtime_instance_id = observation.relational_runtime_instance_id();
    let admitted = world
        .application
        .admit_external_dispatch_attempt(observation)
        .expect("the owning runtime admits its World-bound observation");
    (admitted, commit, record_ref, relational_runtime_instance_id)
}

pub(in crate::domain_computation) fn commit_observe_and_admit_twice_fixture(
    seed: u8,
) -> (
    WorthQueryAdmittedExternalDispatchAttempt,
    WorthQueryAdmittedExternalDispatchAttempt,
) {
    let (world, receipt) = recoverable_application_world(seed, &format!("dispatch-twice-{seed}"));
    let first = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .expect("first exact owner read succeeds")
        .expect("the real operation co-committed an outbox");
    let second = world
        .application
        .observe_committed_dispatch_outbox(&receipt)
        .expect("second exact owner read succeeds")
        .expect("the real operation co-committed an outbox");
    (
        world
            .application
            .admit_external_dispatch_attempt(first)
            .expect("first physical attempt is admitted"),
        world
            .application
            .admit_external_dispatch_attempt(second)
            .expect("second physical attempt is admitted"),
    )
}

pub(in crate::domain_computation) fn commit_distinct_records_and_admit_fixture() -> (
    WorthQueryAdmittedExternalDispatchAttempt,
    WorthQueryAdmittedExternalDispatchAttempt,
    RecordRef,
    RecordRef,
) {
    let (world, first_receipt, second_receipt) = two_recoverable_application_commits(231, 232);
    let first = world
        .application
        .observe_committed_dispatch_outbox(&first_receipt)
        .expect("the retained first commit remains owner-readable")
        .expect("the first operation co-committed an outbox");
    let second = world
        .application
        .observe_committed_dispatch_outbox(&second_receipt)
        .expect("the second commit remains owner-readable")
        .expect("the second operation co-committed an outbox");
    let first_ref = first.record_ref().clone();
    let second_ref = second.record_ref().clone();
    let first = world
        .application
        .admit_external_dispatch_attempt(first)
        .expect("the first World publication admits dispatch");
    let second = world
        .application
        .admit_external_dispatch_attempt(second)
        .expect("the second World publication admits dispatch");
    (first, second, first_ref, second_ref)
}
