//! Native Bridge custody over nonempty planned reads and ordered patch delivery.
mod fixture;
use crate::facade::{BridgeDeliveryErrorKind, BridgeExecutionDenial, BridgeSnapshotReadErrorKind};
use fixture::{fixture, Fixture};
use std::num::NonZeroUsize;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, ExecutionScan, ExecutionWorkCeiling,
    MapKernelFailure, MapStop, ScanOutcome,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    PartitionIdentity,
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
fn cancelled_lease() -> worth_execution::ExecutionResourceLease<'static> {
    let source = CancellationSource::new();
    let lease = lease(1, 1 << 20, 100).controlled_child(source.token(), None);
    source.cancel();
    lease
}

/// Complete one real scan increment, then refuse the next under the parent's
/// one-unit ceiling. The parent retains that charge before Bridge is called.
fn exhaust_active_request(lease: &worth_execution::ExecutionResourceLease<'_>) {
    let first = PartitionIdentity::new(1);
    let second = PartitionIdentity::new(2);
    let scan =
        ExecutionScan::try_from_ordered(vec![first, second], vec![(first, 1_u64), (second, 2_u64)])
            .unwrap();
    let outcome = scan.run(Some(lease), 0_u64, 0, 0, 0, 0, |sum, item, work| {
        work.checkpoint(1)?;
        Ok::<_, MapKernelFailure<()>>((sum + item, ()))
    });
    assert_eq!(outcome.report().charged_work(), 1);
    assert!(matches!(outcome, ScanOutcome::Stopped {
        completed_state: 1,
        reason: MapStop::WorkExhausted { identity },
        ..
    } if identity == second));
}

fn assert_read_stopped(
    f: &Fixture,
    request: ExecutionRequest<'_, '_>,
    cause: BridgeExecutionDenial,
) {
    let error = f
        .context
        .read_packet(f.routes[0].read_packet(), request)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        BridgeSnapshotReadErrorKind::ExecutionDenied(cause)
    );
    assert_eq!(f.contacts.counts(), (0, 0, 0));
}

fn assert_delivery_stopped(
    f: &Fixture,
    request: ExecutionRequest<'_, '_>,
    cause: BridgeExecutionDenial,
) {
    let error = f
        .runtime
        .deliver_invalidation(f.routes[0].clone(), request)
        .unwrap_err();
    assert_eq!(
        error.kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(cause)
    );
    assert_eq!(f.contacts.counts(), (0, 0, 0));
    assert!(f.sink.deliveries().is_empty());
}

#[test]
fn stopped_requests_refuse_bridge_reads_before_reader_contact() {
    let f = fixture();
    let cancelled = cancelled_lease();
    assert_read_stopped(
        &f,
        ExecutionRequest::leased(&cancelled),
        BridgeExecutionDenial::Cancelled,
    );

    let active = lease(1, 1 << 20, 100);
    let request = ExecutionRequest::leased(&active);
    let (_, report) = request
        .run(ExecutionWorkCeiling::new(1), |_| {
            exhaust_active_request(&active);
            assert_read_stopped(&f, request, BridgeExecutionDenial::WorkCeiling);
        })
        .unwrap();
    assert_eq!(report.charged_work(), 1);

    // A declared zero budget with no active exhausted meter is not a stop.
    for (memory, work) in [(1 << 20, 0), (0, 100)] {
        let f = fixture();
        let lease = lease(1, memory, work);
        assert!(f
            .context
            .read_packet(f.routes[0].read_packet(), ExecutionRequest::leased(&lease))
            .is_ok());
        assert_eq!(f.contacts.counts(), (0, 1, 0));
    }
}

#[test]
fn stopped_requests_refuse_delivery_before_source_or_sink_contact() {
    let f = fixture();
    let cancelled = cancelled_lease();
    assert_delivery_stopped(
        &f,
        ExecutionRequest::leased(&cancelled),
        BridgeExecutionDenial::Cancelled,
    );

    let active = lease(1, 1 << 20, 100);
    let request = ExecutionRequest::leased(&active);
    let (_, report) = request
        .run(ExecutionWorkCeiling::new(1), |_| {
            exhaust_active_request(&active);
            assert_delivery_stopped(&f, request, BridgeExecutionDenial::WorkCeiling);
        })
        .unwrap();
    assert_eq!(report.charged_work(), 1);

    for (memory, work) in [(1 << 20, 0), (0, 100)] {
        let f = fixture();
        let lease = lease(1, memory, work);
        assert!(f
            .runtime
            .deliver_invalidation(f.routes[0].clone(), ExecutionRequest::leased(&lease))
            .is_ok());
        assert_eq!(f.contacts.counts(), (1, 1, 1));
        assert_eq!(f.sink.deliveries().len(), 1);
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
        assert_eq!(
            report.charged_work(),
            0,
            "a contact consults; it charges nothing"
        );
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
