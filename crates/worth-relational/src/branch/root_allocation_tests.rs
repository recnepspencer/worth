//! Physical allocation identity, independently of operational root admission.
use super::*;
use std::collections::BTreeMap;
use std::sync::Arc;

#[test]
fn distinct_root_metadata_allocations_preserve_shared_immutable_owners() {
    let root = RelationalBranchRoot::empty();
    // No runtime receives these values. This tests allocation identity, while
    // the public checkpoint court exercises genuine operational readmission.
    let readmitted = Arc::new((*root).clone());
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
