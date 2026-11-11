//! Concrete outcome buffers stay admitted through growth and final data drop.
use super::*;
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
fn controller() -> u64 {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut observed = None;
    ExecutionRequest::serial(&host)
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed.unwrap()
}
#[test]
fn outcome_backing_tracks_actual_growth_and_final_drop() {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let entity = EntityId::new(crate::identity::data::PartitionId::main(), 0, 1);
    let mut outcome = MutationOutcome::entity_deleted(&request, entity).unwrap();
    assert_eq!(
        budget.charged_memory_bytes(),
        std::alloc::Layout::array::<MutationEvent>(outcome.events.capacity())
            .unwrap()
            .size() as u64
    );
    for _ in 0..17 {
        outcome
            .record_event(&request, MutationEvent::EntityDeleted { entity_id: entity })
            .unwrap();
    }
    assert_eq!(outcome.events.len(), 18);
    assert_eq!(
        budget.charged_memory_bytes(),
        std::alloc::Layout::array::<MutationEvent>(outcome.events.capacity())
            .unwrap()
            .size() as u64
    );
    drop(outcome);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn outcome_buffer_denial_is_the_actual_layout_and_original_stop() {
    let bill = std::alloc::Layout::array::<MutationEvent>(64)
        .unwrap()
        .size() as u64;
    let controller = controller();
    assert_ne!(controller, bill);
    let budget = SerialMemoryBudget::new(Some(controller.checked_add(bill).unwrap() - 1));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let request = ExecutionRequest::serial(&host);
    assert!(
        matches!(MutationOutcome::with_capacity(&request, 0, 64), Err(TransactionCommitError::Execution { denial, .. }) if matches!(denial.kind, CommitExecutionDenialKind::Cause(Cause::PolicyMemoryExhausted { requested, admitted, ancestor: 0 }) if requested == bill && admitted == bill - 1))
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    cancellation.cancel();
    assert!(
        matches!(MutationOutcome::with_capacity(&request, 0, 64), Err(TransactionCommitError::Execution { denial, .. }) if denial.kind == CommitExecutionDenialKind::Cause(Cause::Cancelled))
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}
