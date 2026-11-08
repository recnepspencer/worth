//! Actual fixed-page/COW admission; no runtime or copied protection marker.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn host(budget: &SerialMemoryBudget) -> SerialRequest {
    SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None)
}
fn arc_bytes<T>() -> u64 {
    crate::storage::substrate::StorageAllocation::<T>::layout_bytes()
}

fn page_bytes() -> u64 {
    arc_bytes::<ColumnNode<u64>>()
        + std::alloc::Layout::array::<Option<u64>>(PAGE_LEN)
            .unwrap()
            .size() as u64
}
fn controller_bytes() -> u64 {
    let budget = SerialMemoryBudget::new(Some(65536));
    let host = host(&budget);
    let mut observed = None;
    ExecutionRequest::serial(&host)
        .run_native_with_checkpoint(|_| observed = Some(budget.charged_memory_bytes()))
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    observed.unwrap()
}
#[test]
fn unique_fixed_page_edit_needs_no_replacement_but_shared_page_is_prefunded() {
    let mut unique: SharedColumn<u64> = vec![1, 2].into();
    let zero = SerialMemoryBudget::new(Some(0));
    let zero_host = host(&zero);
    unique
        .set_for_request(0, 3, &ExecutionRequest::serial(&zero_host))
        .unwrap();
    assert_eq!(unique[0], 3);
    assert_eq!(zero.charged_memory_bytes(), 0);
    let original = unique.clone();
    let bill = page_bytes();
    let controller = controller_bytes();
    assert!(bill > 0);
    assert!(controller > 0);
    let denied = SerialMemoryBudget::new(Some(controller + bill - 1));
    let denied_host = host(&denied);
    assert!(
        matches!(unique.set_for_request(0,4,&ExecutionRequest::serial(&denied_host)),Err(Cause::PolicyMemoryExhausted{requested,..}) if requested==bill)
    );
    assert_eq!(unique[0], 3);
    assert_eq!(original[0], 3);
    assert_eq!(denied.charged_memory_bytes(), 0);
    let accepted = SerialMemoryBudget::new(Some(controller + bill));
    let accepted_host = host(&accepted);
    unique
        .set_for_request(0, 4, &ExecutionRequest::serial(&accepted_host))
        .unwrap();
    assert_eq!(accepted.charged_memory_bytes(), bill);
    assert_eq!(unique[0], 4);
    assert_eq!(original[0], 3);
    drop(unique);
    assert_eq!(accepted.charged_memory_bytes(), 0);
}
#[test]
fn dense_levels_refund_before_surviving_node_backing_and_final_owner() {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = host(&budget);
    let column = SharedColumn::<u64>::from_dense_values_for_request(
        (0..65).collect(),
        &ExecutionRequest::serial(&host),
    )
    .unwrap();
    // Three fixed pages, then two branches, then one root. No level Vec survives.
    let retained = 3 * page_bytes() + 3 * arc_bytes::<ColumnNode<u64>>();
    assert_eq!(budget.charged_memory_bytes(), retained);
    assert_eq!(column.to_vec(), (0..65).collect::<Vec<_>>());
    let alias = column.clone();
    drop(column);
    assert_eq!(budget.charged_memory_bytes(), retained);
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
#[test]
fn original_stop_preserves_published_column_and_typed_cause() {
    let original: SharedColumn<u64> = (0..65).collect();
    let mut candidate = original.clone();
    let budget = SerialMemoryBudget::new(Some(65536));
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    cancellation.cancel();
    assert!(matches!(
        candidate.set_for_request(32, 99, &ExecutionRequest::serial(&host)),
        Err(Cause::Cancelled)
    ));
    assert_eq!(original[32], 32);
    assert_eq!(candidate[32], 32);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn partial_cow_retains_admitted_branch_until_failed_workspace_drop() {
    let original: SharedColumn<u64> = (0..64).collect();
    let mut candidate = original.clone();
    let branch = arc_bytes::<ColumnNode<u64>>();
    let leaf = page_bytes();
    let budget = SerialMemoryBudget::new(Some(controller_bytes() + branch + leaf - 1));
    let host = host(&budget);
    assert!(
        matches!(candidate.set_for_request(16,99,&ExecutionRequest::serial(&host)),Err(Cause::PolicyMemoryExhausted{requested,..}) if requested==leaf)
    );
    // The actual distinct root proves admission passed the controller and the
    // parent COW allocation; only the selected leaf was refused.
    assert_ne!(
        candidate.root.as_ref().unwrap().id(),
        original.root.as_ref().unwrap().id()
    );
    assert_eq!(original[16], 16);
    assert_eq!(candidate[16], 16);
    assert_eq!(budget.charged_memory_bytes(), branch);
    drop(candidate);
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(original[16], 16);
}
