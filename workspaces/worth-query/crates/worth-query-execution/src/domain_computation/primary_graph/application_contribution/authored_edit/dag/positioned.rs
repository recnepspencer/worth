//! Positioned authoring waits only for the roots available before that position.
use super::*;

pub(super) fn barriers<S: worth_query_installation::facade::ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    stages: &[super::super::CapturedAuthoredEditStage<S>],
    nodes: &[Node<'_>],
) -> Result<Vec<Vec<String>>, DeclaredEditOrderRefusal> {
    let family = |i: usize| match &nodes[i].subject {
        DeclaredEditParticipant::Producer { family, .. } => family.as_str(),
        _ => unreachable!("producer node"),
    };
    let declared = |reader: usize, writer: usize| {
        stages
            .iter()
            .flat_map(|s| &s.producer_connections)
            .any(|(parent, child)| {
                let kind = |kind: &std::any::TypeId| {
                    stages
                        .iter()
                        .flat_map(|s| &s.producers)
                        .find(|p| p.family_type == *kind)
                        .map(|p| p.family)
                };
                kind(parent) == Some(family(writer)) && kind(child) == Some(family(reader))
            })
    };
    close(
        Closure {
            stages: stages.len(),
            nodes: nodes.len(),
            family: &family,
            available: &|position| {
                stages[..position]
                    .iter()
                    .flat_map(|s| &s.producers)
                    .map(|p| p.family)
                    .collect()
            },
            overlaps: &|reader, writer| {
                overlap(&nodes[reader], &nodes[writer], &stages[0].schema).is_some()
            },
            declared: &declared,
        },
        || checkpoint(execution),
    )
}

/// The declared graph over node indices: stages first, then producers.
struct Closure<'a, 'f> {
    stages: usize,
    nodes: usize,
    family: &'a dyn Fn(usize) -> &'f str,
    /// The producer families that run before the stage at this position.
    available: &'a dyn Fn(usize) -> Vec<&'f str>,
    /// `reader` declares a read that `writer` may write.
    overlaps: &'a dyn Fn(usize, usize) -> bool,
    /// An installed producer connection feeds `reader` from `writer`.
    declared: &'a dyn Fn(usize, usize) -> bool,
}

/// Each stage waits for the available producers it reads, and for every
/// available producer those need in turn. The closure follows only producers
/// available before the position: a later producer does not run before the
/// stage, so its own inputs are not the stage's prerequisites. With no pending
/// source a read remains an ordinary current read; a future reference is not
/// supplied, and the real projected query refuses it. Later invalidation is
/// still a final-support failure.
fn close<E>(
    graph: Closure<'_, '_>,
    mut checkpoint: impl FnMut() -> Result<(), E>,
) -> Result<Vec<Vec<String>>, E> {
    let mut barriers = Vec::new();
    for position in 0..graph.stages {
        let available = (graph.available)(position);
        let runs_before = |producer: usize| available.contains(&(graph.family)(producer));
        let mut required = Vec::new();
        for producer in graph.stages..graph.nodes {
            checkpoint()?;
            if (graph.overlaps)(position, producer) {
                required.push(producer);
            }
        }
        let mut cursor = 0;
        while cursor < required.len() {
            let reader = required[cursor];
            cursor += 1;
            if !runs_before(reader) {
                continue;
            }
            for writer in graph.stages..graph.nodes {
                checkpoint()?;
                if writer == reader || required.contains(&writer) {
                    continue;
                }
                if (graph.declared)(reader, writer) || (graph.overlaps)(reader, writer) {
                    required.push(writer);
                }
            }
        }
        let barrier = required
            .into_iter()
            .filter(|producer| runs_before(*producer))
            .map(|producer| (graph.family)(producer).to_owned())
            .collect();
        barriers.push(barrier);
    }
    Ok(barriers)
}

#[cfg(test)]
mod tests;
