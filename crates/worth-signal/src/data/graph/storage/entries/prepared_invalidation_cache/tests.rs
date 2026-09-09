use super::*;
use crate::data::proof::invalidation::binding::{DependencyRevision, OutputCommitOrdinal};
use crate::data::proof::PartitionScopeSet;
use crate::data::retained_storage::RetainedStoragePreparation as Work;

#[test]
fn prepared_cache_matches_owner_rebuild_and_denies_short_work() {
    let payload = "cache-scope".repeat(1000);
    let whole = PartitionSubscription::whole_partition("a");
    let detail = PartitionSubscription::partition_and_detail("a", "detail");
    let cases = vec![
        vec![],
        vec![(1, vec![])],
        vec![(1, vec![whole.clone()]), (1, vec![whole.clone()])],
        vec![(1, vec![whole.clone(), detail.clone()]), (2, vec![detail])],
        vec![(
            1,
            vec![
                PartitionSubscription::whole_partition("z"),
                PartitionSubscription::whole_partition(payload.as_str()),
                whole,
            ],
        )],
    ];
    for specifications in cases {
        let mut graph = SignalGraph::new();
        let node = graph.create_node();
        graph.transition_node_clean(node).unwrap();
        assert!(graph
            .node_direct_invalidation_basis(node)
            .unwrap()
            .is_none());
        let causes: Vec<_> = specifications
            .into_iter()
            .map(|(aspect, scopes)| {
                let producer = graph.create_node();
                ResolvedDependencyCause::new(
                    graph.runtime_instance_id(),
                    node,
                    DependencyRevision(0),
                    producer,
                    Aspect::new(aspect),
                    None,
                    0,
                    OutputCommitOrdinal(1),
                    1,
                    PartitionScopeSet::new(scopes),
                )
            })
            .collect();
        // Storage derivation fixture, not proof of performed publication.
        let id = graph.cause_sets.insert(causes.clone());
        graph.set_node_pending_cause_set_id(node, id).unwrap();
        graph
            .rebuild_dirty_caches_from_pending_causes(node)
            .unwrap();
        let expected = graph
            .node_dirty_partition_scope_payload(node)
            .unwrap()
            .to_vec();
        let dirty = graph.hot_ref(node).unwrap().dirty_aspects;
        let scoped = graph.hot_ref(node).unwrap().dirty_partition_scope_aspects;
        let mut measured = Work::new(usize::MAX);
        let cache = PreparedInvalidationCache::from_causes(
            &causes,
            &mut EvaluationWork::Conditional(&mut measured),
        )
        .unwrap();
        assert_eq!(cache.scopes.as_slice(), expected.as_slice());
        assert_eq!(cache.dirty, dirty);
        assert_eq!(cache.scoped, scoped);
        let cost = measured.visits();
        let mut exact = Work::new(cost + 7);
        exact.reserve_visits(7).unwrap();
        PreparedInvalidationCache::from_causes(
            &causes,
            &mut EvaluationWork::Conditional(&mut exact),
        )
        .unwrap();
        assert_eq!(exact.visits(), cost + 7);
        if cost > 0 {
            let mut short = Work::new(cost - 1);
            assert!(PreparedInvalidationCache::from_causes(
                &causes,
                &mut EvaluationWork::Conditional(&mut short)
            )
            .is_err());
        }
        graph
            .install_prepared_invalidation_cache(node, cache)
            .unwrap();
        assert_eq!(
            graph.node_dirty_partition_scope_payload(node).unwrap(),
            expected.as_slice()
        );
        assert_eq!(graph.hot_ref(node).unwrap().dirty_aspects, dirty);
        assert_eq!(
            graph.hot_ref(node).unwrap().dirty_partition_scope_aspects,
            scoped
        );
    }
}
