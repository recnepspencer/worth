//! Real caller projection without another serial/leased scope for discovery.
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionInterruptionView,
    ExecutionInterruptionViewDenial, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
    MapKernelContext, MapKernelStop, SerialMemoryBudget, SerialRequest, WorkCeilingDenial,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

mod lineage;
mod retention;
mod stop_settlement;

fn lease_request() -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, Some(16 << 10), Some(10)),
        ),
        cancellation: CancellationSource::new().token(),
        deadline: None,
    }
}

fn home_step_controls(
    context: &mut MapKernelContext<'_, '_>,
    execution: ExecutionRequest<'_, '_>,
) -> Result<ExecutionInterruptionView, ExecutionInterruptionViewDenial> {
    let lease = match execution.allocation_policy() {
        ExecutionAllocationPolicy::Execution(lease) => Some(lease),
        ExecutionAllocationPolicy::SystemAllocation => None,
    };
    context.retain_interruption_view(lease)
}

#[test]
fn serial_projection_keeps_original_cancellation_after_the_scope_and_request_are_gone() {
    let cancellation = CancellationSource::new();
    let memory = SerialMemoryBudget::new(None);
    let mut view = None;
    {
        let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
        let execution = ExecutionRequest::serial(&request);
        execution
            .run_native_with_checkpoint(|context| {
                context.checkpoint(2).unwrap();
                view = Some(home_step_controls(context, execution).unwrap());
                assert_eq!(
                    context.charged_work(),
                    2,
                    "projection emits no second computation report"
                );
            })
            .unwrap();
    }
    let view = view.unwrap();
    assert_eq!(view.check_live(), Ok(()));
    assert_eq!(
        memory.charged_memory_bytes(),
        std::mem::size_of::<worth_execution::CancellationToken>() as u64
    );
    cancellation.cancel();
    assert_eq!(view.check_live(), Err(MapKernelStop::Cancelled));
    drop(view);
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn shared_controls_are_send_sync_and_static_and_release_only_at_the_final_owner() {
    fn worker(
        view: std::sync::Arc<ExecutionInterruptionView>,
    ) -> impl FnOnce() -> Result<(), MapKernelStop> + Send + Sync + 'static {
        move || view.check_live()
    }
    let cancellation = CancellationSource::new();
    let memory = SerialMemoryBudget::new(None);
    let mut view = None;
    {
        let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
        ExecutionRequest::serial(&request)
            .run_native_with_checkpoint(|context| {
                view = Some(context.retain_interruption_view(None).unwrap());
            })
            .unwrap();
    }
    let view = std::sync::Arc::new(view.unwrap());
    let charged = memory.charged_memory_bytes();
    assert!(charged > 0);
    let first_step = worker(std::sync::Arc::clone(&view));
    let terminal_step = worker(std::sync::Arc::clone(&view));
    assert_eq!(memory.charged_memory_bytes(), charged);
    drop(view);
    assert_eq!(first_step(), Ok(()));
    assert_eq!(memory.charged_memory_bytes(), charged);
    cancellation.cancel();
    assert_eq!(terminal_step(), Err(MapKernelStop::Cancelled));
    assert_eq!(memory.charged_memory_bytes(), 0);
}

#[test]
fn view_keeps_the_earliest_actual_serial_ancestor_deadline() {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    let parent_memory = SerialMemoryBudget::new(None);
    let child_memory = SerialMemoryBudget::new(None);
    let parent = SerialRequest::from_memory(
        parent_memory.clone(),
        CancellationSource::new().token(),
        Some(deadline),
    );
    let child = SerialRequest::from_memory(
        child_memory.clone(),
        CancellationSource::new().token(),
        Some(deadline + std::time::Duration::from_secs(60)),
    );
    let mut view = None;
    ExecutionRequest::serial(&parent)
        .run_native_with_checkpoint(|_| {
            ExecutionRequest::serial(&child)
                .run_native_with_checkpoint(|context| {
                    view = Some(context.retain_interruption_view(None).unwrap());
                })
                .unwrap();
        })
        .unwrap();
    let view = view.unwrap();
    assert!(parent_memory.charged_memory_bytes() > 0);
    assert_eq!(
        parent_memory.charged_memory_bytes(),
        child_memory.charged_memory_bytes()
    );
    std::thread::sleep(deadline.saturating_duration_since(std::time::Instant::now()));
    assert_eq!(view.check_live(), Err(MapKernelStop::DeadlineElapsed));
    drop(view);
    assert_eq!(parent_memory.charged_memory_bytes(), 0);
    assert_eq!(child_memory.charged_memory_bytes(), 0);
}

#[test]
fn leased_projection_also_observes_the_serial_invoking_ancestor() {
    let cancellation = CancellationSource::new();
    let parent =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let lease = ExecutionResourceLease::serial(lease_request()).unwrap();
    let mut view = None;
    ExecutionRequest::serial(&parent)
        .run_native_with_checkpoint(|_| {
            let execution = ExecutionRequest::leased(&lease);
            execution
                .run_native_with_checkpoint(|context| {
                    view = Some(home_step_controls(context, execution).unwrap());
                })
                .unwrap();
        })
        .unwrap();
    cancellation.cancel();
    assert_eq!(
        view.as_ref().unwrap().check_live(),
        Err(MapKernelStop::Cancelled)
    );
    drop(view);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
