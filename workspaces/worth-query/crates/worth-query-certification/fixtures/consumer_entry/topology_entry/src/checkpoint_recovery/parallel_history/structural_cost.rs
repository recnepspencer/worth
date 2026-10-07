//! Structural costs are input expressions, never report measurements or time.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Cost {
    pub work: u64,
    pub span: u64,
}

pub(super) enum Operations {
    Units(u64),
    Serial(Vec<Operations>),
    Parallel(Vec<Operations>),
}

impl Operations {
    pub(super) fn cost(&self) -> Cost {
        match self {
            Self::Units(work) => Cost {
                work: *work,
                span: *work,
            },
            Self::Serial(children) => {
                children
                    .iter()
                    .map(Self::cost)
                    .fold(Cost { work: 0, span: 0 }, |sum, cost| Cost {
                        work: sum.work + cost.work,
                        span: sum.span + cost.span,
                    })
            }
            Self::Parallel(children) => {
                children
                    .iter()
                    .map(Self::cost)
                    .fold(Cost { work: 0, span: 0 }, |sum, cost| Cost {
                        work: sum.work + cost.work,
                        span: sum.span.max(cost.span),
                    })
            }
        }
    }
}

#[test]
fn waves_add_and_nested_independent_children_take_the_maximum() {
    use Operations::{Parallel as P, Serial as S, Units as U};
    let [prepare, nested_prepare, left, right, peer, complete, next_left, next_right] =
        [2_u64, 3, 4, 7, 6, 1, 5, 8];
    let stages = S(vec![
        U(prepare),
        P(vec![
            S(vec![U(nested_prepare), P(vec![U(left), U(right)])]),
            U(peer),
        ]),
        U(complete),
        P(vec![U(next_left), U(next_right)]),
    ]);
    assert_eq!(
        stages.cost(),
        Cost {
            work: prepare
                + nested_prepare
                + left
                + right
                + peer
                + complete
                + next_left
                + next_right,
            span: prepare
                + (nested_prepare + left.max(right)).max(peer)
                + complete
                + next_left.max(next_right)
        }
    );
    assert_ne!(
        stages.cost().span,
        stages.cost().work,
        "summing independent spans is wrong"
    );
}

#[test]
fn empty_singleton_and_one_positive_branch_have_no_strict_span_gain() {
    use Operations::{Parallel as P, Units as U};
    for operations in [P(vec![]), P(vec![U(5)]), P(vec![U(0), U(5)])] {
        let cost = operations.cost();
        assert_eq!(cost.span, cost.work);
    }
    let cost = P(vec![U(5), U(1)]).cost();
    assert!(cost.span < cost.work);
}

/// The declared kernel stages of a DAG: gather/complete costs are separate
/// serial stages when part two supplies their declared advancement contract.
pub(super) fn declared_waves(
    world: &super::expected_history::World,
    rule: super::expected_history::OrderRule,
) -> Operations {
    if rule == super::expected_history::OrderRule::CallerTraversal {
        return Operations::Serial(
            world
                .order(rule)
                .iter()
                .map(|index| {
                    Operations::Parallel(
                        world.members[*index]
                            .partitions
                            .iter()
                            .map(|p| Operations::Units(p.work))
                            .collect(),
                    )
                })
                .collect(),
        );
    }
    let mut committed = std::collections::BTreeSet::new();
    let required = world.order(super::expected_history::OrderRule::CallerTraversal);
    let mut waves = Vec::new();
    while committed.len() < required.len() {
        let ready = required
            .iter()
            .copied()
            .filter(|index| {
                !committed.contains(index)
                    && world.members[*index]
                        .upstream
                        .iter()
                        .all(|upstream| committed.contains(upstream))
            })
            .collect::<Vec<_>>();
        assert!(!ready.is_empty());
        waves.push(Operations::Parallel(
            ready
                .iter()
                .map(|index| {
                    Operations::Parallel(
                        world.members[*index]
                            .partitions
                            .iter()
                            .map(|partition| Operations::Units(partition.work))
                            .collect(),
                    )
                })
                .collect(),
        ));
        committed.extend(ready);
    }
    Operations::Serial(waves)
}
#[test]
fn seeded_structural_work_sums_declared_operations_and_span_follows_readiness() {
    use super::seeded_world::{world, Family, FAMILIES, SEED};
    for family in FAMILIES {
        let input = world(family, SEED);
        let cost =
            declared_waves(&input, super::expected_history::OrderRule::CanonicalWaves).cost();
        assert_eq!(
            cost.work,
            input
                .members
                .iter()
                .flat_map(|member| &member.partitions)
                .map(|partition| partition.work)
                .sum::<u64>()
        );
        let independent = matches!(
            family,
            Family::Diamond | Family::Independent | Family::Unequal | Family::Nested
        );
        assert_eq!(
            cost.span < cost.work,
            independent,
            "{family:?}: positive independent contributions"
        );
    }
}

/// The reduction owner declares the cost of a complete checked build. This
/// fixture composes that declaration; it does not reproduce the shape law.
pub(super) fn reduction_work(identities: &[u64]) -> u64 {
    worth_execution::ReductionPlan::try_from_sorted_unique(
        identities
            .iter()
            .copied()
            .map(worth_foundational::PartitionIdentity::new)
            .collect(),
    )
    .unwrap()
    .checked_build_work()
    .unwrap()
}

#[test]
fn both_order_rules_compose_the_same_work_and_distinct_member_spans() {
    use super::expected_history::{Member, OrderRule, Partition, World};
    let member = |key, upstream, costs: &[u64]| Member {
        key,
        upstream,
        partitions: costs
            .iter()
            .enumerate()
            .map(|(rank, work)| Partition {
                key: rank as u64,
                value: 1,
                work: *work,
                fails: false,
            })
            .collect(),
    };
    let root = [3_u64, 7];
    let peer = 6;
    let downstream = [5_u64, 8];
    let input = World {
        members: vec![
            member(30, vec![], &root),
            member(10, vec![], &[peer]),
            member(20, vec![0, 1], &downstream),
        ],
        requested: vec![2],
    };
    let work = root.iter().sum::<u64>() + peer + downstream.iter().sum::<u64>();
    let root_span = *root.iter().max().unwrap();
    let downstream_span = *downstream.iter().max().unwrap();
    assert_eq!(
        declared_waves(&input, OrderRule::CallerTraversal).cost(),
        Cost {
            work,
            span: root_span + peer + downstream_span
        }
    );
    assert_eq!(
        declared_waves(&input, OrderRule::CanonicalWaves).cost(),
        Cost {
            work,
            span: root_span.max(peer) + downstream_span
        }
    );
}

/// A fresh route reroutes one item and visits its one membership. The owner
/// pins both counts; composing its public declaration keeps the unit law here.
pub(super) fn routing_work(items: u64) -> u64 {
    worth_execution::PartitionWork {
        items_rerouted: items,
        members_visited: items,
        ..worth_execution::PartitionWork::default()
    }
    .units()
    .unwrap()
}
