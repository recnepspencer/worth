//! Split and merge keepers follow each declared continuity rule in every
//! native creation order; unkept successors carry their prior sources.
use super::*;
use AggregateGroupOrigin::{Born, Kept};

/// Runs one prior/next scenario in every allocation world and returns the
/// single carrier-free outcome they all agree on.
pub(super) fn agreed(
    rule: Rule,
    names: &[u8],
    before: &[Spec<'_>],
    after: &[Spec<'_>],
) -> AggregateGroupTransition {
    let all = worlds(names);
    let outcome = |world: &World| {
        let prior = world.born(rule, before);
        world.run(rule, &prior, &BTreeSet::new(), after).unwrap()
    };
    let reference = outcome(&all[0]);
    for world in &all[1..] {
        assert_eq!(project(&outcome(world)), project(&reference));
    }
    reference
}

fn group_with(transition: &AggregateGroupTransition, n: u8) -> &AggregateGroupAssignment {
    transition
        .assignments
        .iter()
        .find(|group| group.members.get(&name(n)).is_some())
        .unwrap()
}

const B: Role = Role::Boundary;

#[test]
fn anchor_rule_split_keeps_the_anchor_fragment_not_the_least_member() {
    let rule = Rule::Anchor;
    let before: Spec<'_> = &[(7, Role::Anchor), (2, Role::Ordinary), (9, Role::Ordinary)];
    let next = agreed(
        rule,
        &[2, 4, 7, 9],
        &[before],
        &[
            &[(2, Role::Ordinary), (9, Role::Ordinary)],
            &[(7, Role::Anchor), (4, Role::Ordinary)],
        ],
    );
    let (kept, split) = (group_with(&next, 7), group_with(&next, 2));
    let fragment = tid(rule, 2, &[id(rule, 7)]);
    assert_eq!((kept.identity, kept.origin), (id(rule, 7), Kept));
    assert_eq!((split.identity, split.origin), (fragment, Born));
    assert_eq!(split.sources, [id(rule, 7)]);
    assert_eq!(
        next.lineage,
        BTreeSet::from([
            Event::Born { group: fragment },
            Event::SplitFrom {
                fragment,
                source: id(rule, 7),
            },
        ])
    );
    assert!(next.retired.is_empty());
}

#[test]
fn anchor_rule_merge_keeps_the_least_anchor_name() {
    let rule = Rule::Anchor;
    let next = agreed(
        rule,
        &[7, 4, 3, 8],
        &[
            &[(7, Role::Anchor), (4, Role::Ordinary)],
            &[(3, Role::Anchor), (8, Role::Ordinary)],
        ],
        &[&[
            (7, Role::Anchor),
            (4, Role::Ordinary),
            (3, Role::Anchor),
            (8, Role::Ordinary),
        ]],
    );
    assert_eq!(next.assignments[0].identity, id(rule, 3));
    assert_eq!(next.retired, BTreeSet::from([id(rule, 7)]));
    assert!(next.lineage.contains(&Event::MergedInto {
        absorbed: id(rule, 7),
        keeper: id(rule, 3),
    }));
}

#[test]
fn boundary_rule_split_follows_the_most_walls_even_away_from_the_anchor() {
    let rule = Rule::Boundary;
    let walls: Spec<'_> = &[(10, B), (11, B), (12, B), (1, Role::Ordinary)];
    let next = agreed(
        rule,
        &[10, 11, 12, 1, 30],
        &[walls],
        &[
            &[(11, B), (12, B), (1, Role::Ordinary)],
            &[(10, B), (30, B)],
        ],
    );
    let (kept, split) = (group_with(&next, 11), group_with(&next, 10));
    assert_eq!((kept.identity, kept.anchor), (id(rule, 10), name(10)));
    assert_eq!(kept.origin, Kept);
    let fragment = tid(rule, 10, &[id(rule, 10)]);
    assert_eq!((split.identity, split.origin), (fragment, Born));
    assert!(next.lineage.contains(&Event::SplitFrom {
        fragment,
        source: id(rule, 10),
    }));
}

#[test]
fn boundary_rule_merge_keeps_the_most_walls() {
    let rule = Rule::Boundary;
    let merged = agreed(
        rule,
        &[10, 11, 20, 21, 22],
        &[&[(10, B), (11, B)], &[(20, B), (21, B), (22, B)]],
        &[&[(10, B), (11, B), (20, B), (21, B), (22, B)]],
    );
    assert_eq!(merged.assignments[0].identity, id(rule, 20));
    assert_eq!(merged.retired, BTreeSet::from([id(rule, 10)]));
}

#[test]
fn boundary_two_by_two_split_tie_mints_two_successors_and_retires_the_source() {
    let rule = Rule::Boundary;
    let tied = agreed(
        rule,
        &[10, 11, 12, 13],
        &[&[(10, B), (11, B), (12, B), (13, B)]],
        &[&[(10, B), (11, B)], &[(12, B), (13, B)]],
    );
    let source = id(rule, 10);
    let (left, right) = (tid(rule, 10, &[source]), tid(rule, 12, &[source]));
    let born: BTreeSet<_> = tied
        .assignments
        .iter()
        .map(|g| (g.identity, g.origin))
        .collect();
    assert_eq!(born, BTreeSet::from([(left, Born), (right, Born)]));
    assert_eq!(tied.retired, BTreeSet::from([source]));
    assert_eq!(
        tied.lineage,
        BTreeSet::from([
            Event::Born { group: left },
            Event::Born { group: right },
            Event::SplitFrom {
                fragment: left,
                source,
            },
            Event::SplitFrom {
                fragment: right,
                source,
            },
            Event::Retired { group: source },
        ])
    );
}

#[test]
fn boundary_two_by_two_merge_tie_mints_a_successor_of_both_sources() {
    let rule = Rule::Boundary;
    let tied = agreed(
        rule,
        &[10, 11, 20, 21],
        &[&[(10, B), (11, B)], &[(20, B), (21, B)]],
        &[&[(10, B), (11, B), (20, B), (21, B)]],
    );
    let (first, second) = (id(rule, 10), id(rule, 20));
    let merged = tid(rule, 10, &[first, second]);
    assert_eq!(tied.assignments[0].identity, merged);
    assert_eq!(tied.assignments[0].origin, Born);
    assert_eq!(tied.retired, BTreeSet::from([first, second]));
    assert_eq!(
        tied.lineage,
        BTreeSet::from([
            Event::Born { group: merged },
            Event::MergedInto {
                absorbed: first,
                keeper: merged,
            },
            Event::MergedInto {
                absorbed: second,
                keeper: merged,
            },
            Event::Retired { group: first },
            Event::Retired { group: second },
        ])
    );
}

#[test]
fn producer_role_rule_follows_the_producer_and_merges_to_the_least_producer() {
    let rule = Rule::ProducerRole;
    let split = agreed(
        rule,
        &[50, 1, 2, 60],
        &[&[(50, Role::Anchor), (1, Role::Ordinary), (2, Role::Ordinary)]],
        &[
            &[(1, Role::Ordinary), (60, Role::Anchor)],
            &[(50, Role::Anchor), (2, Role::Ordinary)],
        ],
    );
    assert_eq!(group_with(&split, 50).identity, id(rule, 50));
    assert_eq!(group_with(&split, 50).origin, Kept);
    assert_eq!(
        group_with(&split, 1).identity,
        tid(rule, 60, &[id(rule, 50)])
    );
    let merged = agreed(
        rule,
        &[50, 40, 1],
        &[
            &[(50, Role::Anchor), (1, Role::Ordinary)],
            &[(40, Role::Anchor)],
        ],
        &[&[(50, Role::Anchor), (40, Role::Anchor), (1, Role::Ordinary)]],
    );
    assert_eq!(merged.assignments[0].identity, id(rule, 40));
    assert!(merged.lineage.contains(&Event::MergedInto {
        absorbed: id(rule, 50),
        keeper: id(rule, 40),
    }));
}
