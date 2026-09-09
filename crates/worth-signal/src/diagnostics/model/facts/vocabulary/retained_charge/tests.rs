use super::*;
use crate::data::persistent_ord_map::{PersistentOrdMap, RetainedMapMutationOutcome};
use crate::tests::explanation_retention_fixture::materialized_explanation;

#[test]
fn materialized_fact_maps_prepare_bounded_charges_and_share_them_across_forks() {
    let explanation = materialized_explanation();
    let fact = ExplanationFact::from_explanation(&explanation);
    let mut map: PersistentOrdMap<_, _> = [(fact.node, fact.clone())].into_iter().collect();
    let mut work = Work::new(10000);
    map.prepare_retained_charge(&mut work).unwrap();
    let visits = work.visits();
    let mut short: PersistentOrdMap<_, _> = [(fact.node, fact.clone())].into_iter().collect();
    let old_allocation = short.get(&fact.node).unwrap() as *const ExplanationFact;
    assert_eq!(
        short.prepare_retained_charge(&mut Work::new(visits - 1)),
        Err(Denial::WorkExhausted {
            maximum_visits: visits - 1
        })
    );
    assert_eq!(
        short.get(&fact.node).unwrap() as *const ExplanationFact,
        old_allocation
    );
    assert_eq!(short.get(&fact.node).unwrap(), &fact);
    short
        .prepare_retained_charge(&mut Work::new(visits))
        .unwrap();
    let mut fork = map.fork_persistent();
    assert!(map.ptr_eq(&fork));
    let retained_charge = map.prepared_retained_charge().unwrap();
    assert_eq!(
        fork.prepare_retained_charge(&mut Work::new(0)).unwrap(),
        retained_charge
    );
    let outcome = fork
        .edit_with_retained_charge(&fact.node, &mut Work::new(10000), |value| {
            value.state.push_str(" draft")
        })
        .unwrap();
    let RetainedMapMutationOutcome::Accounted { charge, .. } = outcome else {
        panic!("fixture admits changed fact accounting")
    };
    assert_eq!(
        charge,
        fork.retained_heap_charge(&mut Work::new(10000)).unwrap()
    );
    assert_eq!(map.get(&fact.node).unwrap(), &fact);
    assert_eq!(map.prepared_retained_charge().unwrap(), retained_charge);
    let provenance = ProvenanceFact::from_explanation(&explanation);
    let mut map: PersistentOrdMap<_, _> = [(provenance.node, provenance)].into_iter().collect();
    map.prepare_retained_charge(&mut Work::new(10000)).unwrap();
    let fork = map.fork_persistent();
    assert!(map.ptr_eq(&fork));
    assert_eq!(
        fork.prepared_retained_charge().unwrap(),
        map.prepared_retained_charge().unwrap()
    );
}

#[test]
fn fact_capacity_is_separate_from_nested_explanation_and_provenance_payloads() {
    let explanation = materialized_explanation();
    let mut fact = ExplanationFact::from_explanation(&explanation);
    let original = fact.clone();
    let before = fact.retained_heap_charge(&mut Work::new(10000)).unwrap();
    let mut delta = grow_string(&mut fact.state);
    if let Some(output) = &mut fact.output_change {
        delta += grow_string(output);
    }
    assert_eq!(
        fact.retained_heap_charge(&mut Work::new(10000))
            .unwrap()
            .bytes()
            - before.bytes(),
        delta
    );
    assert_eq!(fact, original);
    let mut provenance = ProvenanceFact::from_explanation(&explanation);
    assert!(!provenance.vertices.is_empty());
    let original = provenance.clone();
    let before = provenance
        .retained_heap_charge(&mut Work::new(10000))
        .unwrap();
    let mut delta = grow_vec(&mut provenance.vertices)
        + grow_vec(&mut provenance.edges)
        + grow_vec(&mut provenance.causal_links);
    for vertex in &mut provenance.vertices {
        if let Some(state) = &mut vertex.state {
            delta += grow_string(state);
        }
    }
    assert_eq!(
        provenance
            .retained_heap_charge(&mut Work::new(10000))
            .unwrap()
            .bytes()
            - before.bytes(),
        delta
    );
    assert_eq!(provenance, original);
}
fn grow_string(value: &mut String) -> u64 {
    let old = value.capacity();
    value.reserve_exact(1024);
    (value.capacity() - old) as u64
}
fn grow_vec<T>(value: &mut Vec<T>) -> u64 {
    let old = value.capacity();
    value.reserve_exact(64);
    ((value.capacity() - old) * std::mem::size_of::<T>()) as u64
}

#[test]
fn provenance_nested_edges_and_rewiring_contribute_payload_and_vector_capacity() {
    use crate::data::output::PartitionSubscription;
    use crate::logic::explain::{RewiringDependency, RewiringSummary};
    use crate::tests::support::ASPECT_A;
    let explanation = materialized_explanation();
    let mut fact = ProvenanceFact::from_explanation(&explanation);
    // Explicit representation extension; this is not executed topology evidence.
    fact.edges.clear();
    fact.rewiring = None;
    let before = fact.retained_heap_charge(&mut Work::new(10000)).unwrap();
    let previous_capacity = fact.edges.capacity();
    let partition = String::with_capacity(512);
    let detail = String::with_capacity(256);
    let comparator = String::with_capacity(1024);
    let reason = String::with_capacity(2048);
    let edge_bytes =
        (partition.capacity() + detail.capacity() + comparator.capacity() + reason.capacity())
            as u64;
    let edge = ProvenanceEdge {
        kind: super::super::ProvenanceEdgeKind::Changed,
        source: explanation.node,
        aspect: ASPECT_A,
        subscription: Some(PartitionSubscription::partition_and_detail(
            partition, detail,
        )),
        cached_version: Some(0),
        current_version: Some(1),
        comparator: Some(comparator),
        reason: Some(reason),
    };
    assert_eq!(
        edge.retained_heap_charge(&mut Work::new(100))
            .unwrap()
            .bytes(),
        edge_bytes
    );
    fact.edges.push(edge);
    let edge_growth = ((fact.edges.capacity() - previous_capacity)
        * std::mem::size_of::<ProvenanceEdge>()) as u64;
    let mut added = Vec::with_capacity(16);
    let mut removed = Vec::with_capacity(32);
    let mut scope_bytes = 0;
    for destination in [&mut added, &mut removed] {
        let partition = String::with_capacity(4096);
        let detail = String::with_capacity(8192);
        scope_bytes += partition.capacity() + detail.capacity();
        destination.push(RewiringDependency {
            source: explanation.node,
            aspect: ASPECT_A,
            subscription: Some(PartitionSubscription::partition_and_detail(
                partition, detail,
            )),
        });
    }
    let rewiring_bytes = ((added.capacity() + removed.capacity())
        * std::mem::size_of::<RewiringDependency>()
        + scope_bytes) as u64;
    let rewiring = RewiringSummary { added, removed };
    assert_eq!(
        rewiring
            .retained_heap_charge(&mut Work::new(100))
            .unwrap()
            .bytes(),
        rewiring_bytes
    );
    fact.rewiring = Some(rewiring);
    assert_eq!(
        fact.retained_heap_charge(&mut Work::new(10000))
            .unwrap()
            .bytes()
            - before.bytes(),
        edge_growth + edge_bytes + rewiring_bytes
    );
}
