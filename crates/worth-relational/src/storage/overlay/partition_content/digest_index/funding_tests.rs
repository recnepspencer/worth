//! Actual private Patricia allocations, including partial failure and leaf escape.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn controller_memory() -> u64 {
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut controller = 0;
    ExecutionRequest::serial(&host)
        .run_native_with_checkpoint(|_| {
            controller = budget.charged_memory_bytes();
        })
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    controller
}

#[test]
fn controlled_patricia_refuses_exact_leaf_and_late_parent_and_refunds_children() {
    let bill = Arc::<Node>::layout_bytes();
    let controller = controller_memory();
    for (entries, nodes) in [
        (&[(0u128, [1u8; 32])][..], 1u64),
        (&[(0u128, [1u8; 32]), (1, [2; 32]), (2, [3; 32])][..], 5u64),
    ] {
        let budget = SerialMemoryBudget::new(Some(controller + nodes * bill - 1));
        let host =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let error = DigestIndex::from_sorted_entries_for_request(
            entries,
            Some(&ExecutionRequest::serial(&host)),
        )
        .unwrap_err();
        match error {
            PartitionContentDigestError::Request(
                crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                    requested,
                    admitted,
                    ancestor: 0,
                },
            ) => {
                assert_eq!(requested, bill);
                assert_eq!(admitted, bill - 1);
            }
            other => panic!("the actual entered leaf/parent must refuse: {other:?}"),
        }
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
}

#[test]
fn escaped_patricia_leaf_keeps_only_its_original_allocation_grant() {
    let entries = [(0u128, [1u8; 32]), (1, [2; 32]), (2, [3; 32])];
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let tree = DigestIndex::from_sorted_entries_for_request(
        &entries,
        Some(&ExecutionRequest::serial(&host)),
    )
    .unwrap();
    assert_eq!(
        tree.digest(),
        DigestIndex::from_sorted_entries(&entries).digest()
    );
    let original = tree.root.as_ref().unwrap();
    assert_eq!(original.nodes, 5);
    let leaf = original.children.as_ref().unwrap()[1].clone();
    assert!(leaf.children.is_none());
    assert_eq!(
        budget.charged_memory_bytes(),
        5 * Arc::<Node>::layout_bytes()
    );
    drop(tree);
    assert_eq!(leaf.key, 2);
    assert_eq!(budget.charged_memory_bytes(), Arc::<Node>::layout_bytes());
    drop(leaf);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
