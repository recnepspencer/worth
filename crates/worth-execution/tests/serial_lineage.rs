use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionMemoryReservation, ExecutionRequest, LeaseDenial,
    MemoryLimitDenial, MemoryLimitLevel, SerialRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn serial(bytes: u64) -> SerialRequest {
    SerialRequest::from_policy(
        &ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, bytes, 10_000),
        ),
        CancellationToken::new(),
        None,
    )
}

#[test]
fn nested_serial_scope_cannot_replace_parent_memory() {
    let parent = serial(4_096);
    let child = serial(65_536);
    let mut entered = false;
    ExecutionRequest::serial(&parent)
        .in_scope(|lease| {
            let _held = ExecutionMemoryReservation::reserve_in_scope(lease, 2_048).unwrap();
            ExecutionRequest::serial(&child)
                .in_scope(|lease| {
                    entered = true;
                    let denial =
                        ExecutionMemoryReservation::reserve_in_scope(lease, 4_096).unwrap_err();
                    assert!(matches!(
                        denial,
                        LeaseDenial::MemoryExhausted(MemoryLimitDenial {
                            level: MemoryLimitLevel::Policy { ancestor: 1 },
                            ..
                        })
                    ));
                })
                .unwrap();
        })
        .unwrap();
    assert!(
        entered,
        "the child was admitted before its allocation was refused"
    );
    assert_eq!(parent.memory().reserve(4_096).unwrap().bytes(), 4_096);
}

#[test]
fn nested_serial_scope_keeps_the_child_limit() {
    let parent = serial(65_536);
    let child = serial(1_024);
    ExecutionRequest::serial(&parent)
        .in_scope(|_| {
            ExecutionRequest::serial(&child)
                .in_scope(|lease| {
                    assert!(matches!(
                        ExecutionMemoryReservation::reserve_in_scope(lease, 1_024),
                        Err(LeaseDenial::MemoryExhausted(MemoryLimitDenial {
                            level: MemoryLimitLevel::Policy { ancestor: 0 },
                            ..
                        }))
                    ));
                })
                .unwrap();
        })
        .unwrap();
}
