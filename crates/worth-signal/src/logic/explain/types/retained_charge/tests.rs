use super::*;
use crate::data::node::EvaluationCondition;
use crate::data::output::PartitionSubscription;
use crate::logic::explain::types::{ConditionDecision, MeaningfulChangeReason, UpstreamCause};
use crate::tests::explanation_retention_fixture::materialized_explanation;
use crate::tests::support::ASPECT_A;

#[test]
fn real_explanation_retention_charges_vector_capacity_and_cold_artifact_labels() {
    let mut explanation = materialized_explanation();
    let original = explanation.clone();
    let before = explanation
        .retained_heap_charge(&mut Work::new(10000))
        .unwrap();
    let mut delta = grow_vec(&mut explanation.upstream)
        + grow_vec(&mut explanation.causal_links)
        + grow_vec(&mut explanation.changed_regions);
    let retained = explanation
        .historical_artifact_record
        .as_mut()
        .unwrap()
        .retained
        .as_mut()
        .unwrap();
    assert!(!retained.labels.is_empty());
    delta += grow_vec(&mut retained.labels);
    for label in &mut retained.labels {
        delta += grow_string(label);
    }
    assert_eq!(
        explanation
            .retained_heap_charge(&mut Work::new(10000))
            .unwrap()
            .bytes()
            - before.bytes(),
        delta
    );
    assert_eq!(explanation, original);
}

#[test]
fn upstream_conditions_comparators_and_both_scope_copies_have_independent_cost() {
    let node = materialized_explanation().node;
    let mut key = String::with_capacity(1024);
    key.push_str("condition");
    let key_bytes = key.capacity();
    let mut partition = String::with_capacity(512);
    partition.push_str("partition");
    let mut detail = String::with_capacity(256);
    detail.push_str("detail");
    let scope_bytes = partition.capacity() + detail.capacity();
    let subscription = PartitionSubscription::partition_and_detail(partition, detail);
    let cause = UpstreamCause::ConditionDeferred {
        source: node,
        aspect: ASPECT_A,
        subscription: Some(subscription),
        cached_version: 0,
        current_version: 1,
        condition: EvaluationCondition::Custom(key),
        decision: ConditionDecision::Deferred,
    };
    assert_eq!(
        cause
            .retained_heap_charge(&mut Work::new(100))
            .unwrap()
            .bytes(),
        (key_bytes + scope_bytes) as u64
    );
    let mut comparator = String::with_capacity(2048);
    comparator.push_str("comparator");
    let mut reason = String::with_capacity(4096);
    reason.push_str("reason");
    let expected = comparator.capacity() + reason.capacity();
    let cause = UpstreamCause::Changed {
        source: node,
        aspect: ASPECT_A,
        subscription: None,
        cached_version: 0,
        current_version: 1,
        comparator: crate::data::comparator::VersionComparatorPolicy::Custom { key: comparator },
        reason: MeaningfulChangeReason::CustomComparator { key: reason },
    };
    assert_eq!(
        cause
            .retained_heap_charge(&mut Work::new(100))
            .unwrap()
            .bytes(),
        expected as u64
    );
    let mut scope = ScopeProvenance {
        source_scope: Some(PartitionSubscription::partition_and_detail(
            "partition",
            "detail",
        )),
        validation_scope: Some(PartitionSubscription::partition_and_detail(
            "partition",
            "detail",
        )),
        kind: super::super::ScopeProvenanceKind::Direct,
        note: Some("note".into()),
    };
    let original = scope.clone();
    let before = scope.retained_heap_charge(&mut Work::new(100)).unwrap();
    let mut delta = grow_string(scope.note.as_mut().unwrap());
    for subscription in [&mut scope.source_scope, &mut scope.validation_scope]
        .into_iter()
        .flatten()
    {
        delta += grow_string(&mut subscription.partition.0)
            + grow_string(subscription.detail.as_mut().unwrap());
    }
    assert_eq!(
        scope
            .retained_heap_charge(&mut Work::new(100))
            .unwrap()
            .bytes()
            - before.bytes(),
        delta
    );
    assert_eq!(scope, original);
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
