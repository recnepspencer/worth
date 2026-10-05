//! The prepared cache owns only the scopes it actually contains.
use super::measure;
use crate::data::aspect::Aspect;
use crate::data::graph::storage::PreparedInvalidationCache;
use crate::data::graph::SignalGraph;
use crate::data::output::PartitionSubscription;
use crate::data::proof::invalidation::binding::{
    DependencyRevision, OutputCommitOrdinal, ResolvedDependencyCause,
};
use crate::data::proof::PartitionScopeSet;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement, RetainedStoragePreparation as Work,
};
use crate::logic::evaluation::EvaluationWork;

#[test]
fn prepared_cache_allocates_no_scope_buffer_for_unscoped_causes() {
    let mut graph = SignalGraph::new();
    let consumer = graph.create_node();
    let producer = graph.create_node();
    let cause = ResolvedDependencyCause::new(
        graph.runtime_instance_id(),
        consumer,
        DependencyRevision(0),
        producer,
        Aspect::new(1),
        None,
        0,
        OutputCommitOrdinal(1),
        1,
        PartitionScopeSet::default(),
    );
    let (_, peak) = measure(|| {
        PreparedInvalidationCache::from_causes(
            &[cause],
            &mut EvaluationWork::Conditional(&mut Work::new(usize::MAX)),
        )
        .unwrap()
    });
    assert_eq!(peak, Some(0));
}

#[test]
fn prepared_cache_scoped_allocation_fits_owned_vec_and_path() {
    let mut graph = SignalGraph::new();
    let consumer = graph.create_node();
    let producer = graph.create_node();
    let scope = PartitionSubscription::partition_and_detail("books", "risk");
    let cause = ResolvedDependencyCause::new(
        graph.runtime_instance_id(),
        consumer,
        DependencyRevision(0),
        producer,
        Aspect::new(1),
        None,
        0,
        OutputCommitOrdinal(1),
        1,
        PartitionScopeSet::new([scope.clone()]),
    );
    let path = scope
        .retained_heap_charge(&mut Work::new(usize::MAX))
        .unwrap();
    let bound = Charge::capacity::<(Aspect, PartitionSubscription)>(1)
        .unwrap()
        .checked_add(path)
        .unwrap()
        .bytes();
    let (cache, peak) = measure(|| {
        PreparedInvalidationCache::from_causes(
            &[cause],
            &mut EvaluationWork::Conditional(&mut Work::new(usize::MAX)),
        )
        .unwrap()
    });
    assert!(peak.expect("tracked scoped cache allocation") as u64 <= bound);
    graph
        .install_prepared_invalidation_cache(consumer, cache)
        .unwrap();
    assert_eq!(
        graph
            .node_dirty_partition_scope_payload(consumer)
            .unwrap()
            .len(),
        1
    );
}
