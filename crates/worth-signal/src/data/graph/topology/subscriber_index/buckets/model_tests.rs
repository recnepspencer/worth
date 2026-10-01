use std::collections::BTreeMap;

use super::{IndexedSubscriptionMembership, ReverseSubscriptionIndex};
use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::output::{
    InternedPartitionSubscription, InternedScopePath, PartitionTokenId, ScopeCoverage,
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
    let membership_path = membership.path();
    let query_path = query.path();
    let left = membership_path.segments();
    let right = query_path.segments();
    let left_prefix = right.starts_with(left);
    let right_prefix = left.starts_with(right);
    match (membership.coverage(), query.coverage()) {
        (ScopeCoverage::Exact, ScopeCoverage::Exact) => left == right,
        (ScopeCoverage::Subtree, ScopeCoverage::Exact) => left_prefix,
        (ScopeCoverage::Exact, ScopeCoverage::Subtree) => right_prefix,
        (ScopeCoverage::Subtree, ScopeCoverage::Subtree) => left_prefix || right_prefix,
    }
}

fn whole(partition: u32) -> InternedPartitionSubscription {
    InternedPartitionSubscription::new(
        InternedScopePath::new(&[PartitionTokenId(partition)]).unwrap(),
        ScopeCoverage::Subtree,
    )
}

fn detail(partition: u32, detail: u32) -> InternedPartitionSubscription {
    InternedPartitionSubscription::new(
        InternedScopePath::new(&[PartitionTokenId(partition), PartitionTokenId(detail)]).unwrap(),
        ScopeCoverage::Exact,
    )
}

#[test]
fn hierarchy_candidates_match_authoritative_membership_model_at_all_depths() {
    let producer = NodeId::new(0, 0);
    let aspect = Aspect::new(1);
    for depth in [1usize, 2, 4, 8] {
        let stem: Vec<_> = (0..depth)
            .map(|level| PartitionTokenId(level as u32))
            .collect();
        let leaf = InternedScopePath::new(&stem).unwrap();
        let ancestor = leaf.prefix(1).unwrap();
        let mut sibling = stem.clone();
        sibling[depth - 1] = PartitionTokenId(99);
        let sibling = InternedScopePath::new(&sibling).unwrap();
        let scopes = [
            None,
            Some(InternedPartitionSubscription::new(
                ancestor,
                ScopeCoverage::Subtree,
            )),
            Some(InternedPartitionSubscription::new(
                leaf,
                ScopeCoverage::Exact,
            )),
            Some(InternedPartitionSubscription::new(
                sibling,
                ScopeCoverage::Exact,
            )),
        ];
        let mut index = ReverseSubscriptionIndex::default();
        let mut model = BTreeMap::new();
        for (ordinal, scope) in scopes.into_iter().enumerate() {
            let consumer = NodeId::new((ordinal + 1) as u32, 0);
            index.replace_consumer(consumer, memberships(producer, aspect, &[scope]));
            model.insert(consumer, vec![scope]);
        }
        let mut fork = index.fork_persistent();
        for coverage in [ScopeCoverage::Exact, ScopeCoverage::Subtree] {
            let query = InternedPartitionSubscription::new(leaf, coverage);
            let expected = expected_scope_candidates(&model, query);
            assert_eq!(
                index
                    .query_scope(
                        producer,
                        aspect,
                        query,
                        &mut crate::logic::evaluation::EvaluationWork::Ordinary
                    )
                    .unwrap()
                    .candidates,
                expected,
                "flat depth {depth}"
            );
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
                "fork depth {depth}"
            );
        }
        fork.replace_consumer(NodeId::new(3, 0), Vec::new());
        index.replace_consumer(NodeId::new(3, 0), Vec::new());
        model.remove(&NodeId::new(3, 0));
        let query = InternedPartitionSubscription::new(leaf, ScopeCoverage::Exact);
        assert_eq!(
            fork.query_scope(
                producer,
                aspect,
                query,
                &mut crate::logic::evaluation::EvaluationWork::Ordinary
            )
            .unwrap()
            .candidates,
            expected_scope_candidates(&model, query)
        );
        let replacement = Some(InternedPartitionSubscription::new(
            sibling,
            ScopeCoverage::Subtree,
        ));
        index.replace_consumer(
            NodeId::new(2, 0),
            memberships(producer, aspect, &[replacement]),
        );
        model.insert(NodeId::new(2, 0), vec![replacement]);
        for changed in [leaf, sibling] {
            let query = InternedPartitionSubscription::new(changed, ScopeCoverage::Exact);
            assert_eq!(
                index
                    .query_scope(
                        producer,
                        aspect,
                        query,
                        &mut crate::logic::evaluation::EvaluationWork::Ordinary
                    )
                    .unwrap()
                    .candidates,
                expected_scope_candidates(&model, query),
                "replacement depth {depth}"
            );
        }
    }
}
