//! New append/projection operations, separate from accepted container proofs.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::facts::payload::{
    tests::{fact, native},
    CodecPayloadReservation,
};
use crate::domain_computation::primary_graph::application_contribution::with_bootstrap_advancement;
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[test]
fn appends_keep_original_objects_funding_and_balanced_paths() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, 1 << 24, 1_000_000),
    );
    with_bootstrap_advancement(policy, |phase| {
        let execution = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(lease) = execution.allocation_policy() else {
            panic!("native payer")
        };
        let baseline = lease.charged_memory_bytes();
        let control = StorageControl::new(execution.allocation_policy(), None);
        let parse = CodecPayloadReservation::admit(control, 4096).unwrap();
        let weak = Arc::downgrade(&parse);
        let original = FactPayload::decoded(vec![fact(0)], parse);
        let mut joined = original.clone();
        for ordinal in 1..=257 {
            let mut tail =
                super::super::super::NativeFactPayloadBuilder::allocate_completed_read(1, control)
                    .unwrap();
            tail.push(fact(ordinal)).unwrap();
            let tail = tail.seal().unwrap();
            let pointer = tail.get(0).unwrap() as *const _;
            joined = joined.append_completed(tail, control).unwrap();
            assert!(std::ptr::eq(joined.get(ordinal).unwrap(), pointer));
            assert!(std::ptr::eq(
                original.get(0).unwrap(),
                joined.get(0).unwrap()
            ));
            assert!(joined.height <= (joined.len().next_power_of_two().ilog2() as usize) * 2);
        }
        assert!(joined
            .iter()
            .eq((0..=257).map(fact).collect::<Vec<_>>().iter()));
        assert!(joined
            .iter()
            .rev()
            .eq((0..=257).rev().map(fact).collect::<Vec<_>>().iter()));
        drop(original);
        assert!(
            weak.upgrade().is_some(),
            "original decode funding stays held through native joins"
        );
        let clone = joined.clone();
        drop(joined);
        assert!(weak.upgrade().is_some());
        drop(clone);
        assert!(weak.upgrade().is_none());
        assert_eq!(lease.charged_memory_bytes(), baseline);
    })
    .unwrap();
}

#[test]
fn canonical_reordering_of_joined_ranges_retains_exact_leaf_objects() {
    let control = StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, None);
    let source = native(100, control);
    let mut output = native(0, control);
    let order = [64, 65, 0, 1, 31, 32, 33, 99, 31];
    for ordinal in order {
        output = output
            .append_completed(source.slice(ordinal..ordinal + 1), control)
            .unwrap();
    }
    for (position, ordinal) in order.into_iter().enumerate() {
        assert!(std::ptr::eq(
            output.get(position).unwrap(),
            source.get(ordinal).unwrap()
        ));
    }
    assert!(output
        .iter()
        .eq(order.into_iter().map(fact).collect::<Vec<_>>().iter()));
}

#[test]
fn refused_append_keeps_old_owner_and_releases_all_partial_join_funding() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, 1 << 20, 1_000_000),
    );
    with_bootstrap_advancement(policy, |phase| {
        let execution = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(parent) = execution.allocation_policy() else {
            panic!("native payer")
        };
        let control = StorageControl::new(execution.allocation_policy(), None);
        let source = native(100, control);
        let tail = native(1, control);
        let held_bytes = parent.charged_memory_bytes();
        let full = parent
            .reserve_memory(parent.available_memory_bytes())
            .unwrap();
        assert!(source.append_completed(tail.clone(), control).is_err());
        drop(full);
        assert_eq!(parent.charged_memory_bytes(), held_bytes);
        assert_eq!(source.len(), 100);
        assert_eq!(source.get(99), Some(&fact(99)));
        let cancellation = worth_execution::CancellationSource::new();
        let child = parent.controlled_child(cancellation.token(), None);
        cancellation.cancel();
        assert!(source
            .append_completed(
                tail.clone(),
                StorageControl::new(ExecutionAllocationPolicy::Execution(&child), None)
            )
            .is_err());
        assert_eq!(parent.charged_memory_bytes(), held_bytes);
        let fresh = source.append_completed(tail, control).unwrap();
        assert_eq!(fresh.len(), 101, "lawful uncancelled retry");
    })
    .unwrap();
}
