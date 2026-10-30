#[allow(unused_imports)]
pub(crate) use {
    crate::request_differential::ordinary_context::checked_request,
    crate::request_differential::ordinary_context::serial, worth_signal::facade::SignalError,
};

// The ordinary entry publishes one task per epoch and the checked entry
// publishes grouped epochs, so the two charge different work for one plan.
// Results are the law here. Charged work is compared between requests that
// enter the same door: one worker and many, below.
#[test]
fn serial_entry_and_one_worker_lease_have_identical_results() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let serial = serial(300).unwrap();
    let one = checked_request(300, 1, u64::MAX).unwrap();
    assert_eq!(serial.0, one.0);
    assert!(serial.1.execution.last().unwrap().charged_work() > 0);
    assert!(one.1.execution.last().unwrap().charged_work() > 0);
}

#[test]
fn request_results_and_charged_work_are_identical_at_one_and_many_workers() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let one = checked_request(32, 1, u64::MAX).unwrap();
    let many = checked_request(32, 4, u64::MAX).unwrap();
    assert_eq!(one.0, many.0);
    assert_eq!(
        one.1.execution.last().unwrap().charged_work(),
        many.1.execution.last().unwrap().charged_work()
    );
}

#[test]
fn leased_three_hundred_tasks_still_refuse_a_callers_work_ceiling() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    // One unit cannot cover the required per-item checkpoints of 300 tasks.
    // This caller allowance is independent of a measured successful report.
    let worth_signal::facade::SignalError::ExecutionStopped(stop) =
        checked_request(300, 1, 1).unwrap_err()
    else {
        panic!("the caller's finite work ceiling must refuse the evaluation");
    };
    assert!(matches!(
        stop.reason(),
        worth_signal::facade::SignalExecutionStopReason::WorkExhausted { .. }
            | worth_signal::facade::SignalExecutionStopReason::Failure {
                cause: worth_signal::facade::SignalExecutionFailure::WorkCeiling,
                ..
            }
    ));
}

#[test]
fn checked_serial_request_and_one_worker_lease_have_identical_results_and_charged_work() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let serial = checked_request(300, 0, 16_000_000).unwrap();
    let one = checked_request(300, 1, 16_000_000).unwrap();
    assert_eq!(serial.0, one.0);
    assert_eq!(
        serial.1.execution.last().unwrap().charged_work(),
        one.1.execution.last().unwrap().charged_work()
    );
    eprintln!(
        "checked serial={} one-worker={}",
        serial.1.execution.last().unwrap().charged_work(),
        one.1.execution.last().unwrap().charged_work()
    );
}

#[path = "request_custody/mod.rs"]
pub(crate) mod request_custody;
