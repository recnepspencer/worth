//! Native storage, exact backing and wire/funding boundaries of the carrier.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::with_bootstrap_advancement;
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_relational::facade::identity::PartitionId;

pub(super) fn fact(ordinal: usize) -> Fact {
    Fact::SourceEntity {
        entity_id: EntityId::new(PartitionId::main(), (ordinal % 17) as u64, 1),
    }
}

pub(super) fn native(count: usize, control: StorageControl<'_, '_>) -> FactPayload {
    let mut builder = NativeFactPayloadBuilder::allocate(count, control).unwrap();
    for ordinal in 0..count {
        builder.push(fact(ordinal)).unwrap();
    }
    builder.seal().unwrap()
}

#[test]
fn native_chunks_preserve_order_duplicates_and_bidirectional_range_iteration() {
    let control = StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, None);
    for count in [0, 1, 31, 32, 33, 100, 1025] {
        let payload = native(count, control);
        let expected: Vec<_> = (0..count).map(fact).collect();
        assert_eq!(payload.len(), count);
        assert!(
            payload.retained_storage_bytes().unwrap() > original_storage_bytes(count).unwrap(),
            "retained custody includes native array wrappers and join backing"
        );
        assert!(payload.iter().eq(expected.iter()));
        assert!(payload.iter().rev().eq(expected.iter().rev()));
        assert!(payload.get(count).is_none());
        for begin in [0, count / 3, count] {
            let view = payload.slice(begin..count);
            assert_eq!(
                view.retained_storage_bytes(),
                payload.retained_storage_bytes(),
                "a range retains the entire original backing, not just visible slots"
            );
            assert!(view.iter().eq(expected[begin..].iter()));
            let mut both = view.iter();
            let mut expected = expected[begin..].iter();
            while expected.len() > 0 {
                assert_eq!(both.next(), expected.next());
                assert_eq!(both.next_back(), expected.next_back());
                assert_eq!(both.len(), expected.len());
            }
        }
        let clone = payload.clone();
        assert!(payload.same_backing(&clone));
        assert_eq!(payload, native(count, control));
        assert!(
            !payload.same_backing(&native(count, control)),
            "equal data never grants backing identity"
        );
    }
}

#[test]
fn decoded_views_keep_the_original_parse_reservation_and_wire_order() {
    let control = StorageControl::new(ExecutionAllocationPolicy::SystemAllocation, None);
    let funding = CodecPayloadReservation::admit(control, 10_000).unwrap();
    let weak = Arc::downgrade(&funding);
    let decoded = FactPayload::decoded((0..100).map(fact).collect(), funding);
    let view = decoded.slice(31..67);
    let encoded = super::super::encode(&view).unwrap();
    let expected: Vec<_> = (31..67).map(fact).collect();
    assert_eq!(encoded, super::super::encode(&expected).unwrap());
    assert!(!decoded.same_backing(&view));
    drop(decoded);
    assert!(weak.upgrade().is_some());
    let clone = view.clone();
    drop(view);
    assert!(weak.upgrade().is_some());
    assert!(clone.iter().eq(expected.iter()));
    drop(clone);
    assert!(weak.upgrade().is_none());
}

#[test]
fn native_chunks_keep_real_execution_funding_and_refuse_before_construction() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, 1 << 20, 1_000_000),
    );
    with_bootstrap_advancement(policy, |phase| {
        let execution = phase.execution_request();
        let ExecutionAllocationPolicy::Execution(lease) = execution.allocation_policy() else {
            panic!("native payer")
        };
        let control = StorageControl::new(execution.allocation_policy(), None);
        let before = lease.charged_memory_bytes();
        let payload = native(100, control);
        let charged = lease.charged_memory_bytes();
        assert!(charged > before);
        let view = payload.slice(32..33);
        let clone = view.clone();
        assert_eq!(
            lease.charged_memory_bytes(),
            charged,
            "clones share the actual admission"
        );
        drop(payload);
        drop(view);
        assert!(lease.charged_memory_bytes() > before);
        assert_eq!(clone.get(0), Some(&fact(32)));
        drop(clone);
        assert_eq!(lease.charged_memory_bytes(), before);
        let wire = super::super::encode(&(0..100).map(fact).collect::<Vec<_>>()).unwrap();
        let parsed = super::super::decode(&wire, None, execution.allocation_policy()).unwrap();
        let parsed_view = parsed.slice(31..67);
        let parsed_clone = parsed_view.clone();
        drop(parsed);
        drop(parsed_view);
        assert!(
            lease.charged_memory_bytes() > before,
            "a decoded subrange retains actual deep parse funding"
        );
        assert_eq!(parsed_clone.get(0), Some(&fact(31)));
        drop(parsed_clone);
        assert_eq!(lease.charged_memory_bytes(), before);
        let held = lease
            .reserve_memory(lease.available_memory_bytes())
            .unwrap();
        assert!(NativeFactPayloadBuilder::allocate(100, control).is_err());
        drop(held);
        let retry = native(100, control);
        assert_eq!(retry.len(), 100);
    })
    .unwrap();
}

#[test]
fn native_storage_returns_late_interruption_without_sealing_partial_evidence() {
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
        for interrupt_at_seal in [false, true] {
            let cancellation = worth_execution::CancellationSource::new();
            let child = parent.controlled_child(cancellation.token(), None);
            let control = StorageControl::new(ExecutionAllocationPolicy::Execution(&child), None);
            let mut builder = NativeFactPayloadBuilder::allocate(1, control).unwrap();
            if interrupt_at_seal {
                builder.push(fact(0)).unwrap();
            }
            cancellation.cancel();
            let cause = if interrupt_at_seal {
                builder.seal().err().expect("cancelled seal")
            } else {
                builder.push(fact(0)).unwrap_err()
            };
            assert_eq!(
                cause.kind(),
                worth_execution::ExecutionAllocationDenialKind::Cancelled
            );
            assert!(NativeFactPayloadBuilder::allocate(1, control).is_err());
        }
        assert_eq!(
            native(1, StorageControl::new(execution.allocation_policy(), None)).get(0),
            Some(&fact(0)),
            "a lawful fresh retry uses the uncancelled actual parent"
        );
    })
    .unwrap();
}
