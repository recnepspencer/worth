//! One invocation-local DFS over the immutable functional parent graph.
use std::cell::Cell;
use worth_execution::ExecutionRequestArrayBuilder;
use {
    super::ExecutionRequest,
    crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl as Control,
    crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop as Stop,
};

/// The caller supplies every live node as a seed and only actual live parents.
/// This numeric traversal creates no Source, node-identity or root authority.
pub fn acyclic(
    seeds: impl IntoIterator<Item = usize>,
    node_count: usize,
    frontier: usize,
    execution: &ExecutionRequest<'_, '_>,
    control: &Control<'_>,
    mut parent: impl FnMut(usize) -> Result<Option<usize>, Stop>,
) -> Result<(), Stop> {
    let mut walks = ExecutionRequestArrayBuilder::allocate(*execution, frontier)
        .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    for _ in 0..frontier {
        control.checkpoint(1)?;
        walks
            .push(Cell::new(None))
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    }
    let walks = walks
        .seal()
        .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
    let walks = walks.elements();
    let mut visited = 0;
    for seed in seeds {
        if walk(walks, seed, control)?.is_some() {
            continue;
        }
        let mut current = Some(seed);
        while let Some(ordinal) = current {
            match walk(walks, ordinal, control)? {
                Some(active) if active == seed => {
                    return Err(control.refuse(Stop::SourceStructureCycle))
                }
                Some(_) => break,
                None => {
                    if visited >= node_count {
                        return Err(control.refuse(Stop::SourceStructureIncomplete));
                    }
                    walks[ordinal].set(Some(seed));
                    visited += 1;
                    current = parent(ordinal)?;
                }
            }
        }
        // This whole walk succeeded. Its tag now identifies a completed walk;
        // only a later unvisited seed can start another active tag.
    }
    control.poll()
}
fn walk(
    walks: &[Cell<Option<usize>>],
    ordinal: usize,
    control: &Control<'_>,
) -> Result<Option<usize>, Stop> {
    control.checkpoint(1)?;
    walks
        .get(ordinal)
        .map(Cell::get)
        .ok_or_else(|| control.refuse(Stop::SourceStructureIncomplete))
}
