#[path = "../../worth-execution/tests/lease_denial_cases/mod.rs"]
mod lease_denial_cases;
use lease_denial_cases::{Cause, CASES};
use worth_execution::LeaseDenial;
use worth_runtime_bridge::facade::{BridgeExecutionDenial as Own, SignalBridgeSinkError};
fn check(cause: Cause) {
    let raw = lease_denial_cases::denial(cause);
    let expected = match cause {
        Cause::Workers => Own::WorkerLimitExceedsParent,
        Cause::MemoryLimit => Own::MemoryLimitExceedsParent,
        Cause::Work => Own::WorkLimitExceedsParent,
        Cause::Memory => {
            let LeaseDenial::MemoryExhausted(memory) = raw else {
                panic!("memory cause required");
            };
            assert_eq!(
                memory.level,
                worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
            );
            assert!(memory.requested > memory.admitted);
            Own::MemoryExhausted(memory)
        }
        Cause::Overflow => Own::ChargedBytesOverflow,
        Cause::Nested => Own::UnrelatedNestedLease,
        Cause::Equivalence => Own::EquivalenceContractUnavailable,
        Cause::MissingScope => Own::NoActiveExecutionScope,
    };
    let SignalBridgeSinkError::Execution(actual) = raw.into() else {
        panic!("typed Bridge resource cause required");
    };
    assert_eq!(actual, expected, "{cause:?}");
}

#[test]
fn worker_limit_cause_survives_the_door() {
    check(CASES[0]);
}

#[test]
fn memory_limit_cause_survives_the_door() {
    check(CASES[1]);
}

#[test]
fn work_limit_cause_survives_the_door() {
    check(CASES[2]);
}

#[test]
fn memory_exhaustion_keeps_its_limit_and_amounts() {
    check(CASES[3]);
}

#[test]
fn charge_overflow_cause_survives_the_door() {
    check(CASES[4]);
}

#[test]
fn unrelated_nested_cause_survives_the_door() {
    check(CASES[5]);
}

#[test]
fn equivalence_cause_survives_the_door() {
    check(CASES[6]);
}

#[test]
fn absent_scope_is_distinct_from_an_unrelated_lease() {
    check(CASES[7]);
}
