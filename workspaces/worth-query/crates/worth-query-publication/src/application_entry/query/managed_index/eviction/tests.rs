//! The real execution owner's final poll cannot erase completed cleanup.
//! Native cache/guard effects are convicted separately in their owner court.
use super::completed_eviction;
use worth_execution::{
    CancellationSource, ExecutionRequest, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};
#[test]
fn managed_eviction_completed_cleanup_survives_the_actual_final_request_poll() {
    let cancellation = CancellationSource::new();
    let request =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&request);
    let mut completed = None;
    let mut released = false;
    let advanced = execution
        .run_native(|_| {
            let owned = Box::new([0_u8; 16]);
            drop(owned);
            released = true;
            cancellation.cancel();
            completed = Some(Ok(()));
        })
        .map(|_| ());
    assert!(released);
    assert!(matches!(
        advanced,
        Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    ));
    assert_eq!(completed_eviction(advanced, completed), Ok(()));
}
#[test]
fn managed_eviction_pre_effect_interruption_never_becomes_completed_success() {
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let request =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let mut completed = None;
    let advanced = ExecutionRequest::serial(&request)
        .run_native(|_| {
            completed = Some(Ok(()));
        })
        .map(|_| ());
    assert!(completed.is_none());
    assert!(matches!(
        completed_eviction(advanced, completed),
        Err(WorkCeilingDenial::Stopped(MapKernelStop::Cancelled))
    ));
}
