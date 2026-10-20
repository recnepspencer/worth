use super::*;
use crate::{
    backend::{run_checked_batch, BackendKind, KernelFailure},
    tests::{batch, request},
};

#[test]
fn serial_lease_uses_the_existing_caller_thread_witness() {
    let lease = ExecutionResourceLease::serial(request(4, 4_096, 20)).unwrap();
    assert!(
        lease.authority.pool.is_none(),
        "a serial backing has no thread pool"
    );
    assert_eq!(lease.policy().budget().max_workers(), NonZeroUsize::MIN);
    let child = lease.controlled_child(crate::CancellationToken::new(), None);
    let caller = std::thread::current().id();
    let admitted = batch(&[1, 2, 3], 8);
    let outcome = run_checked_batch(
        Some(&child),
        &admitted,
        BackendKind::Native,
        &|value, context| {
            assert_eq!(std::thread::current().id(), caller);
            context.checkpoint(*value)?;
            Ok::<_, KernelFailure<()>>(*value)
        },
    );
    assert_eq!(outcome.stop, None);
    assert_eq!(outcome.values, vec![1, 2, 3]);
    assert_eq!(outcome.report.charged_work(), 1 + 2 + 3);
    assert_eq!(outcome.report.charged_span(), 3);
    assert_eq!(outcome.report.resolved_posture(), ExecutionPosture::Serial);
}

#[test]
fn serial_children_share_memory_and_interruption() {
    let source = crate::CancellationSource::new();
    let mut declared = request(1, 32, 20);
    declared.cancellation = source.token();
    let lease = ExecutionResourceLease::serial(declared).unwrap();
    let child = lease.controlled_child(crate::CancellationToken::new(), None);
    let held = lease.reserve_memory(24).unwrap();
    let denial = child.reserve_memory(9).unwrap_err();
    assert_eq!(
        denial,
        crate::MemoryLimitDenial::Capacity {
            requested: 9,
            admitted: 8,
            level: crate::MemoryLimitLevel::Policy { ancestor: 1 }
        }
    );
    drop(held);
    let held = child.reserve_memory(32).unwrap();
    source.cancel();
    assert!(child.is_cancelled());
    drop(held);
    // The inline authority is shared; this request owns only its lineage.
    assert_eq!(lease.charged_memory_bytes(), 0);
}

#[test]
fn a_foreign_serial_root_cannot_reset_an_active_lease() {
    let lease = ExecutionResourceLease::serial(request(1, 32, 20)).unwrap();
    let _entry = lease.reserve_entry(16).unwrap();
    let _worker = lease.enter_worker_context();
    let foreign = ExecutionResourceLease::serial(request(1, 32, 20)).unwrap();
    assert!(matches!(
        foreign.reserve_entry(1),
        Err(LeaseDenial::UnrelatedNestedLease)
    ));
    let child = lease.controlled_child(crate::CancellationToken::new(), None);
    assert!(child.reserve_entry(1).is_ok());
}
