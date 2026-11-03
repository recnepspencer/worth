//! An owned raw tree's payer survives handoff and cancelled final disposal.
use crate::domain_computation::primary_graph::application_query::projection::projected_tree::content_comparison::tests::shape::*;
use worth_query_execution::domain_computation::primary_graph::application_query::projection::projected_tree::owned_rows::OwnedProjectionRows;
use worth_query_execution::domain_computation::primary_graph::application_query::projection::projected_tree::owned_rows::ProjectionDisposal;
use worth_query_execution::domain_computation::primary_graph::source_facts::allocation::SourceFactsAllocation;
use worth_execution::ChargedBytes;
use worth_execution::ExecutionRequestArrayBuilder;
#[test]
fn frozen_deep_rows_keep_original_funding_through_take_retain_clear_and_last_drop() {
    crate::domain_computation::primary_graph::application_query::managed_index::invocation::tests::with_original_content_scope(|execution, memory, cancel| {
    let allocation = SourceFactsAllocation::open(execution).unwrap();
    let depth = 128;
    let disposal = ProjectionDisposal::admitted(execution, &allocation, depth + 1).unwrap();
    let key = slot("owned");
    let mut root = shape(&key);
    for _ in 1..depth {
        let mut parent = shape(&key);
        parent.relations[0].rows[0] = root;
        root = parent;
    }
    let rows = vec![root];
    let payload = rows.capacity() * size_of::<WorthQueryApplicationProjectionNode>()
        + rows[0].retained_bytes();
    allocation.claim(execution, payload as u64).unwrap();
    allocation
        .claim_arc::<worth_query_execution::domain_computation::primary_graph::application_query::projection::retained_projection::body::OriginalProjection>(
            execution,
        )
        .unwrap();
    let disclosure = ExecutionRequestArrayBuilder::allocate(*execution, 0)
        .unwrap()
        .seal()
        .unwrap();
    let mut tree = WorthQueryApplicationDisclosedProjectionTree::from_managed(
        OwnedProjectionRows::managed(rows, disposal, payload, Arc::clone(&allocation)),
    );
    drop(allocation);
    assert_eq!(tree.additional_charged_bytes(), payload as u64);
    let mut taken = tree.take();
    tree.clear();
    let original = taken.retain_projection(disclosure, None).unwrap();
    let last_share = Arc::clone(&original);
    taken.clear();
    drop(original);
    assert!(memory.charged_memory_bytes() >= payload as u64);
    cancel.cancel();
    assert!(execution.consult().is_err());
    drop(last_share);
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "no allocation or live request needed for funded final disposal"
    );
    });
}
