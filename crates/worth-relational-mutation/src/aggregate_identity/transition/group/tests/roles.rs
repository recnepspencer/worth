//! Continuity roles must hold before and after; required roles never vanish.
use super::*;

fn step(
    world: &World,
    rule: Rule,
    prior: &[AggregatePriorGroup],
    next: &[Spec<'_>],
) -> Result<AggregateGroupPartitionTransition, Denial> {
    world.run(rule, prior, &BTreeSet::new(), next)
}

#[test]
fn demoted_producer_hands_off_at_once_and_the_repeat_is_a_no_op() {
    let rule = Rule::ProducerRole;
    for world in worlds(&[50, 60, 1]) {
        let first = world.born(rule, &[&[(50, Role::Anchor), (1, Role::Ordinary)]]);
        let handoff: Spec<'_> = &[
            (50, Role::Ordinary),
            (60, Role::Anchor),
            (1, Role::Ordinary),
        ];
        let second = step(&world, rule, &first, &[handoff]).unwrap();
        let successor = tid(rule, 60, &[id(rule, 50)]);
        assert_eq!(second.assignments[0].identity, successor);
        assert_eq!(second.retired, BTreeSet::from([id(rule, 50)]));
        let third = step(&world, rule, &prior_of(&second), &[handoff]).unwrap();
        assert_eq!(third.assignments[0].identity, successor);
        assert_eq!(third.assignments[0].origin, AggregateGroupOrigin::Kept);
        assert!(third.retired.is_empty() && third.lineage.is_empty());
    }
}

#[test]
fn boundary_counts_only_walls_that_stay_walls() {
    let rule = Rule::Boundary;
    let world = World::new(&[10, 11, 12, 20, 21], 3, 1, 1);
    let prior = world.born(rule, &[&[(10, Role::Boundary), (11, Role::Boundary)]]);
    let demoted: Spec<'_> = &[
        (10, Role::Ordinary),
        (11, Role::Ordinary),
        (20, Role::Boundary),
    ];
    let next = step(&world, rule, &prior, &[demoted, &[(21, Role::Boundary)]]).unwrap();
    assert!(next
        .assignments
        .iter()
        .all(|group| group.identity != id(rule, 10)));
    assert_eq!(next.retired, BTreeSet::from([id(rule, 10)]));
    let kept: Spec<'_> = &[
        (10, Role::Ordinary),
        (11, Role::Boundary),
        (12, Role::Boundary),
    ];
    let once = step(&world, rule, &prior, &[kept]).unwrap();
    assert_eq!(once.assignments[0].identity, id(rule, 10));
    let twice = step(&world, rule, &prior_of(&once), &[kept]).unwrap();
    assert_eq!(project(&twice).0, project(&once).0);
    assert!(twice.lineage.is_empty());
}

#[test]
fn demoting_the_last_required_role_member_refuses_even_for_a_keeper() {
    let world = World::new(&[50, 1, 10, 11], 0, 1, 1);
    let producer = world.born(
        Rule::ProducerRole,
        &[&[(50, Role::Anchor), (1, Role::Ordinary)]],
    );
    assert_eq!(
        step(
            &world,
            Rule::ProducerRole,
            &producer,
            &[&[(50, Role::Ordinary), (1, Role::Ordinary)]]
        ),
        Err(Denial::MissingAnchorMember {
            rule: Rule::ProducerRole,
            least_member: name(1),
        })
    );
    let walls = world.born(
        Rule::Boundary,
        &[&[(10, Role::Boundary), (11, Role::Boundary)]],
    );
    assert_eq!(
        step(
            &world,
            Rule::Boundary,
            &walls,
            &[&[(10, Role::Ordinary), (11, Role::Ordinary)]]
        ),
        Err(Denial::MissingAnchorMember {
            rule: Rule::Boundary,
            least_member: name(10),
        })
    );
}
