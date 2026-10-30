//! Physical metadata identity stays distinct when genuine readmission shares payloads.
use super::*;
use crate::history::data::BranchId;
use crate::tests::support::{create_entity_outcome, durable_runtime_with_test_schema};
use std::collections::BTreeMap;
use std::sync::Arc;

#[test]
fn readmitted_root_metadata_is_distinct_while_unchanged_allocations_remain_shared() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = durable_runtime_with_test_schema();
    create_entity_outcome(execution, &runtime, "root-allocation-readmission");
    let root = runtime
        .history
        .branch_cell(&BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    let readmitted = root
        .readmit_descriptor(
            root.descriptor().unwrap().clone(),
            &runtime.services.symbols.interner_snapshot(),
        )
        .unwrap();
    assert!(!Arc::ptr_eq(&root, &readmitted));
    assert_eq!(root.id(), readmitted.id(), "semantic identity is preserved");
    let original = allocations(&root);
    let restored = allocations(&readmitted);
    let metadata = RelationalRootAuthoritativeAllocationKind::RootMetadata;
    let original_metadata = original
        .iter()
        .find(|((kind, _), _)| *kind == metadata)
        .unwrap();
    let restored_metadata = restored
        .iter()
        .find(|((kind, _), _)| *kind == metadata)
        .unwrap();
    assert_ne!(original_metadata.0, restored_metadata.0);
    assert_eq!(original_metadata.1, restored_metadata.1);
    assert_eq!(original.len(), restored.len());
    for (locator, bytes) in &original {
        if locator.0 != metadata {
            assert_eq!(Some(bytes), restored.get(locator), "shared {locator:?}");
        }
    }
    for observed in [&root, &readmitted] {
        let independent = observed
            .owner_allocation_ledger_entries()
            .into_iter()
            .map(|entry| ((entry.kind, entry.owner_id), entry.authoritative_bytes))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            allocations(observed),
            independent,
            "the independent owner walk preserves physical identity"
        );
    }
}

fn allocations(
    root: &RelationalBranchRoot,
) -> BTreeMap<(RelationalRootAuthoritativeAllocationKind, u64), u64> {
    root.authoritative_allocation_observations()
        .into_iter()
        .map(|entry| ((entry.kind, entry.owner_id), entry.authoritative_bytes))
        .collect()
}
