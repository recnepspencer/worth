//! Pure meaning: no execution, owner, comparator, or reducer dependencies.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, serde::Serialize)]
pub struct Partition {
    pub key: u64,
    pub value: u64,
    pub work: u64,
    pub fails: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Member {
    pub key: u64,
    pub upstream: Vec<usize>,
    pub partitions: Vec<Partition>,
}

#[derive(Clone, Debug)]
pub(crate) struct World {
    pub members: Vec<Member>,
    pub requested: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OrderRule {
    CallerTraversal,
    CanonicalWaves,
}

/// The charge boundary is independent of publication order. The current
/// completed-run observer omits failed members; aggregate settlement replaces
/// this parameter in part two without changing the dependency evaluator.
#[derive(Clone, Copy)]
pub(crate) struct FailureChargeRule(pub fn(&World, OrderRule, &History, u64) -> u64);
impl FailureChargeRule {
    pub const THROUGH_LEAST_PARTITION: Self = Self(|_, _, history, _| history.kernel_work);
    pub const COMPLETED_MEMBERS_ONLY: Self = Self(|_, _, _, before_member| before_member);
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct History {
    pub effects: Vec<(u64, [u8; 8])>,
    pub failure: Option<(u64, u64)>,
    pub kernel_work: u64,
}

impl World {
    pub fn order(&self, rule: OrderRule) -> Vec<usize> {
        let mut required = BTreeSet::new();
        let mut traversal = Vec::new();
        for &member in &self.requested {
            self.visit(member, &mut required, &mut traversal);
        }
        if rule == OrderRule::CallerTraversal {
            return traversal;
        }
        let mut committed = BTreeSet::new();
        let mut ordered = Vec::new();
        while committed.len() < required.len() {
            let mut ready: Vec<_> = required
                .iter()
                .copied()
                .filter(|member| {
                    !committed.contains(member)
                        && self.members[*member]
                            .upstream
                            .iter()
                            .all(|u| committed.contains(u))
                })
                .collect();
            assert!(!ready.is_empty(), "a seeded graph is acyclic");
            ready.sort_by_key(|member| self.members[*member].key);
            committed.extend(ready.iter().copied());
            ordered.extend(ready);
        }
        ordered
    }

    fn visit(&self, member: usize, visited: &mut BTreeSet<usize>, order: &mut Vec<usize>) {
        if !visited.insert(member) {
            return;
        }
        for &upstream in &self.members[member].upstream {
            self.visit(upstream, visited, order);
        }
        order.push(member);
    }

    pub fn expected(&self, rule: OrderRule, failure_charge: FailureChargeRule) -> History {
        let mut history = History {
            effects: Vec::new(),
            failure: None,
            kernel_work: 0,
        };
        let mut values = BTreeMap::new();
        for index in self.order(rule) {
            let member = &self.members[index];
            let before_member = history.kernel_work;
            let mut value: u64 = member.upstream.iter().map(|u| values[u]).sum();
            let mut partitions: Vec<_> = member.partitions.iter().collect();
            partitions.sort_by_key(|partition| partition.key);
            for partition in partitions {
                history.kernel_work += partition.work;
                if partition.fails {
                    history.failure = Some((member.key, partition.key));
                    history.kernel_work = (failure_charge.0)(self, rule, &history, before_member);
                    return history;
                }
                value += partition.value;
            }
            values.insert(index, value);
            history.effects.push((member.key, value.to_le_bytes()));
        }
        history
    }
}

#[test]
fn canonical_readiness_is_distinct_from_caller_and_completion_order() {
    let member = |key, upstream, work| Member {
        key,
        upstream,
        partitions: vec![Partition {
            key: 0,
            value: 1,
            work,
            fails: false,
        }],
    };
    let world = World {
        members: vec![
            member(30, vec![], 9),
            member(10, vec![], 3),
            member(20, vec![], 1),
            member(40, vec![0, 1, 2], 2),
        ],
        requested: vec![3],
    };
    assert_eq!(world.order(OrderRule::CallerTraversal), [0, 1, 2, 3]);
    assert_eq!(world.order(OrderRule::CanonicalWaves), [1, 2, 0, 3]);
    // Structural finish costs, not a clock or a recorded completion order.
    let mut finishes = BTreeMap::new();
    let mut completion_order = world.order(OrderRule::CallerTraversal);
    for &index in &completion_order {
        let member = &world.members[index];
        let start = member
            .upstream
            .iter()
            .map(|upstream| finishes[upstream])
            .max()
            .unwrap_or(0);
        let span = member
            .partitions
            .iter()
            .map(|partition| partition.work)
            .max()
            .unwrap_or(0);
        finishes.insert(index, start + span);
    }
    completion_order.sort_by_key(|index| finishes[index]);
    assert_ne!(world.order(OrderRule::CallerTraversal), completion_order);
    assert_ne!(world.order(OrderRule::CanonicalWaves), completion_order);
}

#[test]
fn shared_upstream_is_applied_once_and_a_failure_keeps_only_the_prefix() {
    let p = |key, fails| Partition {
        key,
        value: key,
        work: 2,
        fails,
    };
    let world = World {
        members: vec![
            Member {
                key: 1,
                upstream: vec![],
                partitions: vec![p(1, false)],
            },
            Member {
                key: 2,
                upstream: vec![0],
                partitions: vec![p(4, true), p(2, true)],
            },
            Member {
                key: 3,
                upstream: vec![0],
                partitions: vec![p(1, false)],
            },
        ],
        requested: vec![1, 2],
    };
    assert_eq!(
        world.expected(
            OrderRule::CallerTraversal,
            FailureChargeRule::THROUGH_LEAST_PARTITION
        ),
        History {
            effects: vec![(1, 1_u64.to_le_bytes())],
            failure: Some((2, 2)),
            kernel_work: 4
        }
    );
}
