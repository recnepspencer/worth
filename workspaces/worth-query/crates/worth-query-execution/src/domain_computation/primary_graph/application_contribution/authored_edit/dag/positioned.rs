//! Positioned authoring waits only for producers an earlier stage actually left
//! pending. The declared graph is planned once, before any stage runs; each
//! position's barrier is resolved when its stage is reached.
use super::*;

pub(super) fn barriers<S: worth_query_installation::facade::ApplicationSchema>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    stages: &[super::super::CapturedAuthoredEditStage<S>],
    nodes: &[Node<'_>],
) -> Result<Vec<PositionedBarrier>, DeclaredEditOrderRefusal> {
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
    plan(
        Closure {
            stages: stages.len(),
            nodes: nodes.len(),
            family: &family,
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
    /// `reader` declares a read that `writer` may write.
    overlaps: &'a dyn Fn(usize, usize) -> bool,
    /// An installed producer connection feeds `reader` from `writer`.
    declared: &'a dyn Fn(usize, usize) -> bool,
}

/// Every declared producer family, and what each one reads or is fed from.
#[derive(Debug)]
struct ProducerGraph {
    families: Vec<String>,
    /// `needs[reader]`: producers `reader` may read, or is connected from.
    needs: Vec<Vec<usize>>,
}

/// One stage's planned barrier: the producers its own reads may overlap, over
/// the envelope's shared declared graph. Declarations only say what may run;
/// [`PositionedBarrier::resolve`] decides from what actually is pending.
#[derive(Clone, Debug)]
pub struct PositionedBarrier {
    graph: std::sync::Arc<ProducerGraph>,
    direct: Vec<usize>,
}

impl PositionedBarrier {
    /// Each stage waits for the pending producers it reads, and for every
    /// pending producer those need in turn. `pending` holds the families with
    /// a source or root that the earlier stages actually left unprepared; a
    /// family that is only declared has nothing to settle. The closure follows
    /// only pending producers: one that does not run before the stage is not
    /// its prerequisite, and neither are its inputs. With no pending source a
    /// read remains an ordinary current read; a future reference is not
    /// supplied, and the real projected query refuses it. A producer that
    /// runs later and changes what the stage read is a final-support refusal
    /// (`require_read_support`), never a stale result.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn resolve(
        &self,
        pending: &[&str],
    ) -> Vec<String> {
        let graph = &*self.graph;
        let runs_before = |producer: usize| pending.contains(&graph.families[producer].as_str());
        let mut required = self.direct.clone();
        let mut cursor = 0;
        while cursor < required.len() {
            let reader = required[cursor];
            cursor += 1;
            if !runs_before(reader) {
                continue;
            }
            for writer in &graph.needs[reader] {
                if !required.contains(writer) {
                    required.push(*writer);
                }
            }
        }
        required
            .into_iter()
            .filter(|producer| runs_before(*producer))
            .map(|producer| graph.families[producer].clone())
            .collect()
    }
}

/// Pays every declared overlap once, before the first stage runs.
fn plan<E>(
    graph: Closure<'_, '_>,
    mut checkpoint: impl FnMut() -> Result<(), E>,
) -> Result<Vec<PositionedBarrier>, E> {
    let producers = graph.stages..graph.nodes;
    let mut needs = Vec::new();
    for reader in producers.clone() {
        let mut row = Vec::new();
        for writer in producers.clone() {
            checkpoint()?;
            if writer != reader
                && ((graph.declared)(reader, writer) || (graph.overlaps)(reader, writer))
            {
                row.push(writer - graph.stages);
            }
        }
        needs.push(row);
    }
    let shared = std::sync::Arc::new(ProducerGraph {
        families: producers
            .clone()
            .map(|i| (graph.family)(i).to_owned())
            .collect(),
        needs,
    });
    let mut barriers = Vec::new();
    for position in 0..graph.stages {
        let mut direct = Vec::new();
        for producer in producers.clone() {
            checkpoint()?;
            if (graph.overlaps)(position, producer) {
                direct.push(producer - graph.stages);
            }
        }
        barriers.push(PositionedBarrier {
            graph: shared.clone(),
            direct,
        });
    }
    Ok(barriers)
}

#[cfg(test)]
mod tests;
