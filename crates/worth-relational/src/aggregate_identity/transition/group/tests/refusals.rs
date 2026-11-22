//! Typed refusals: live name collisions and malformed group states.
use super::*;

const B: Role = Role::Boundary;

/// `G = {10, 11, 12}` named by wall 10, which then leaves it entirely.
fn detached_anchor(world: &World) -> Vec<AggregatePriorGroup> {
    let rule = Rule::Boundary;
    let born = world.born(rule, &[&[(10, B), (11, B), (12, B)]]);
    let next = world.run(rule, &born, &BTreeSet::new(), &[&[(11, B), (12, B)]]);
    let next = next.unwrap();
    assert_eq!(next.assignments[0].identity, id(rule, 10));
    prior_of(&next)
}

fn claimant(anchor: u8, least: u8, live_prior: Option<u8>) -> AggregateGroupClaimant {
    AggregateGroupClaimant {
        anchor: name(anchor),
        least_member: name(least),
        live_prior: live_prior.map(|anchor| id(Rule::Boundary, anchor)),
    }
}

#[test]
fn simultaneous_live_groups_claiming_one_anchor_refuse_naming_both() {
    let rule = Rule::Boundary;
    for world in worlds(&[10, 11, 12, 1, 30]) {
        let prior = detached_anchor(&world);
        let before = prior.clone();
        let keeper: Spec<'_> = &[(11, B), (12, B), (1, Role::Ordinary)];
        let fresh: Spec<'_> = &[(10, B), (30, B)];
        for order in [[keeper, fresh], [fresh, keeper]] {
            assert_eq!(
                world.run(rule, &prior, &BTreeSet::new(), &order),
                Err(Denial::LiveNameCollision {
                    identity: id(rule, 10),
                    first: claimant(10, 1, Some(10)),
                    second: claimant(10, 10, None),
                })
            );
        }
        assert_eq!(prior, before, "a refusal leaves the prior state untouched");
    }
}

#[test]
fn unrelated_birth_never_inherits_a_dissolving_or_merging_group_name() {
    let rule = Rule::Boundary;
    for world in worlds(&[10, 11, 12, 20, 21, 22, 99]) {
        let mut prior = detached_anchor(&world);
        let fresh: Spec<'_> = &[(10, B), (99, B)];
        let expected = Err(Denial::LiveNameCollision {
            identity: id(rule, 10),
            first: claimant(10, 10, None),
            second: claimant(10, 11, Some(10)),
        });
        assert_eq!(
            world.run(rule, &prior, &BTreeSet::new(), &[fresh]),
            expected
        );
        prior.extend(world.born(rule, &[&[(20, B), (21, B), (22, B)]]));
        let merged: Spec<'_> = &[(11, B), (12, B), (20, B), (21, B), (22, B)];
        let none = BTreeSet::new();
        assert_eq!(world.run(rule, &prior, &none, &[merged, fresh]), expected);
        let absorbed = world.run(rule, &prior, &none, &[merged]).unwrap();
        assert_eq!(absorbed.retired, BTreeSet::from([id(rule, 10)]));
        assert!(absorbed.lineage.contains(&Event::MergedInto {
            absorbed: id(rule, 10),
            keeper: id(rule, 20),
        }));
        let after = world.run(
            rule,
            &prior_of(&absorbed),
            &absorbed.retired,
            &[merged, fresh],
        );
        let after = after.unwrap();
        let revived = after
            .assignments
            .iter()
            .find(|g| g.identity == id(rule, 10));
        assert_eq!(revived.unwrap().origin, AggregateGroupOrigin::Revived);
    }
}

#[test]
fn scheme_ids_follow_the_declaration_grammar() {
    for lawful in ["cad.Space.v1", "cad.loop", "a", "worth.example.face.v1"] {
        assert!(AggregateGroupSchemeId::new(lawful).is_ok(), "{lawful}");
    }
    for unlawful in [
        "",
        "...",
        ".a..b.",
        ".cad",
        "cad.",
        "a..b",
        "cad space",
        " cad",
        "cad\n",
    ] {
        assert_eq!(
            AggregateGroupSchemeId::new(unlawful),
            Err(Denial::InvalidSchemeId {
                scheme_id: unlawful.to_owned()
            }),
            "{unlawful:?}"
        );
    }
}

#[test]
fn malformed_states_refuse_with_the_true_subject() {
    let world = World::new(&[1, 2, 3], 0, 1, 1);
    let none = BTreeSet::new();
    assert_eq!(
        world.run(Rule::Anchor, &[], &none, &[&[(1, Role::Ordinary)], &[]]),
        Err(Denial::EmptyGroup)
    );
    let overlap: [Spec<'_>; 2] = [&[(1, Role::Ordinary)], &[(1, Role::Anchor)]];
    assert_eq!(
        world.run(Rule::Anchor, &[], &none, &overlap),
        Err(Denial::OverlappingMember { name: name(1) })
    );
    let mut forged = world.born(Rule::Anchor, &[&[(1, Role::Ordinary)]]);
    forged[0].anchor = name(2);
    let identity = forged[0].identity;
    assert_eq!(
        world.run(Rule::Anchor, &forged, &none, &[]),
        Err(Denial::PriorIdentityMismatch { identity })
    );
    let mut unsourced = world.born(Rule::Anchor, &[&[(1, Role::Ordinary)]]);
    unsourced[0].sources = vec![id(Rule::Anchor, 3)];
    assert_eq!(
        world.run(Rule::Anchor, &unsourced, &none, &[]),
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
}

#[test]
fn non_canonical_prior_sources_refuse_without_hashing() {
    let world = World::new(&[1, 2], 0, 1, 1);
    let (a, b) = (id(Rule::Anchor, 3), id(Rule::Anchor, 4));
    let (low, high) = (a.min(b), a.max(b));
    for sources in [vec![high, low], vec![low, low]] {
        let mut prior = world.born(Rule::Anchor, &[&[(1, Role::Ordinary)]]);
        let identity = prior[0].identity;
        prior[0].sources = sources;
        assert_eq!(
            world.run(
                Rule::Anchor,
                &prior,
                &BTreeSet::new(),
                &[&[(1, Role::Ordinary)]]
            ),
            Err(Denial::PriorIdentityMismatch { identity })
        );
    }
}
