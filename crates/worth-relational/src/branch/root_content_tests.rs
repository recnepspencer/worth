use crate::history::data::BranchId;
use crate::tests::support::*;

impl super::RelationalBranchRoot {
    pub(crate) fn assert_derived_inventory_matches_cold(&self) {
        let mut unavailable = 0;
        for region in self.regions.values() {
            assert_eq!(
                region.allocation_inventory,
                region.partition.allocation_inventory(),
                "partition {:?}: incremental allocation inventory",
                region.partition_id
            );
            let cold_unavailable = region.partition.entity_arena.lifecycle_counts().unavailable
                + region
                    .partition
                    .relation_arena
                    .lifecycle_counts()
                    .unavailable;
            assert_eq!(
                region.materialization_unavailable_records(),
                cold_unavailable,
                "partition {:?}: incremental unavailable count",
                region.partition_id
            );
            unavailable += cold_unavailable;
        }
        assert_eq!(
            self.regions.materialization_unavailable_records(),
            unavailable
        );
    }
}

#[test]
fn incremental_content_matches_cold_rebuild_after_each_real_graph_publication() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "source");
    assert_complete(&runtime, "first entity");
    let target = create_entity(&runtime, "target");
    assert_complete(&runtime, "second entity and empty adjacency");
    let relation = create_relation_outcome(&runtime, source, target, "edge");
    assert_complete(&runtime, "relation creation");
    delete_relation_on_branch(
        &runtime,
        changed_relations(&relation)[0],
        BranchId("main".to_owned()),
    );
    assert_complete(&runtime, "relation deletion");
}

fn assert_complete(runtime: &crate::runtime::RelationalRuntime, stage: &str) {
    let root = runtime
        .history
        .branch_cell(&BranchId("main".to_owned()))
        .and_then(|cell| cell.root())
        .unwrap();
    let symbols = runtime.services.symbols.interner_snapshot();
    assert!(
        root.is_complete(&symbols),
        "{stage}: carried commitment must match cold truth"
    );
}
