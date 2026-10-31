//! Original backing remains admitted through projection and worker observation.
use super::*;

#[test]
fn finite_serial_memory_refuses_before_retaining_a_projection() {
    let limit = 4096;
    let memory = SerialMemoryBudget::new(Some(limit));
    let request =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let mut refusal = None;
    let mut no_retention = false;
    let outer = ExecutionRequest::serial(&request).run_native_with_checkpoint(|context| {
        let base = memory.charged_memory_bytes();
        let occupied = memory.reserve(limit - base).unwrap();
        refusal = context.retain_interruption_view(None).err();
        no_retention = memory.charged_memory_bytes() == limit;
        drop(occupied);
    });
    assert!(no_retention);
    assert!(matches!(
        refusal,
        Some(ExecutionInterruptionViewDenial::Request(
            WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(
                worth_execution::MemoryLimitDenial::Capacity { admitted: 0, .. }
            ))
        ))
    ));
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
    ));
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn finite_child_memory_cannot_be_replaced_by_the_root_projection_argument() {
    let root = ExecutionResourceLease::serial(lease_request()).unwrap();
    let mut grant = lease_request();
    grant.policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(4096), Some(10)),
    );
    let child = root.child(grant).unwrap();
    let mut refusal = None;
    let outer = ExecutionRequest::leased(&child).run_native_with_checkpoint(|context| {
        let occupied = child
            .reserve_memory(4096 - child.charged_memory_bytes())
            .unwrap();
        refusal = context.retain_interruption_view(Some(&root)).err();
        drop(occupied);
    });
    assert!(matches!(
        refusal,
        Some(ExecutionInterruptionViewDenial::Request(
            WorkCeilingDenial::Admission(worth_execution::LeaseDenial::MemoryExhausted(_))
        ))
    ));
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped))
    ));
    assert_eq!(child.charged_memory_bytes(), 0);
    assert_eq!(root.charged_memory_bytes(), 0);
}

#[test]
fn projected_observation_neither_exports_nor_clears_the_callers_work_stop() {
    let request = SerialRequest::from_policy(
        &lease_request().policy,
        CancellationSource::new().token(),
        None,
    );
    let mut view = None;
    let mut work_refused = false;
    let outer = ExecutionRequest::serial(&request).run_native_with_checkpoint(|context| {
        view = Some(context.retain_interruption_view(None).unwrap());
        work_refused = context.checkpoint(11) == Err(MapKernelStop::WorkCeiling);
    });
    assert!(work_refused);
    assert!(matches!(
        outer,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::WorkCeiling))
    ));
    assert_eq!(view.unwrap().check_live(), Ok(()));
}
