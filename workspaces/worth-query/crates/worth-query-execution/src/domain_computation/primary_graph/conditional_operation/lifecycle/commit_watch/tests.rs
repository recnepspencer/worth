use super::*;
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
fn lease(memory: Option<u64>) -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, memory, None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}
fn entity(n: u64) -> EntityId {
    EntityId::new(PartitionId(1), n, 0)
}
#[test]
fn original_watch_backing_is_shared_and_successor_allocation_refuses_without_releasing_it() {
    let lease = lease(Some(32 * 1024));
    let request = ExecutionRequest::leased(&lease);
    let entries = WorthQueryConditionalCommitWatchSet::retain_entities(
        &request,
        None,
        [entity(1)].into_iter(),
        1,
        None,
    )
    .unwrap();
    let original = WorthQueryConditionalCommitWatchSet {
        entities: Some(entries),
        whole_graph: false,
    };
    let held = lease.charged_memory_bytes();
    assert!(held > 0);
    let clone = original.clone();
    assert_eq!(lease.charged_memory_bytes(), held);
    let denied = WorthQueryConditionalCommitWatchSet::retain_entities(
        &request,
        Some(&original),
        [entity(2)].into_iter(),
        1,
        Some(1),
    );
    assert!(matches!(denied, Err(ref cause) if cause.kind() == Kind::RetentionCapacityExhausted));
    assert_eq!(
        original.entities().iter().copied().collect::<Vec<_>>(),
        [entity(1)]
    );
    assert_eq!(lease.charged_memory_bytes(), held);
    let full = lease
        .reserve_memory(lease.available_memory_bytes().unwrap())
        .unwrap();
    let stop = CancellationSource::new();
    let child = lease.controlled_child(stop.token(), None);
    stop.cancel();
    let cancelled = WorthQueryConditionalCommitWatchSet::retain_entities(
        &ExecutionRequest::leased(&child),
        Some(&original),
        [entity(2)].into_iter(),
        1,
        None,
    );
    assert!(
        matches!(cancelled, Err(ref cause) if matches!(cause.kind(), Kind::ReadExecution(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled))))
    );
    assert_eq!(lease.charged_memory_bytes(), 32 * 1024);
    drop(full);
    let successor = WorthQueryConditionalCommitWatchSet::retain_entities(
        &request,
        Some(&original),
        [entity(2)].into_iter(),
        1,
        None,
    )
    .unwrap();
    assert_eq!(
        successor.entities.iter().copied().collect::<Vec<_>>(),
        [entity(1), entity(2)]
    );
    assert!(lease.charged_memory_bytes() > held);
    drop(successor);
    drop(original);
    assert_eq!(lease.charged_memory_bytes(), held);
    drop(clone);
    assert_eq!(lease.charged_memory_bytes(), 0);
}
