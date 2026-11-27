//! Group names follow declared continuity, never native allocation or commit.
use super::*;
use crate::contracts::graph_member::key::NativeMemberKey;
use crate::contracts::identity::data::primitives::PartitionId;
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::lineage::data::naming::PersistentNameIdentityOwnerPort as _;
use AggregateContinuityRule as Rule;
pub mod continuity;
pub mod delta_authority_matches_full_partition;
pub mod memory_state;
pub mod property;
pub mod refusals;
pub mod roles;

pub(super) type Spec<'a> = &'a [(u8, Role)];

pub(super) fn name(n: u8) -> Name {
    let mut bytes = [0x5a; 32];
    bytes[0] = n;
    Name::from_minted_digest(bytes)
}

pub(super) fn scheme(rule: Rule) -> AggregateGroupScheme {
    AggregateGroupScheme::new(AggregateGroupSchemeId::new("cad.loop").unwrap(), rule)
}

/// A native allocator whose slot order is chosen independently of names.
pub(super) struct World {
    carriers: BTreeMap<u8, NativeMemberKey>,
}

impl World {
    /// Allocates every name in `creation` order after `dummies` throwaway
    /// slots, on its own partition and generation.
    pub(super) fn new(creation: &[u8], dummies: u64, partition: u32, generation: u32) -> Self {
        let carriers = creation
            .iter()
            .enumerate()
            .map(|(slot, &n)| {
                let entity = EntityId::new(
                    PartitionId::new(partition),
                    dummies + slot as u64,
                    generation,
                );
                (n, NativeMemberKey::Entity(entity))
            })
            .collect();
        Self { carriers }
    }

    pub(super) fn members(&self, spec: Spec<'_>) -> AggregateGroupMembers {
        let mut members = AggregateGroupMembers::new();
        for &(n, role) in spec {
            members
                .insert(name(n), self.carriers[&n].clone(), role)
                .unwrap();
        }
        members
    }

    pub(super) fn run(
        &self,
        rule: Rule,
        prior: &[AggregatePriorGroup],
        retired: &BTreeSet<AggregateIdentity>,
        next: &[Spec<'_>],
    ) -> Result<AggregateGroupPartitionTransition, Denial> {
        let proposed = next.iter().map(|spec| self.members(spec)).collect();
        AggregateGroupNamingAuthority::transition(&scheme(rule), prior, retired, proposed)
    }

    /// Births `groups` from nothing and returns them as prior live groups.
    pub(super) fn born(&self, rule: Rule, groups: &[Spec<'_>]) -> Vec<AggregatePriorGroup> {
        prior_of(&self.run(rule, &[], &BTreeSet::new(), groups).unwrap())
    }
}

pub(super) fn prior_of(transition: &AggregateGroupPartitionTransition) -> Vec<AggregatePriorGroup> {
    transition
        .assignments
        .iter()
        .map(|assignment| AggregatePriorGroup {
            identity: assignment.identity,
            anchor: assignment.anchor,
            sources: assignment.sources.clone(),
            members: assignment.members.clone(),
        })
        .collect()
}

/// One next-state group without its native carriers.
pub(super) type ProjectedGroup = (
    AggregateIdentity,
    Name,
    Vec<AggregateIdentity>,
    AggregateGroupOrigin,
    Vec<(Name, Role)>,
);

/// Everything observable except native carriers.
pub(super) type Projection = (
    Vec<ProjectedGroup>,
    BTreeSet<AggregateIdentity>,
    BTreeSet<Event>,
);

pub(super) fn project(transition: &AggregateGroupPartitionTransition) -> Projection {
    let groups = transition
        .assignments
        .iter()
        .map(|group| {
            let members = group.members.iter().map(|(n, m)| (*n, m.role)).collect();
            let sources = group.sources.clone();
            (group.identity, group.anchor, sources, group.origin, members)
        })
        .collect();
    (
        groups,
        transition.retired.clone(),
        transition.lineage.clone(),
    )
}

pub(super) fn id(rule: Rule, anchor: u8) -> AggregateIdentity {
    AggregateIdentity::from_group_name(scheme(rule).id(), name(anchor), rule)
}

/// The successor name of an unkept split/merge group fed by `sources`.
pub(super) fn tid(rule: Rule, anchor: u8, sources: &[AggregateIdentity]) -> AggregateIdentity {
    AggregateIdentity::from_group_transition(scheme(rule).id(), name(anchor), rule, sources)
}

/// Every creation order of `names`, each in a distinct allocation world.
pub(super) fn worlds(names: &[u8]) -> Vec<World> {
    let mut orders = vec![names.to_vec()];
    let mut reversed = names.to_vec();
    reversed.reverse();
    orders.push(reversed);
    let mut rotated = names.to_vec();
    rotated.rotate_left(names.len() / 2);
    orders.push(rotated);
    let mut interleaved: Vec<u8> = names.iter().copied().step_by(2).collect();
    interleaved.extend(names.iter().copied().skip(1).step_by(2));
    orders.push(interleaved);
    orders
        .iter()
        .enumerate()
        .map(|(at, order)| World::new(order, 17 * at as u64, at as u32 + 1, at as u32 * 3 + 1))
        .collect()
}

#[test]
fn group_name_hashes_only_scheme_anchor_and_rule_under_v2_domain() {
    let scheme_id = AggregateGroupSchemeId::new("cad.loop").unwrap();
    let base = AggregateIdentity::from_group_name(&scheme_id, name(4), Rule::Anchor);
    let mut expected = sha2::Sha256::new();
    use sha2::Digest;
    expected.update(b"worth.relational.group-aggregate-identity.v2\0");
    expected.update(8u64.to_be_bytes());
    expected.update(b"cad.loop");
    expected.update(name(4).as_bytes());
    expected.update([1u8]);
    let expected: [u8; 32] = expected.finalize().into();
    assert_eq!(base.bytes(), &expected);
    let other_scheme = AggregateGroupSchemeId::new("cad.shell").unwrap();
    for distinct in [
        AggregateIdentity::from_group_name(&other_scheme, name(4), Rule::Anchor),
        AggregateIdentity::from_group_name(&scheme_id, name(5), Rule::Anchor),
        AggregateIdentity::from_group_name(&scheme_id, name(4), Rule::Boundary),
        AggregateIdentity::from_group_name(&scheme_id, name(4), Rule::ProducerRole),
    ] {
        assert_ne!(distinct, base);
    }
    let legacy = super::super::mint(
        worth_foundational::facade::AggregateDefinitionIdentity::from_bytes([0; 32]),
        crate::contracts::history::data::coordinates::CommitId(1),
        EntityId::new(PartitionId::new(0), 4, 0),
    );
    assert_ne!(legacy, base, "v1 bytes are never reinterpreted as v2 names");
    let (low, high) = (id(Rule::Anchor, 1), id(Rule::Anchor, 2));
    let (low, high) = (low.min(high), low.max(high));
    let successor = AggregateIdentity::from_group_transition(
        &scheme_id,
        name(4),
        Rule::Anchor,
        &[high, low, high],
    );
    let mut expected = sha2::Sha256::new();
    expected.update(b"worth.relational.group-aggregate-transition.v2\0");
    expected.update(8u64.to_be_bytes());
    expected.update(b"cad.loop");
    expected.update(name(4).as_bytes());
    expected.update([1u8]);
    expected.update(2u64.to_be_bytes());
    expected.update(low.bytes());
    expected.update(high.bytes());
    let expected: [u8; 32] = expected.finalize().into();
    assert_eq!(
        successor.bytes(),
        &expected,
        "sources hash sorted and deduplicated"
    );
    assert_ne!(successor, base);
}

#[test]
fn allocation_commit_and_rebuild_order_leave_names_byte_identical() {
    let names = [9, 3, 7, 1, 5];
    let spec: Spec<'_> = &[(9, Role::Ordinary), (3, Role::Ordinary), (7, Role::Anchor)];
    let other: Spec<'_> = &[(1, Role::Ordinary), (5, Role::Ordinary)];
    let reference = project(
        &worlds(&names)[0]
            .run(Rule::Anchor, &[], &BTreeSet::new(), &[spec, other])
            .unwrap(),
    );
    assert_eq!(reference.0[0].0.bytes().len(), 32);
    for world in worlds(&names) {
        for order in [[spec, other], [other, spec]] {
            let next = world
                .run(Rule::Anchor, &[], &BTreeSet::new(), &order)
                .unwrap();
            assert_eq!(project(&next), reference);
        }
    }
    let identities: BTreeSet<_> = reference.0.iter().map(|group| group.0).collect();
    assert_eq!(
        identities,
        BTreeSet::from([id(Rule::Anchor, 7), id(Rule::Anchor, 1)])
    );
}

#[test]
fn retired_group_revives_its_exact_name_around_the_same_anchor() {
    for world in worlds(&[6, 2, 8, 4]) {
        let rule = Rule::Anchor;
        let group: Spec<'_> = &[(6, Role::Anchor), (2, Role::Ordinary)];
        let prior = world.born(rule, &[group]);
        let original = prior[0].identity;
        let dissolved = world.run(rule, &prior, &BTreeSet::new(), &[]).unwrap();
        assert_eq!(dissolved.retired, BTreeSet::from([original]));
        assert_eq!(
            dissolved.lineage,
            BTreeSet::from([Event::Retired { group: original }])
        );
        let reformed: Spec<'_> = &[(8, Role::Ordinary), (6, Role::Anchor), (4, Role::Ordinary)];
        let revived = world
            .run(rule, &[], &dissolved.retired, &[reformed])
            .unwrap();
        assert_eq!(revived.assignments[0].identity, original);
        assert_eq!(revived.assignments[0].identity.bytes(), original.bytes());
        assert_eq!(revived.assignments[0].origin, AggregateGroupOrigin::Revived);
        assert_eq!(
            revived.lineage,
            BTreeSet::from([Event::Revived { group: original }])
        );
    }
}
