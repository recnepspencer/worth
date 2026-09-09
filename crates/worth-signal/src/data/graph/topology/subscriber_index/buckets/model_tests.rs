use std::collections::BTreeMap;

use super::{IndexedSubscriptionMembership, ReverseSubscriptionIndex};
use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::output::{
    DetailTokenId, InternedPartitionSubscription, PartitionMatchMode, PartitionTokenId,
};

#[test]
fn inherited_multi_membership_readmission_does_not_resurrect_retired_scope() {
    let producer = NodeId::new(0, 0);
    let aspect = Aspect::new(1);
    let consumer = NodeId::new(1, 0);
    let unrelated = NodeId::new(2, 0);
    let retained = whole(1);
    let retired = detail(2, 9);
    let mut source = ReverseSubscriptionIndex::default();
    source.replace_consumer(
        consumer,
        memberships(producer, aspect, &[Some(retained), Some(retired)]),
    );
    source.replace_consumer(unrelated, memberships(producer, aspect, &[None]));

    let mut fork = source.fork_persistent();
    assert!(source.shares_storage_with(&fork));
    fork.replace_consumer(consumer, Vec::new());
    fork.replace_consumer(consumer, memberships(producer, aspect, &[Some(retained)]));

    assert!(fork
        .query_scope(
            producer,
            aspect,
            retained,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary
        )
        .unwrap()
        .candidates
        .contains(&consumer));
    assert!(!fork
        .query_scope(
            producer,
            aspect,
            retired,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary
        )
        .unwrap()
        .candidates
        .contains(&consumer));
    assert!(fork
        .query_whole_aspect(
            producer,
            aspect,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary
        )
        .unwrap()
        .candidates
        .contains(&unrelated));
    assert!(source
        .query_scope(
            producer,
            aspect,
            retained,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary
        )
        .unwrap()
        .candidates
        .contains(&consumer));
    assert!(source
        .query_scope(
            producer,
            aspect,
            retired,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary
        )
        .unwrap()
        .candidates
        .contains(&consumer));
    assert_eq!(
        fork.operational_clone()
            .query_scope(
                producer,
                aspect,
                retired,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary
            )
            .unwrap(),
        fork.query_scope(
            producer,
            aspect,
            retired,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary
        )
        .unwrap()
    );
}

#[test]
fn forked_replace_sequences_match_independent_scope_model() {
    let producer = NodeId::new(0, 0);
    let aspect = Aspect::new(1);
    let scopes = [None, Some(whole(1)), Some(detail(1, 7)), Some(detail(2, 8))];
    let mut source = ReverseSubscriptionIndex::default();
    let mut source_model = BTreeMap::new();
    for ordinal in 1..=16_u32 {
        let consumer = NodeId::new(ordinal, 0);
        let selected = vec![scopes[ordinal as usize % scopes.len()]];
        source.replace_consumer(consumer, memberships(producer, aspect, &selected));
        source_model.insert(consumer, selected);
    }
    let mut fork = source.fork_persistent();
    let mut model = source_model.clone();

    let mut random = 17_u64;
    for step in 0..512 {
        random = random
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let consumer = NodeId::new((random % 16 + 1) as u32, 0);
        let selected = match random.rotate_left(19) % 5 {
            0 => Vec::new(),
            1 => vec![None],
            2 => vec![Some(whole(1))],
            3 => vec![Some(detail(1, 7))],
            _ => vec![Some(whole(1)), Some(detail(2, 8))],
        };
        fork.replace_consumer(consumer, memberships(producer, aspect, &selected));
        model.insert(consumer, selected);

        for query in [whole(1), detail(1, 7), detail(2, 8)] {
            let expected = expected_scope_candidates(&model, query);
            assert_eq!(
                fork.query_scope(
                    producer,
                    aspect,
                    query,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates,
                expected,
                "step {step} query {query:?}"
            );
        }
    }

    for query in [whole(1), detail(1, 7), detail(2, 8)] {
        assert_eq!(
            source
                .query_scope(
                    producer,
                    aspect,
                    query,
                    &mut crate::logic::evaluation::EvaluationWork::Ordinary
                )
                .unwrap()
                .candidates,
            expected_scope_candidates(&source_model, query),
            "mutating the fork must preserve the source model"
        );
    }
}

fn memberships(
    producer: NodeId,
    aspect: Aspect,
    scopes: &[Option<InternedPartitionSubscription>],
) -> Vec<IndexedSubscriptionMembership> {
    scopes
        .iter()
        .map(|scope| {
            IndexedSubscriptionMembership::from_edge(producer, aspect, *scope)
                .expect("model scopes are indexable")
        })
        .collect()
}

fn expected_scope_candidates(
    model: &BTreeMap<NodeId, Vec<Option<InternedPartitionSubscription>>>,
    query: InternedPartitionSubscription,
) -> Vec<NodeId> {
    model
        .iter()
        .filter_map(|(consumer, scopes)| {
            scopes
                .iter()
                .any(|scope| scope_matches(*scope, query))
                .then_some(*consumer)
        })
        .collect()
}

fn scope_matches(
    membership: Option<InternedPartitionSubscription>,
    query: InternedPartitionSubscription,
) -> bool {
    let Some(membership) = membership else {
        return true;
    };
    if membership.partition != query.partition {
        return false;
    }
    match query.match_mode {
        PartitionMatchMode::WholePartition => true,
        PartitionMatchMode::PartitionAndDetail => {
            membership.match_mode == PartitionMatchMode::WholePartition
                || membership.detail == query.detail
        }
    }
}

fn whole(partition: u32) -> InternedPartitionSubscription {
    InternedPartitionSubscription {
        partition: PartitionTokenId(partition),
        detail: None,
        match_mode: PartitionMatchMode::WholePartition,
    }
}

fn detail(partition: u32, detail: u32) -> InternedPartitionSubscription {
    InternedPartitionSubscription {
        partition: PartitionTokenId(partition),
        detail: Some(DetailTokenId(detail)),
        match_mode: PartitionMatchMode::PartitionAndDetail,
    }
}
