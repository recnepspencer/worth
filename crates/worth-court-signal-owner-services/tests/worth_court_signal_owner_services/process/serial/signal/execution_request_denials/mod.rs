#[allow(unused_imports)]
use crate::execution_request_denials::isolated_context::*;
use worth_execution::LeaseDenial;
use {
    crate::execution_request_denials::isolated_context::lease_denial_cases::Cause,
    crate::execution_request_denials::isolated_context::lease_denial_cases::CASES,
};
use {worth_signal::facade::SignalError, worth_signal::facade::SignalLeaseDenial as Own};

#[test]
fn worker_limit_cause_survives_the_door() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    check(CASES[0]);
}

#[test]
fn memory_limit_cause_survives_the_door() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    check(CASES[1]);
}

#[test]
fn work_limit_cause_survives_the_door() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    check(CASES[2]);
}

#[test]
fn memory_exhaustion_keeps_its_limit_and_amounts() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    check(CASES[3]);
}

#[test]
fn charge_overflow_cause_survives_the_door() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    check(CASES[4]);
}

#[test]
fn unrelated_nested_cause_survives_the_door() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    check(CASES[5]);
}

#[test]
fn equivalence_cause_survives_the_door() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    check(CASES[6]);
}

#[test]
fn absent_scope_is_distinct_from_an_unrelated_lease() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    check(CASES[7]);
}
