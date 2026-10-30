// A component partitioner edited one step at a time equals one built fresh
// from the same items and edges after every step, and a step leaves every
// island it does not touch with its identity and its members.

pub(crate) use {std::collections::BTreeMap, std::collections::BTreeSet};

pub(crate) use worth_foundational::PartitionIdentity;
pub(crate) use {worth_execution::ComponentPartitioner, worth_execution::PartitionItemId};

pub(crate) type Islands = BTreeMap<PartitionIdentity, BTreeSet<u64>>;

/// Every island by its identity, read through `route` and checked against
/// `members`.
pub(crate) fn islands(partitioner: &ComponentPartitioner, items: &BTreeSet<u64>) -> Islands {
    let mut islands = crate::components_differential::reference_context::Islands::new();
    for item in items {
        let identity = partitioner
            .route(PartitionItemId(*item))
            .expect("every item routes");
        islands.entry(identity).or_default().insert(*item);
    }
    for (identity, members) in &islands {
        let held = partitioner
            .members(*identity)
            .expect("every routed island has members")
            .iter()
            .map(|item| item.0)
            .collect::<BTreeSet<_>>();
        assert_eq!(&held, members, "members and routes agree");
    }
    islands
}

pub(crate) fn fresh(items: &BTreeSet<u64>, edges: &BTreeSet<(u64, u64)>) -> ComponentPartitioner {
    let mut partitioner = worth_execution::ComponentPartitioner::new();
    for item in items {
        partitioner.upsert_item(PartitionItemId(*item));
    }
    for (a, b) in edges {
        partitioner
            .add_edge(PartitionItemId(*a), PartitionItemId(*b))
            .expect("both ends are items");
    }
    partitioner
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Step {
    AddItem(u64),
    AddEdge(u64, u64),
    RemoveEdge(u64, u64),
    RemoveItem(u64),
}

/// The items whose islands `step` touches, before it runs.
pub(crate) fn touched(step: Step) -> Vec<u64> {
    match step {
        crate::components_differential::reference_context::Step::AddItem(_) => Vec::new(),
        crate::components_differential::reference_context::Step::AddEdge(a, b)
        | crate::components_differential::reference_context::Step::RemoveEdge(a, b) => vec![a, b],
        crate::components_differential::reference_context::Step::RemoveItem(item) => vec![item],
    }
}
