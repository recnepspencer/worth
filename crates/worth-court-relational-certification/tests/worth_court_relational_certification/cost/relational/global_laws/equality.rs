//! Oracle uses a separate edge list and exhaustive graph traversals, never SDK adjacency.
mod mixed;
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    inspection::{observe_relation_law_sources, RelationLawSourceObservation},
    transactions::{EntityReference, RecordRef, TransactionCommitError, WorkerIntentBatch},
};
use worth_testkit_relational::maintenance::relation_laws::{request, World};

#[test]
fn deleting_transit_edge_refuses_its_indirect_source_and_preserves_head() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let world = World::new(&execution, 2);
    let version = world
        .runtime
        .branch_reference_state(&worth_relational::facade::history::BranchId("main".into()))
        .unwrap();
    let (result, observation) = observe_relation_law_sources(1024, || {
        world.commit(
            &execution,
            WorkerIntentBatch::new("disconnect-transit").push(world.delete(1)),
        )
    });
    let after: Vec<_> = world
        .edges
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != 1)
        .map(|(_, (_, s, t))| (*s, *t))
        .collect();
    let before: Vec<_> = world.edges.iter().map(|(_, s, t)| (*s, *t)).collect();
    check_witness(&world.nodes, &before, &after, &[1, 2], &observation);
    assert_connectivity_refusal(result.unwrap_err());
    assert_eq!(
        world
            .runtime
            .branch_reference_state(&worth_relational::facade::history::BranchId("main".into()))
            .unwrap(),
        version
    );
}

#[test]
fn maintained_sources_equal_cold_seeds_1_to_4() {
    random_edits(1..=4);
}
#[test]
fn maintained_sources_equal_cold_seeds_5_to_8() {
    random_edits(5..=8);
}
#[test]
fn maintained_sources_equal_cold_seeds_9_to_12() {
    random_edits(9..=12);
}
#[test]
fn maintained_sources_equal_cold_seeds_13_to_16() {
    random_edits(13..=16);
}
fn random_edits(seeds: std::ops::RangeInclusive<u64>) {
    mixed::random_edits(seeds.clone());
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let mut retirements = 0;
    let mut restorations = 0;
    for seed in seeds {
        let mut world = World::new(&execution, 4);
        let mut rng: u64 = seed;
        let mut retired_pairs = Vec::new();
        for step in 0..96 {
            let before: Vec<_> = world.edges.iter().map(|(_, s, t)| (*s, *t)).collect();
            let mut after = before.clone();
            let mut next = || {
                rng = rng.wrapping_mul(6364136223846793005u64).wrapping_add(1);
                (rng >> 32) as usize
            };
            let action = next() % 4;
            let remove = action == 0;
            let retarget = action == 1;
            let index = next() % world.edges.len().max(1);
            let mut source = next() % world.nodes.len();
            let mut target = next() % world.nodes.len();
            if retarget {
                source = world.edges[index].1;
            }
            if !remove {
                if !retarget
                    && step % 7 == 0
                    && let Some(&(s, t)) = retired_pairs.iter().find(|pair| !before.contains(pair))
                {
                    source = s;
                    target = t;
                }
                // Native structural relation identity is endpoint-based. Draw a
                // legal unused identity so duplicate admission cannot masquerade
                // as the reachability law under test.
                while before
                    .iter()
                    .enumerate()
                    .any(|(i, pair)| !(retarget && i == index) && *pair == (source, target))
                {
                    target = (target + 1) % world.nodes.len();
                    if target == 0 && !retarget {
                        source = (source + 1) % world.nodes.len();
                    }
                }
            }
            let (intent, seeds) = if remove {
                let (_, s, t) = world.edges[index];
                after.remove(index);
                (world.delete(index), vec![s, t])
            } else if retarget {
                let (_, s, t) = world.edges[index];
                after[index] = (s, target);
                (world.retarget(index, target), vec![s, t, target])
            } else {
                after.push((source, target));
                (
                    world.insert(source, target, &format!("random-{seed}-{step}")),
                    vec![source, target],
                )
            };
            let expected = valid(&after, world.nodes.len());
            let version = world
                .runtime
                .branch_reference_state(&worth_relational::facade::history::BranchId("main".into()))
                .unwrap();
            let (result, observation) = observe_relation_law_sources(16_384, || {
                world.commit(
                    &execution,
                    WorkerIntentBatch::new("random-law-edit").push(intent),
                )
            });
            assert!(
                !observation.witnesses.is_empty(),
                "seed {seed} step {step}, {before:?} -> {after:?}: {result:?}"
            );
            check_witness(&world.nodes, &before, &after, &seeds, &observation);
            assert_eq!(
                result.is_ok(),
                expected,
                "seed {seed} step {step}: {before:?} -> {after:?}: {result:?}"
            );
            match result {
                Ok(result) => {
                    if remove {
                        let (_, s, t) = world.edges.remove(index);
                        retired_pairs.push((s, t));
                        retirements += 1;
                    } else if retarget {
                        world.edges[index].2 = target;
                    } else {
                        if retired_pairs.contains(&(source, target)) {
                            restorations += 1;
                        }
                        let relation = result
                            .changed_records
                            .iter()
                            .find_map(|r| match r {
                                RecordRef::Relation(id) => Some(*id),
                                _ => None,
                            })
                            .unwrap();
                        world.edges.push((relation, source, target));
                    }
                    world.release(&result);
                }
                Err(error) => {
                    assert_connectivity_refusal(error);
                    assert_eq!(
                        world
                            .runtime
                            .branch_reference_state(&worth_relational::facade::history::BranchId(
                                "main".into()
                            ))
                            .unwrap(),
                        version
                    );
                }
            }
        }
    }
    assert!(
        retirements > 0 && restorations > 0,
        "the random journeys must exercise both edge retirement and restoration"
    );
}
fn assert_connectivity_refusal(error: TransactionCommitError) {
    match error {
        TransactionCommitError::Publication { error, .. } => {
            assert!(error.detail.contains("source_reaches_target"), "{error:?}")
        }
        other => panic!("a resource/other refusal cannot stand for connectivity: {other:?}"),
    }
}
fn closure(seeds: &[usize], edges: &[(usize, usize)], reverse: bool) -> BTreeSet<usize> {
    let mut seen: BTreeSet<_> = seeds.iter().copied().collect();
    let mut pending = seeds.to_vec();
    while let Some(vertex) = pending.pop() {
        for &(s, t) in edges {
            let (from, to) = if reverse { (t, s) } else { (s, t) };
            if from == vertex && seen.insert(to) {
                pending.push(to);
            }
        }
    }
    seen
}
fn valid(edges: &[(usize, usize)], nodes: usize) -> bool {
    (0..nodes).step_by(4).all(|source| {
        closure(&[source], edges, false)
            .iter()
            .any(|&id| id != source && id % 4 >= 2)
    })
}
fn check_witness(
    nodes: &[EntityId],
    before: &[(usize, usize)],
    after: &[(usize, usize)],
    seeds: &[usize],
    observation: &RelationLawSourceObservation,
) {
    assert_eq!(observation.omitted, 0);
    let actual = observation
        .witnesses
        .iter()
        .find(|w| w.relation_kind == KindId(9))
        .expect("ordinary SDK commit must derive its source witness");
    assert!(
        !actual.all_visible_sources,
        "an ordinary commit must use its affected source witness"
    );
    let both: Vec<_> = before.iter().chain(after).copied().collect();
    let sources = closure(seeds, &both, true);
    let reference = |index: usize| EntityReference::Existing(nodes[index]);
    let expected_sources: BTreeSet<_> = sources.iter().copied().map(reference).collect();
    assert_eq!(
        actual.sources, expected_sources,
        "indirect predecessor source-set differs from cold oracle"
    );
    let reachable = closure(&sources.into_iter().collect::<Vec<_>>(), after, false);
    let expected_successors: BTreeMap<_, _> = reachable
        .into_iter()
        .map(|source| {
            let targets: BTreeSet<_> = after
                .iter()
                .filter_map(|&(s, t)| (s == source).then_some(reference(t)))
                .collect();
            (reference(source), targets.into_iter().collect::<Vec<_>>())
        })
        .collect();
    assert_eq!(
        actual.successors, expected_successors,
        "maintained successor witness differs from cold edge traversal"
    );
}
