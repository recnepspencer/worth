use super::*;
use crate::{
    tests::{accounted_request, authority, TEST_LOCK},
    CancellationSource, CancellationToken, ExecutionWorkCeiling, PartitionItemId,
    SerialMemoryBudget, SerialRequest,
};
use std::collections::BTreeSet;
use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};

fn journey(request: &ExecutionRequest<'_, '_>) -> ComponentPartitioner {
    let mut graph = ComponentPartitioner::new();
    for item in 1..=4 {
        graph
            .edit_with_request(request, ComponentEdit::UpsertItem(PartitionItemId(item)))
            .unwrap();
    }
    for (a, b) in [(1, 2), (2, 3)] {
        graph
            .edit_with_request(
                request,
                ComponentEdit::AddEdge(PartitionItemId(a), PartitionItemId(b)),
            )
            .unwrap();
    }
    let route = graph.route(PartitionItemId(1)).unwrap();
    assert_eq!(
        graph.members(route),
        Some(&BTreeSet::from([
            PartitionItemId(1),
            PartitionItemId(2),
            PartitionItemId(3)
        ]))
    );
    graph
        .edit_with_request(
            request,
            ComponentEdit::RemoveEdge(PartitionItemId(2), PartitionItemId(3)),
        )
        .unwrap();
    graph
        .edit_with_request(request, ComponentEdit::RemoveItem(PartitionItemId(1)))
        .unwrap();
    assert_eq!(graph.route(PartitionItemId(1)), None);
    for item in 2..=4 {
        let item = PartitionItemId(item);
        assert_eq!(
            graph.members(graph.route(item).unwrap()),
            Some(&BTreeSet::from([item]))
        );
    }
    graph
}

#[test]
fn leased_component_edits_hold_and_release_the_original_charge() {
    let _lock = TEST_LOCK.lock().unwrap();
    let lease = authority()
        .request_lease(accounted_request(1, 100_000))
        .unwrap();
    let baseline = lease.charged_memory_bytes();
    let graph = journey(&ExecutionRequest::leased(&lease));
    assert!(lease.charged_memory_bytes() > baseline);
    drop(graph);
    assert_eq!(lease.charged_memory_bytes(), baseline);
}

#[test]
fn serial_component_edits_reuse_the_original_budget_until_last_drop() {
    let _lock = TEST_LOCK.lock().unwrap();
    let budget = SerialMemoryBudget::new(Some(100_000));
    let serial = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
    let graph = journey(&ExecutionRequest::serial(&serial));
    assert!(budget.charged_memory_bytes() > 0);
    drop(graph);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn unaccounted_component_edits_keep_native_controls_without_minting_accounting() {
    let _lock = TEST_LOCK.lock().unwrap();
    let cancellation = CancellationSource::new();
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::new(1).unwrap(),
    );
    let serial = SerialRequest::from_policy(&policy, cancellation.token(), None);
    assert!(serial.memory().is_none());
    let request = ExecutionRequest::serial(&serial);
    let (mut graph, _) = request
        .run_native_with_checkpoint(|context| {
            assert!(!context.has_accounting());
            let graph = journey(&request);
            assert!(!context.has_accounting());
            graph
        })
        .unwrap();
    cancellation.cancel();
    assert_eq!(
        graph.edit_with_request(&request, ComponentEdit::RemoveItem(PartitionItemId(2))),
        Err(PartitionUpdateDenial::Stop(MapKernelStop::Cancelled))
    );
    assert!(graph.route(PartitionItemId(2)).is_some());
}

#[test]
fn foreign_or_unaccounted_requests_cannot_take_over_serial_backing() {
    let _lock = TEST_LOCK.lock().unwrap();
    let budget = SerialMemoryBudget::new(Some(100_000));
    let serial = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&serial);
    let mut graph = journey(&request);
    let held = budget.charged_memory_bytes();
    let foreign_budget = SerialMemoryBudget::new(Some(100_000));
    let foreign =
        SerialRequest::from_memory(foreign_budget.clone(), CancellationToken::new(), None);
    let native = SerialRequest::from_policy(
        &ExecutionRequestPolicy::for_workers(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            std::num::NonZeroUsize::new(1).unwrap(),
        ),
        CancellationToken::new(),
        None,
    );
    for other in [&foreign, &native] {
        assert_eq!(
            graph.edit_with_request(
                &ExecutionRequest::serial(other),
                ComponentEdit::UpsertItem(PartitionItemId(99))
            ),
            Err(PartitionUpdateDenial::Admission(
                LeaseDenial::UnrelatedNestedLease
            ))
        );
        assert_eq!(budget.charged_memory_bytes(), held);
        assert_eq!(foreign_budget.charged_memory_bytes(), 0);
        assert_eq!(graph.route(PartitionItemId(99)), None);
    }
    graph
        .edit_with_request(&request, ComponentEdit::UpsertItem(PartitionItemId(99)))
        .unwrap();
    drop(graph);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn memory_and_work_refusals_preserve_the_component_before_commit() {
    let _lock = TEST_LOCK.lock().unwrap();
    const LIMIT: u64 = 100_000;
    let budget = SerialMemoryBudget::new(Some(LIMIT));
    let serial = SerialRequest::from_memory(budget.clone(), CancellationToken::new(), None);
    let request = ExecutionRequest::serial(&serial);
    let mut graph = journey(&request);
    let held = budget.charged_memory_bytes();
    // Observe the actual two original request frames, then leave them room
    // but refuse retained graph growth at its own admission boundary.
    let framed = request
        .in_scope(|_| request.run_native_with_checkpoint(|_| budget.charged_memory_bytes()))
        .unwrap()
        .unwrap()
        .0;
    let blocker = budget.reserve(LIMIT - framed - 1).unwrap();
    assert!(matches!(
        graph.edit_with_request(&request, ComponentEdit::UpsertItem(PartitionItemId(99))),
        Err(PartitionUpdateDenial::Admission(
            LeaseDenial::MemoryExhausted(_)
        ))
    ));
    assert_eq!(graph.route(PartitionItemId(99)), None);
    drop(blocker);
    assert_eq!(budget.charged_memory_bytes(), held);
    let (refused, _) = ExecutionWorkCeiling::new(Some(0))
        .run_serial(&serial, || {
            graph.edit_with_request(&request, ComponentEdit::RemoveItem(PartitionItemId(2)))
        })
        .unwrap();
    assert_eq!(
        refused,
        Err(PartitionUpdateDenial::Stop(MapKernelStop::WorkCeiling))
    );
    assert!(graph.route(PartitionItemId(2)).is_some());
    drop(graph);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
