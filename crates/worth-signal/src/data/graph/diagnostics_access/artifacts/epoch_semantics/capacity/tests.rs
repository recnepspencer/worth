//! Operational version storage is not a semantic projection source.
use super::*;
use crate::data::aspect::{Aspect, AspectVersion, PartitionVersionOverrides};
use crate::data::output::ChangedRegion;
use std::sync::Arc;

#[test]
fn semantic_projection_bound_ignores_operational_version_storage() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let baseline = graph
        .epoch_semantic_diagnostic_fixed_bound(node, 0, 0, &mut Work::new(None))
        .unwrap();
    let mut previous_heap = 0;
    for count in [32, 512] {
        let mut versions = PartitionVersionOverrides::default();
        let regions: Vec<_> = (0..count)
            .map(|index| ChangedRegion::new(format!("operational-{index}")))
            .collect();
        let version = AspectVersion::zero().with(Aspect::new(0), 7);
        versions.apply_evaluation(version, &regions);
        let heap = versions
            .retained_heap_charge(&mut Work::new(None))
            .unwrap()
            .bytes();
        assert!(heap > previous_heap);
        previous_heap = heap;
        // Keep the real node's version header consistent with its scoped
        // operational storage. No runtime artifact or cold projection changes.
        let index = node.index() as usize;
        graph.arena.warm[index].aspect_version_overrides = Arc::new(versions);
        graph.arena.hot[index]
            .as_mut()
            .unwrap()
            .aspect_version_header
            .set_global(version);
        graph.arena.hot[index]
            .as_mut()
            .unwrap()
            .aspect_version_header
            .set_has_partition_overrides(true);
        let bound = graph
            .epoch_semantic_diagnostic_fixed_bound(node, 0, 0, &mut Work::new(None))
            .unwrap();
        assert_eq!(
            bound, baseline,
            "semantic admission must be independent of operational version population"
        );
    }
}
