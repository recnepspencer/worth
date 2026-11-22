//! Typed refusals: live name collisions and malformed group states.
use super::*;

#[test]
fn simultaneous_live_groups_claiming_one_anchor_refuse_naming_both() {
    let rule = Rule::Boundary;
    for world in worlds(&[10, 11, 12, 1, 30]) {
        let prior = world.born(
            rule,
            &[&[
                (10, Role::Boundary),
                (11, Role::Boundary),
                (12, Role::Boundary),
            ]],
        );
        let before = prior.clone();
        let keeper: Spec<'_> = &[
            (11, Role::Boundary),
            (12, Role::Boundary),
            (1, Role::Ordinary),
        ];
        let claimant: Spec<'_> = &[(10, Role::Boundary), (30, Role::Boundary)];
        for order in [[keeper, claimant], [claimant, keeper]] {
            let refused = world.run(rule, &prior, &BTreeSet::new(), &order);
            let expected = Denial::LiveNameCollision {
                identity: id(rule, 10),
                first: AggregateGroupClaimant {
                    anchor: name(10),
                    least_member: name(1),
                    kept_from: Some(id(rule, 10)),
                },
                second: AggregateGroupClaimant {
                    anchor: name(10),
                    least_member: name(10),
                    kept_from: None,
                },
            };
            assert_eq!(refused, Err(expected));
        }
        assert_eq!(prior, before, "a refusal leaves the prior state untouched");
    }
}

#[test]
fn malformed_states_refuse_with_the_true_subject() {
    let world = World::new(&[1, 2, 3], 0, 1, 1);
    let rule = Rule::ProducerRole;
    let none = BTreeSet::new();
    assert_eq!(
        world.run(
            rule,
            &[],
            &none,
            &[&[(2, Role::Ordinary), (3, Role::Boundary)]]
        ),
        Err(Denial::MissingAnchorMember {
            rule,
            least_member: name(2),
        })
    );
    assert_eq!(
        world.run(Rule::Boundary, &[], &none, &[&[(1, Role::Anchor)]]),
        Err(Denial::MissingAnchorMember {
            rule: Rule::Boundary,
            least_member: name(1),
        })
    );
    assert_eq!(
        world.run(Rule::Anchor, &[], &none, &[&[(1, Role::Ordinary)], &[]]),
        Err(Denial::EmptyGroup)
    );
    assert_eq!(
        world.run(
            Rule::Anchor,
            &[],
            &none,
            &[&[(1, Role::Ordinary)], &[(1, Role::Anchor)]]
        ),
        Err(Denial::OverlappingMember { name: name(1) })
    );
    let mut forged = world.born(Rule::Anchor, &[&[(1, Role::Ordinary)]]);
    forged[0].anchor = name(2);
    let identity = forged[0].identity;
    assert_eq!(
        world.run(Rule::Anchor, &forged, &none, &[]),
        Err(Denial::PriorIdentityMismatch { identity })
    );
    let live = world.born(Rule::Anchor, &[&[(1, Role::Ordinary)]]);
    assert_eq!(
        world.run(Rule::Anchor, &live, &BTreeSet::from([identity]), &[]),
        Err(Denial::RetiredIdentityLive { identity })
    );
    let mut twin = AggregateGroupMembers::new();
    let carrier = NativeMemberKey::Entity(EntityId::new(PartitionId::new(1), 0, 1));
    twin.insert(name(1), carrier.clone(), Role::Ordinary)
        .unwrap();
    assert_eq!(
        twin.insert(name(1), carrier.clone(), Role::Ordinary),
        Err(Denial::DuplicateMemberName { name: name(1) })
    );
    twin.insert(name(2), carrier, Role::Ordinary).unwrap();
    assert_eq!(
        AggregateGroupNamingAuthority::transition(&scheme(Rule::Anchor), &[], &none, vec![twin]),
        Err(Denial::DuplicateCarrier { name: name(2) })
    );
    assert!(matches!(
        AggregateGroupSchemeId::new("Bad Scheme"),
        Err(Denial::InvalidSchemeId { .. })
    ));
}
