//! Positioned authoring waits only for the roots available before that position.
use super::*;

pub(super) fn barriers<S: worth_query_installation::facade::ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    stages: &[super::super::CapturedAuthoredEditStage<S>],
    nodes: &[Node<'_>],
) -> Result<Vec<Vec<String>>, DeclaredEditOrderRefusal> {
    let count = stages.len();
    let producer_family = |i: usize| match &nodes[i].subject {
        DeclaredEditParticipant::Producer { family, .. } => family,
        _ => unreachable!("producer node"),
    };
    let mut barriers = Vec::new();
    for position in 0..count {
        let mut required = Vec::new();
        for producer in count..nodes.len() {
            checkpoint(execution)?;
            if let Some(dependency) = overlap(&nodes[position], &nodes[producer], &stages[0].schema)
            {
                if position == 0 {
                    return Err(DeclaredEditOrderRefusal::StageAfterProducer(dependency));
                }
                required.push(producer);
            }
        }
        // Close the installed producer requirements, without invoking a supplier.
        let mut cursor = 0;
        while cursor < required.len() {
            let reader = required[cursor];
            for writer in count..nodes.len() {
                checkpoint(execution)?;
                if writer == reader || required.contains(&writer) {
                    continue;
                }
                let declared =
                    stages
                        .iter()
                        .flat_map(|s| &s.producer_connections)
                        .any(|(parent, child)| {
                            let family = |kind: &std::any::TypeId| {
                                stages
                                    .iter()
                                    .flat_map(|s| &s.producers)
                                    .find(|p| p.family_type == *kind)
                                    .map(|p| p.family)
                            };
                            family(parent) == Some(producer_family(writer).as_str())
                                && family(child) == Some(producer_family(reader).as_str())
                        });
                if declared || overlap(&nodes[reader], &nodes[writer], &stages[0].schema).is_some()
                {
                    required.push(writer);
                }
            }
            cursor += 1;
        }
        let available = stages[..position]
            .iter()
            .flat_map(|s| &s.producers)
            .map(|p| p.family)
            .collect::<Vec<_>>();
        let mut barrier = Vec::new();
        for producer in required {
            let family = producer_family(producer);
            if available.iter().any(|prior| *prior == family) {
                barrier.push(family.clone());
            }
            // With no pending source this remains an ordinary current read.
            // A future D reference is not supplied: the real projected query
            // refuses it. Later invalidation is still a final-support failure.
        }
        barriers.push(barrier);
    }
    Ok(barriers)
}
