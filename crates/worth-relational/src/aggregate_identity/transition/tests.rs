use super::*;
use crate::identity::data::PartitionId;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

fn member(slot: u64) -> EntityId {
    EntityId::new(PartitionId::new(3), slot, 1)
}
fn set(slots: &[u64]) -> BTreeSet<EntityId> {
    slots.iter().copied().map(member).collect()
}
fn definition() -> AggregateDefinitionIdentity {
    AggregateDefinitionIdentity::from_bytes([7; 32])
}
fn run(prior: &[PriorAggregate], groups: Vec<BTreeSet<EntityId>>, at: u64) -> AggregateTransition {
    transition(definition(), CommitId(at), prior, groups, &mut |_| {
        Ok::<_, AggregateTransitionDenial>(())
    })
    .unwrap()
}
fn prior(groups: &[&[u64]]) -> Vec<PriorAggregate> {
    run(&[], groups.iter().map(|group| set(group)).collect(), 1)
        .assignments
        .into_iter()
        .map(|assignment| PriorAggregate {
            identity: assignment.identity,
            members: assignment.members,
        })
        .collect()
}

#[test]
fn anchor_removal_and_whole_growth_preserve_birth_identity() {
    let old = prior(&[&[4, 5, 6]]);
    let next = run(&old, vec![set(&[1, 5, 6])], 2);
    assert_eq!(next.assignments[0].identity, old[0].identity);
    assert!(!next.assignments[0].born);
    assert!(next.lineage.is_empty());
    assert!(next.retired.is_empty());
    assert_ne!(mint(definition(), CommitId(2), member(1)), old[0].identity);
}

#[test]
fn winning_nonkeeper_mints_final_identity_and_retires_losing_keeper() {
    let old = prior(&[&[1, 2], &[3, 4]]);
    let next = run(&old, vec![set(&[1]), set(&[2, 3, 4])], 2);
    let split = next.assignments[1].identity;
    assert_eq!(next.assignments[0].identity, old[0].identity);
    assert_eq!(split, name(definition(), 2, member(2)));
    assert!(next.retired.contains(&old[1].identity));
    assert_eq!(
        next.lineage,
        BTreeSet::from([
            AggregateLineageEvent {
                occurrence: CommitId(2),
                kind: AggregateLineageKind::SplitFrom,
                from: split,
                to: old[0].identity
            },
            AggregateLineageEvent {
                occurrence: CommitId(2),
                kind: AggregateLineageKind::MergedInto,
                from: old[1].identity,
                to: split
            },
        ])
    );
}

#[test]
fn reciprocal_split_events_do_not_retire_current_identities() {
    let old = prior(&[&[1, 3], &[2, 4]]);
    let next = run(&old, vec![set(&[1, 4]), set(&[2, 3])], 2);
    assert_eq!(
        next.assignments
            .iter()
            .map(|group| group.identity)
            .collect::<BTreeSet<_>>(),
        old.iter().map(|group| group.identity).collect()
    );
    assert!(next.retired.is_empty());
    assert_eq!(next.lineage.len(), 2);
    assert!(next
        .lineage
        .iter()
        .all(|event| event.kind == AggregateLineageKind::SplitFrom));
    let gone = run(&old, Vec::new(), 3);
    assert_eq!(gone.retired.len(), 2);
    assert!(gone.lineage.is_empty());
}

#[test]
fn independent_fragment_model_agrees_under_seeded_permutation_rewires() {
    for seed in 1..=24u64 {
        let mut random = seed;
        let mut permutation: Vec<usize> = (0..12).collect();
        let mut old = Vec::new();
        for at in 1..=64 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            let left = (random as usize) % permutation.len();
            let right = ((random >> 32) as usize) % permutation.len();
            permutation.swap(left, right);
            let groups = cycles(&permutation);
            let expected = model(&old, &groups, at);
            let actual = run(&old, groups.clone(), at);
            assert_eq!(
                actual, expected,
                "seed={seed}, occurrence={at}, permutation={permutation:?}"
            );
            let mut reversed = old.clone();
            reversed.reverse();
            let mut reverse_groups = groups;
            reverse_groups.reverse();
            assert_eq!(run(&reversed, reverse_groups, at), actual);
            old = actual
                .assignments
                .into_iter()
                .map(|group| PriorAggregate {
                    identity: group.identity,
                    members: group.members,
                })
                .collect();
        }
    }
}

// Independent successor-array cycle decomposition; the production owner uses BFS.
fn cycles(permutation: &[usize]) -> Vec<BTreeSet<EntityId>> {
    let mut unvisited: BTreeSet<_> = (0..permutation.len()).collect();
    let mut groups = Vec::new();
    while let Some(&first) = unvisited.first() {
        let mut group = BTreeSet::new();
        let mut cursor = first;
        loop {
            assert!(unvisited.remove(&cursor));
            group.insert(member(cursor as u64));
            cursor = permutation[cursor];
            if cursor == first {
                break;
            }
        }
        groups.push(group);
    }
    groups
}

// Explicit old fragments are labelled per member and mark their unique heir.
fn model(old: &[PriorAggregate], groups: &[BTreeSet<EntityId>], at: u64) -> AggregateTransition {
    let mut labelled = BTreeMap::new();
    for group in old {
        let heir = group
            .members
            .iter()
            .find(|member| groups.iter().any(|group| group.contains(member)))
            .copied();
        for &member in &group.members {
            labelled.insert(member, (group.identity, Some(member) == heir));
        }
    }
    let mut result = AggregateTransition {
        assignments: Vec::new(),
        retired: BTreeSet::new(),
        lineage: BTreeSet::new(),
    };
    for group in groups {
        let fragments: Vec<_> = group
            .iter()
            .filter_map(|member| labelled.get(member).map(|label| (*member, *label)))
            .collect();
        let retained = fragments
            .first()
            .filter(|(_, (_, heir))| *heir)
            .map(|(_, (identity, _))| *identity);
        let identity = retained.unwrap_or_else(|| name(definition(), at, *group.first().unwrap()));
        let mut contributions: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for (member, (old, heir)) in fragments {
            contributions.entry(old).or_default().push((member, heir));
        }
        for (old, fragment) in contributions {
            if !fragment.iter().any(|(_, heir)| *heir) {
                result.lineage.insert(AggregateLineageEvent {
                    occurrence: CommitId(at),
                    kind: AggregateLineageKind::SplitFrom,
                    from: identity,
                    to: old,
                });
            }
        }
        result.assignments.push(AggregateAssignment {
            identity,
            members: group.clone(),
            born: retained.is_none(),
        });
    }
    result
        .assignments
        .sort_by_key(|group| *group.members.first().unwrap());
    for group in old {
        if result
            .assignments
            .iter()
            .any(|new| new.identity == group.identity)
        {
            continue;
        }
        result.retired.insert(group.identity);
        if let Some(heir) = labelled
            .iter()
            .find(|(_, (identity, heir))| *identity == group.identity && *heir)
            .map(|(member, _)| member)
        {
            let successor = result
                .assignments
                .iter()
                .find(|group| group.members.contains(heir))
                .unwrap()
                .identity;
            result.lineage.insert(AggregateLineageEvent {
                occurrence: CommitId(at),
                kind: AggregateLineageKind::MergedInto,
                from: group.identity,
                to: successor,
            });
        }
    }
    result
}

fn name(definition: AggregateDefinitionIdentity, at: u64, member: EntityId) -> AggregateIdentity {
    let transcript = [
        b"WORTH_GRAPH_AGGREGATE_ID_V1\0".as_slice(),
        definition.bytes(),
        &at.to_be_bytes(),
        &member.partition_value_u64().to_be_bytes(),
        &member.local_slot_value().to_be_bytes(),
        &member.generation_value().to_be_bytes(),
    ]
    .concat();
    AggregateIdentity::from_bytes(Sha256::digest(transcript).into())
}

#[test]
fn namespace_occurrence_and_checked_control_are_real_inputs() {
    let seed = member(9);
    assert_ne!(
        mint(definition(), CommitId(2), seed),
        mint(
            AggregateDefinitionIdentity::from_bytes([8; 32]),
            CommitId(2),
            seed
        )
    );
    assert_ne!(
        mint(definition(), CommitId(2), seed),
        mint(definition(), CommitId(3), seed)
    );
    let mut visits = 0;
    let result = transition(
        definition(),
        CommitId(2),
        &[],
        vec![set(&[1, 2])],
        &mut |_| {
            visits += 1;
            Err(AggregateTransitionDenial::MintCollision)
        },
    );
    assert_eq!(visits, 1);
    assert_eq!(result, Err(AggregateTransitionDenial::MintCollision));
}

#[test]
fn birth_mint_matches_an_independently_encoded_big_endian_vector() {
    // Python hashlib + struct.pack(">QQQI", 17, 3, 9, 1), domain and [7; 32].
    assert_eq!(
        *mint(definition(), CommitId(17), member(9)).bytes(),
        [
            239, 15, 20, 189, 192, 177, 43, 64, 128, 130, 233, 52, 176, 75, 131, 162, 191, 49, 137,
            20, 29, 130, 214, 31, 132, 117, 88, 180, 97, 252, 62, 219
        ]
    );
}
