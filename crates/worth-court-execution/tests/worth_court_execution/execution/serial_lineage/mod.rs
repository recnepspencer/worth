#[allow(unused_imports)] use crate::serial_lineage::ordinary_context::*;
use {std::num::NonZeroUsize};
use {worth_execution::CancellationToken, worth_execution::ExecutionMemoryReservation, worth_execution::ExecutionRequest, worth_execution::LeaseDenial, worth_execution::MemoryLimitDenial, worth_execution::MemoryLimitLevel, worth_execution::SerialRequest};
use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy};

#[test]
fn nested_serial_scope_cannot_replace_parent_memory() {
    let parent = serial(4_096);
    let child = serial(65_536);
    let mut entered = false;
    worth_execution::ExecutionRequest::serial(&parent)
        .in_scope(|lease| {
            let _held = worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, 2_048).unwrap();
            worth_execution::ExecutionRequest::serial(&child)
                .in_scope(|lease| {
                    entered = true;
                    let denial =
                        worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, 4_096).unwrap_err();
                    assert!(matches!(
                        denial,
                        worth_execution::LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 1 },
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
    assert_eq!(
        parent
            .memory()
            .expect("explicit accounted request")
            .reserve(4_096)
            .unwrap()
            .bytes(),
        4_096
    );
}

#[test]
fn nested_serial_scope_keeps_the_child_limit() {
    let parent = serial(65_536);
    let child = serial(1_024);
    worth_execution::ExecutionRequest::serial(&parent)
        .in_scope(|_| {
            worth_execution::ExecutionRequest::serial(&child)
                .in_scope(|lease| {
                    assert!(matches!(
                        worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, 1_024),
                        Err(worth_execution::LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
                            ..
                        }))
                    ));
                })
                .unwrap();
        })
        .unwrap();
}
