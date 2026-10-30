//! Paired adjacency preserves the cold owner's actual storage refusal.
use super::*;
use crate::runtime::{RelationalAdjacencyDirection, RelationalAdjacencySupportDenial};

#[test]
fn paired_adjacency_propagates_current_head_storage_failure_without_history_fallback() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = CurrentFixture::reopened(execution);
    fixture.source.failures.lock().unwrap().extend(
        fixture
            .heads
            .iter()
            .map(|member| (ColdRead::Member(*member), Cause::SourceUnavailable)),
    );
    let snapshot = snapshot_for_owner_branch(
        &fixture.runtime,
        &crate::history::data::BranchId("main".into()),
    );
    let view = fixture
        .runtime
        .read_truth()
        .project_snapshot(&snapshot)
        .unwrap();
    assert!(
        matches!(view.compare_adjacency_support(execution, &view, fixture.entity,
        KindId(2), RelationalAdjacencyDirection::Outgoing),
        Err(RelationalAdjacencySupportDenial::Execution(crate::execution::RelationalExecutionDenialCause::StorageUnavailable(failure)))
        if failure.cause == Cause::SourceUnavailable
            && failure.subject == crate::storage::data::RelationalStorageSubject::Region(fixture.entity.partition_id))
    );
    assert!(!fixture
        .source
        .member_reads
        .lock()
        .unwrap()
        .iter()
        .any(|member| fixture.history.contains(member)));
    fixture
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
}
