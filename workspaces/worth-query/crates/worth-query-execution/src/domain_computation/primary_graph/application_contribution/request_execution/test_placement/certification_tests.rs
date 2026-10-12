//! Diagnostic interruption preserves the request cause rather than panicking.
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn a_request_cancelled_during_certification_reports_cancellation() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    let cancellation = worth_execution::CancellationSource::new();
    let lease = query_test_execution_authority_for_test()
        .request_lease(worth_execution::LeaseRequest {
            policy: test_policy(NonZeroUsize::MIN, 1 << 24),
            cancellation: cancellation.token(),
            deadline: None,
        })
        .unwrap();
    let calls = AtomicUsize::new(0);
    let map =
        ExecutionMap::<_, u64>::from_keyless_partitions(std::collections::BTreeMap::from([(
            worth_foundational::PartitionIdentity::new(1),
            worth_execution::KeylessPartition {
                value: 3_u64,
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            },
        )]))
        .unwrap();
    let kernel = |value: &u64, context: &mut MapKernelContext<'_, '_>| {
        if calls.fetch_add(1, Ordering::SeqCst) != 0 {
            cancellation.cancel();
        }
        context.checkpoint(*value)?;
        Ok::<_, MapKernelFailure<()>>(*value)
    };
    let combine: fn(&u64, &u64) -> u64 = |a, b| a + b;
    let (tree, report, _) = map
        .run_reduce(Some(&lease), kernel, 0_u64, combine, 8, 0)
        .unwrap();
    let stopped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        certify(&map, &lease, 1, kernel, 0_u64, combine, 8, (&tree, &report))
    }));
    assert!(
        stopped.is_ok(),
        "a request stop during certification is returned with its cause, never a panic"
    );
    assert_eq!(
        stopped.unwrap(),
        Err(worth_execution::WorkCeilingDenial::Stopped(
            worth_execution::MapKernelStop::Cancelled
        )),
        "the diagnostic returns the request cancellation by value"
    );
}

#[test]
fn a_diagnostic_memory_reservation_cannot_refuse_the_request() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    let lease = query_test_execution_authority_for_test()
        .request_lease(worth_execution::LeaseRequest {
            policy: test_policy(NonZeroUsize::MIN, 1024),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let _held = lease.reserve_memory(1024).unwrap();
    let (diagnostic, _) = worth_execution::ExecutionWorkCeiling::new(u64::MAX)
        .run_detached_for_test(&lease, |diagnostic| {
            diagnostic.reserve_memory(4096).map(|hold| hold.bytes())
        })
        .unwrap();
    assert_eq!(
        diagnostic,
        Ok(4096),
        "the diagnostic has its own memory room even when the request is full"
    );
    assert_eq!(
        lease.charged_memory_bytes(),
        1024,
        "the diagnostic leaves the request's memory unchanged"
    );
}

#[test]
fn a_certification_disagreement_keeps_its_own_panic_message() {
    let _observation = crate::domain_computation::execution_observation::WorthQueryExecutionObservation::install_on_this_thread();
    let lease = query_test_execution_authority_for_test()
        .request_lease(worth_execution::LeaseRequest {
            policy: test_policy(NonZeroUsize::MIN, 1 << 24),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        })
        .unwrap();
    let calls = AtomicUsize::new(0);
    let map =
        ExecutionMap::<_, u64>::from_keyless_partitions(std::collections::BTreeMap::from([(
            worth_foundational::PartitionIdentity::new(1),
            worth_execution::KeylessPartition {
                value: 3_u64,
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            },
        )]))
        .unwrap();
    let kernel = |value: &u64, context: &mut MapKernelContext<'_, '_>| {
        context.checkpoint(*value)?;
        Ok::<_, MapKernelFailure<()>>(if calls.fetch_add(1, Ordering::SeqCst) == 0 {
            *value
        } else {
            *value + 1
        })
    };
    let combine: fn(&u64, &u64) -> u64 = |a, b| a + b;
    let (tree, report, _) = map
        .run_reduce(Some(&lease), kernel, 0_u64, combine, 8, 0)
        .unwrap();
    let payload = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        certify(&map, &lease, 1, kernel, 0_u64, combine, 8, (&tree, &report))
    }))
    .unwrap_err();
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap();
    assert_eq!(
        message, "the run's result differs from the certified one under seed 1",
        "certification retains the assertion that convicted its disagreement"
    );
}
