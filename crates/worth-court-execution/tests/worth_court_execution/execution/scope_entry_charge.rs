//! A scope's static entry charge and measured growth at the exact budget edge.
use std::cell::Cell;
use worth_execution::{
    CancellationSource, ChargedBytes, ExecutionRequest, ExecutionWorkCeiling, LeaseDenial,
    MapKernelFailure, MemoryLimitDenial, ScopeOutcome, ScopeStop, SerialMemoryBudget,
    SerialRequest,
};

/// One measured scope's outcome, the bytes its kernel saw held, and what it kept.
struct Measured<R, E> {
    outcome: ScopeOutcome<R, E>,
    entry: u64,
    kept: u64,
}

fn run_measured<R: ChargedBytes, E: ChargedBytes>(
    limit: Option<u64>,
    retained: u64,
    kernel: impl FnOnce() -> R,
) -> Measured<R, E> {
    let budget = SerialMemoryBudget::new(limit);
    let request =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut memory = budget.reserve(0).unwrap();
    let entry = Cell::new(0);
    let outcome = ExecutionRequest::serial(&request).run_measured_result(
        ExecutionWorkCeiling::new(None),
        retained,
        &mut memory,
        |_, _| {
            entry.set(budget.charged_memory_bytes());
            Ok::<_, MapKernelFailure<E>>(kernel())
        },
    );
    Measured {
        outcome,
        entry: entry.get(),
        kept: memory.bytes(),
    }
}

/// The bytes a scope holds while its kernel runs, under an unbounded budget.
fn entry_charge<R: ChargedBytes + Default, E: ChargedBytes>(retained: u64) -> u64 {
    let measured = run_measured::<R, E>(None, retained, R::default);
    assert!(measured.outcome.result.is_ok());
    measured.entry
}

#[test]
fn entry_charge_counts_both_payload_types_and_retained_bytes() {
    type Value = [u8; 32];
    type Failure = [u8; 24];
    let framework = entry_charge::<(), ()>(0);
    assert!(framework > 0, "the scope's context is charged");
    assert_eq!(entry_charge::<Value, Failure>(7), framework + 32 + 24 + 7);
}

#[test]
fn entry_refusal_precedes_invocation_only_below_the_exact_charge() {
    type Value = [u8; 32];
    let exact = entry_charge::<Value, ()>(7);
    for (budget, admitted) in [(0, false), (exact - 1, false), (exact, true)] {
        let ran = Cell::new(false);
        let measured = run_measured::<Value, ()>(Some(budget), 7, || {
            ran.set(true);
            [0; 32]
        });
        assert_eq!(ran.get(), admitted, "budget {budget}");
        assert_eq!(measured.outcome.result.is_ok(), admitted, "budget {budget}");
        if !admitted {
            assert!(matches!(
                measured.outcome.result,
                Err(ScopeStop::Admission(LeaseDenial::MemoryExhausted(
                    MemoryLimitDenial::Capacity { .. }
                )))
            ));
        }
        assert_eq!(measured.kept, 0);
    }
}

#[test]
fn measured_growth_is_admitted_to_the_exact_byte_and_keeps_only_the_payload() {
    let entry = entry_charge::<Vec<u8>, ()>(0);
    for (spare, admitted) in [(0_u64, true), (1, false)] {
        let measured =
            run_measured::<_, ()>(Some(entry + 64 - spare), 0, || Vec::<u8>::with_capacity(64));
        match measured.outcome.result {
            Ok(value) => {
                assert!(admitted);
                assert_eq!(measured.kept, value.additional_charged_bytes());
            }
            Err(stop) => {
                assert!(!admitted);
                assert!(matches!(
                    stop,
                    ScopeStop::Admission(LeaseDenial::MemoryExhausted(
                        MemoryLimitDenial::Capacity { .. }
                    ))
                ));
                assert_eq!(measured.kept, 0);
            }
        }
    }
}
