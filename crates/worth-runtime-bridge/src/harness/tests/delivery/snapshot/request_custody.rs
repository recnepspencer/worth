//! Native Bridge custody over nonempty planned reads and ordered patch delivery.
mod fixture;
use crate::facade::{BridgeDeliveryErrorKind, BridgeExecutionDenial, BridgeSnapshotReadErrorKind};
use fixture::fixture;
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionRequest, ExecutionWorkCeiling, MemoryLimitLevel,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

fn lease(
    workers: usize,
    memory: u64,
    work: u64,
) -> worth_execution::ExecutionResourceLease<'static> {
    crate::snapshot::test_execution_lease_for_policy(
        ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), memory, work),
        ),
        CancellationToken::new(),
    )
}
fn assert_refusal(cause: BridgeExecutionDenial, work: u64) {
    if work == 0 {
        assert_eq!(cause, BridgeExecutionDenial::WorkCeiling);
    } else {
        let BridgeExecutionDenial::MemoryExhausted(memory) = cause else {
            panic!("native memory cause required: {cause:?}");
        };
        assert_eq!(memory.level, MemoryLimitLevel::Policy { ancestor: 0 });
        assert_eq!(memory.admitted, 0);
        assert!(memory.requested > 0);
    }
}
#[test]
fn zero_budgets_refuse_nonempty_bridge_reads_before_reader_contact() {
    let f = fixture();
    for (memory, work) in [(1 << 20, 0), (0, 100)] {
        let lease = lease(1, memory, work);
        let error = f
            .context
            .read_packet(f.routes[0].read_packet(), ExecutionRequest::leased(&lease))
            .unwrap_err();
        let BridgeSnapshotReadErrorKind::ExecutionDenied(cause) = error.kind() else {
            panic!("native request cause required: {error:?}");
        };
        assert_refusal(cause, work);
        assert_eq!(f.contacts.counts(), (0, 0, 0));
    }
}
#[test]
fn zero_budgets_refuse_delivery_before_source_or_sink_contact() {
    let f = fixture();
    for (memory, work) in [(1 << 20, 0), (0, 100)] {
        let lease = lease(1, memory, work);
        let error = f
            .runtime
            .deliver_invalidation(f.routes[0].clone(), ExecutionRequest::leased(&lease))
            .unwrap_err();
        let BridgeDeliveryErrorKind::ExecutionDenied(cause) = error.kind() else {
            panic!("native request cause required: {error:?}");
        };
        assert_refusal(cause, work);
        assert_eq!(f.contacts.counts(), (0, 0, 0));
        assert!(f.sink.deliveries().is_empty());
    }
}
#[test]
fn snapshot_bits_patch_order_and_actual_work_agree_across_widths() {
    let f = fixture();
    let mut oracle = None;
    for workers in [0, 1, 4] {
        f.contacts.reset();
        let serial = crate::snapshot::test_serial_request();
        let leased = (workers > 0).then(|| lease(workers, 1 << 20, 100));
        let request = leased.as_ref().map_or_else(
            || ExecutionRequest::serial(&serial),
            ExecutionRequest::leased,
        );
        let (bits, report) = request
            .run(ExecutionWorkCeiling::new(100), |_| {
                let read = f
                    .context
                    .read_packet(f.routes[0].read_packet(), request)
                    .unwrap();
                for route in &f.routes {
                    f.runtime
                        .deliver_invalidation(route.clone(), request)
                        .unwrap();
                }
                read
            })
            .unwrap();
        let deliveries = f
            .sink
            .deliveries()
            .into_iter()
            .skip(f.sink.deliveries().len() - 2)
            .map(|item| item.delivery)
            .collect::<Vec<_>>();
        let order = f.contacts.order.lock().unwrap().clone();
        assert_eq!(f.contacts.counts(), (2, 3, 2));
        assert_eq!(report.charged_work(), 7, "one actual charge per contact");
        assert_eq!(
            order,
            vec!["read", "open", "read", "sink", "open", "read", "sink"]
        );
        let observed = (bits, deliveries, report.charged_work(), order);
        if let Some(expected) = &oracle {
            assert_eq!(&observed, expected);
        } else {
            oracle = Some(observed);
        }
    }
}
