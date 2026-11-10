//! AVL node bills are separate from inline scalar values.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
#[test]
fn map_cow_owns_only_actual_nodes_and_unique_replacement_refunds_nothing_early() {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut current = SharedMap::<u64, u64>::new();
    for key in 0..128 {
        current.insert_for_request(key, key + 1, &request).unwrap();
    }
    let bill = Arc::<MapNode<u64, u64>>::layout_bytes();
    assert_eq!(budget.charged_memory_bytes(), 128 * bill);
    let retained = current.clone();
    current.insert_for_request(64, 999, &request).unwrap();
    assert_eq!(retained.get(&64), Some(&65));
    assert_eq!(current.get(&64), Some(&999));
    assert!(budget.charged_memory_bytes() > 128 * bill);
    drop(retained);
    assert_eq!(budget.charged_memory_bytes(), 128 * bill);
    let zero = SerialMemoryBudget::new(Some(0));
    let zero_host =
        SerialRequest::from_memory(zero.clone(), CancellationSource::new().token(), None);
    current
        .insert_for_request(64, 1000, &ExecutionRequest::serial(&zero_host))
        .unwrap();
    assert_eq!(zero.charged_memory_bytes(), 0);
    assert_eq!(budget.charged_memory_bytes(), 128 * bill);
    drop(current);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
