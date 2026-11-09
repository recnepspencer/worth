//! Public checked component edits: independent graph truth and growth classes.
use std::collections::{BTreeMap, BTreeSet};
use worth_execution::{
    CancellationToken, ComponentEdit, ComponentPartitioner, ExecutionRequest, ExecutionWorkCeiling,
    PartitionItemId, SerialMemoryBudget, SerialRequest,
};
use worth_foundational::PartitionIdentity;
fn item(id: u64) -> PartitionItemId {
    PartitionItemId(id)
}
fn request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    )
}
fn chain(first: u64, count: u64) -> ComponentPartitioner {
    let mut graph = ComponentPartitioner::new();
    for id in first..first + count {
        graph.upsert_item(item(id));
        if id > first {
            graph.add_edge(item(id - 1), item(id)).unwrap();
        }
    }
    graph
}
fn fixed_edit(count: u64, case: u8) -> u64 {
    let mut graph = chain(if case == 1 { 2 } else { 1 }, count);
    let edit = match case {
        0 => ComponentEdit::RemoveEdge(item(count - 1), item(count)),
        1 => {
            graph.upsert_item(item(1));
            ComponentEdit::AddEdge(item(1), item(2))
        }
        2 => {
            graph.add_edge(item(1), item(count)).unwrap();
            ComponentEdit::RemoveEdge(item(1), item(count))
        }
        _ => ComponentEdit::RemoveItem(item(1)),
    };
    let original = request();
    let execution = ExecutionRequest::serial(&original);
    let (outcome, report) = execution
        .run(ExecutionWorkCeiling::new(None), |_| {
            graph.edit_with_request(&execution, edit)
        })
        .unwrap();
    outcome.unwrap();
    let (member, expected) = match case {
        0 => (count, count),
        1 => (count + 1, 1),
        2 => (count, 1),
        _ => (count, 2),
    };
    assert_eq!(
        graph.route(item(member)),
        Some(PartitionIdentity::new(expected))
    );
    report.charged_work()
}
#[test]
fn fixed_component_edits_do_not_visit_the_growing_unchanged_island() {
    for case in 0..4 {
        let small = fixed_edit(256, case);
        let large = fixed_edit(512, case);
        assert!(
            large as f64 <= small.max(1) as f64 * 1.5,
            "case={case}: unchanged component doubled, visits {small} -> {large}"
        );
    }
}
fn oracle(adjacent: &BTreeMap<u64, BTreeSet<u64>>) -> BTreeMap<u64, u64> {
    let mut result = BTreeMap::new();
    for &seed in adjacent.keys() {
        if result.contains_key(&seed) {
            continue;
        }
        let mut pending = vec![seed];
        let mut found = BTreeSet::new();
        while let Some(member) = pending.pop() {
            if found.insert(member) {
                pending.extend(adjacent[&member].iter().copied());
            }
        }
        let least = *found.first().unwrap();
        result.extend(found.into_iter().map(|member| (member, least)));
    }
    result
}
#[test]
fn forest_cuts_equal_cold_connectivity_through_removal_reconnection_and_identity_reuse() {
    for seed in 0..24u64 {
        let mut random = seed + 1;
        let mut graph = ComponentPartitioner::new();
        let mut adjacent = BTreeMap::<u64, BTreeSet<u64>>::new();
        let original = request();
        let execution = ExecutionRequest::serial(&original);
        for _ in 0..512 {
            random = random
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let a = (random >> 32) % 24;
            let b = (random >> 40) % 24;
            let edit = match random % 4 {
                0 => {
                    adjacent.entry(a).or_default();
                    ComponentEdit::UpsertItem(item(a))
                }
                1 => {
                    if let Some(neighbors) = adjacent.remove(&a) {
                        for neighbor in neighbors {
                            adjacent.get_mut(&neighbor).unwrap().remove(&a);
                        }
                    }
                    ComponentEdit::RemoveItem(item(a))
                }
                2 if a != b && adjacent.contains_key(&a) && adjacent.contains_key(&b) => {
                    adjacent.get_mut(&a).unwrap().insert(b);
                    adjacent.get_mut(&b).unwrap().insert(a);
                    ComponentEdit::AddEdge(item(a), item(b))
                }
                _ if a != b && adjacent.contains_key(&a) && adjacent.contains_key(&b) => {
                    adjacent.get_mut(&a).unwrap().remove(&b);
                    adjacent.get_mut(&b).unwrap().remove(&a);
                    ComponentEdit::RemoveEdge(item(a), item(b))
                }
                _ => continue,
            };
            graph.edit_with_request(&execution, edit).unwrap();
            let truth = oracle(&adjacent);
            for id in 0..24 {
                assert_eq!(
                    graph.route(item(id)),
                    truth.get(&id).map(|&least| PartitionIdentity::new(least)),
                    "seed={seed} edit={edit:?}"
                );
            }
            for &least in truth.values() {
                let expected = truth
                    .iter()
                    .filter_map(|(&id, &group)| (group == least).then_some(item(id)))
                    .collect::<BTreeSet<_>>();
                assert_eq!(
                    graph.members(PartitionIdentity::new(least)),
                    Some(&expected)
                );
            }
        }
    }
}
