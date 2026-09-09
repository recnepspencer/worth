use super::super::*;
use crate::data::aspect::Aspect;
use crate::data::retained_storage::{RetainedStorageMeasurement, RetainedStoragePreparation};
use crate::facade::SignalGraph;

#[test]
fn full_fork_preserves_pending_inputs_and_catalog_with_isolated_edits() {
    let mut graph = SignalGraph::new();
    let nodes: Vec<_> = (0..128).map(|_| graph.node().build()).collect();
    let mut source = DiagnosticsState::default();
    for &node in &nodes {
        source.note_change_input(node, Aspect::new(3), &[], Some("cause".repeat(1024)));
    }
    for id in 1..128 {
        source.branch_catalog.insert(
            SignalBranchId(id),
            SignalBranchHandle {
                id: SignalBranchId(id),
                name: format!("branch {id}"),
                parent_branch_id: Some(SignalBranchId(0)),
                head_snapshot_id: None,
            },
        );
    }
    let wire = serde_json::to_value(&source).unwrap();
    let mut draft = source.fork_persistent();
    assert_eq!(serde_json::to_value(&draft).unwrap(), wire);
    assert!(source.branch_catalog.ptr_eq(&draft.branch_catalog));
    let original = source.pending_input.as_ref().unwrap();
    let pending = draft.pending_input.as_mut().unwrap();
    assert!(original.changed_nodes.ptr_eq(&pending.changed_nodes));
    assert!(original.changed_aspects.ptr_eq(&pending.changed_aspects));
    assert!(Arc::ptr_eq(
        original.causality_kind.as_ref().unwrap(),
        pending.causality_kind.as_ref().unwrap()
    ));
    pending.changed_nodes.remove(&nodes[0]);
    pending.changed_aspects.insert(4);
    pending.causality_kind = Some(Arc::new("draft cause".into()));
    draft
        .branch_catalog
        .get_mut(&SignalBranchId(1))
        .unwrap()
        .head_snapshot_id = Some(SignalSnapshotId(17));
    assert_eq!(serde_json::to_value(&source).unwrap(), wire);
    assert!(std::ptr::eq(
        source.branch_catalog.get(&SignalBranchId(2)).unwrap(),
        draft.branch_catalog.get(&SignalBranchId(2)).unwrap()
    ));
    let mut restored: DiagnosticsState = serde_json::from_value(wire.clone()).unwrap();
    let restored_fork = restored.fork_persistent();
    assert_eq!(serde_json::to_value(&restored_fork).unwrap(), wire);
    assert!(restored
        .branch_catalog
        .ptr_eq(&restored_fork.branch_catalog));
    assert!(restored
        .pending_input
        .as_ref()
        .unwrap()
        .changed_nodes
        .ptr_eq(&restored_fork.pending_input.as_ref().unwrap().changed_nodes));
}

#[test]
fn carrier_catalog_charge_includes_removed_shared_base_entries() {
    let mut short = DiagnosticsState::default();
    let mut long = DiagnosticsState::default();
    let short_name = String::from("branch");
    let long_name = "retained name".repeat(1024);
    let capacity_difference = (long_name.capacity() - short_name.capacity()) as u64;
    for (state, name) in [(&mut short, short_name), (&mut long, long_name)] {
        state.branch_catalog.insert(
            SignalBranchId(1),
            SignalBranchHandle {
                id: SignalBranchId(1),
                name,
                parent_branch_id: Some(SignalBranchId(0)),
                head_snapshot_id: None,
            },
        );
    }
    let mut short_draft = short.fork_persistent();
    let mut long_draft = long.fork_persistent();
    for draft in [&mut short_draft, &mut long_draft] {
        draft.branch_catalog.remove(&SignalBranchId(1));
        assert!(!draft.branch_catalog.contains_key(&SignalBranchId(1)));
    }
    let catalog_charge = |state: &DiagnosticsState| {
        state
            .branch_catalog
            .retained_heap_charge(&mut RetainedStoragePreparation::new(1000))
            .unwrap()
            .bytes()
    };
    let carrier_charge = |state: &DiagnosticsState| {
        state
            .retained_branch_carrier_charge(&mut RetainedStoragePreparation::new(1000))
            .unwrap()
            .bytes()
    };
    assert_eq!(
        catalog_charge(&long_draft) - catalog_charge(&short_draft),
        capacity_difference
    );
    assert_eq!(
        carrier_charge(&long_draft) - carrier_charge(&short_draft),
        capacity_difference
    );
    assert!(short.branch_catalog.contains_key(&SignalBranchId(1)));
    assert!(long.branch_catalog.contains_key(&SignalBranchId(1)));
}
