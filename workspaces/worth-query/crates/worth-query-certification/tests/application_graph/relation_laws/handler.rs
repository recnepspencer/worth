//! Decision authority, then native link authoring; no application law evaluator.
use super::{operation::*, schema::*};
use std::collections::BTreeSet;
use worth_query_host::facade::primary_graph::*;

pub struct Handler;
impl<L: LawPolicy> OperationHandler<GraphSchema<L>, EditBinding<L>> for Handler {
    fn decide(
        &self,
        input: &EdgeEdit,
        reader: &mut DecisionReader<'_, '_, '_, GraphSchema<L>, EditBinding<L>>,
    ) -> HandlerResult<Targets<L>, GraphDenial> {
        let mut targets = Targets {
            added: Vec::new(),
            removed: Vec::new(),
        };
        let mut removal_commands = BTreeSet::new();
        for (remove, edges) in [(false, &input.edges), (true, &input.removals)] {
            for &(source, target) in edges {
                if remove && !removal_commands.insert((source, target)) {
                    continue;
                }
                let source = match reader.resolve_entity(NodeKey::reference(), source) {
                    Ok(value) => value,
                    Err(error) => return HandlerResult::ExecutionDenied(error),
                };
                let target = match reader.resolve_entity(NodeKey::reference(), target) {
                    Ok(value) => value,
                    Err(error) => return HandlerResult::ExecutionDenied(error),
                };
                if remove {
                    match reader.relations_from(Precedes::reference(), &source) {
                        Ok(edges) if edges.iter().any(|edge| edge.to() == &target) => {}
                        Ok(_) => continue,
                        Err(error) => {
                            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(
                                error,
                            ))
                        }
                    }
                }
                let source = match reader.mutation_target(&source) {
                    Ok(value) => value,
                    Err(error) => return HandlerResult::ExecutionDenied(error),
                };
                let target = match reader.mutation_target(&target) {
                    Ok(value) => value,
                    Err(error) => return HandlerResult::ExecutionDenied(error),
                };
                if remove {
                    targets.removed.push((source, target));
                } else {
                    targets.added.push((source, target));
                }
            }
        }
        HandlerResult::Completed(targets)
    }
    fn build_candidate(
        &self,
        input: &EdgeEdit,
        targets: Targets<L>,
        writer: &mut CandidateWriter<'_, GraphSchema<L>, EditBinding<L>>,
    ) -> HandlerResult<Edited, GraphDenial> {
        match write_edges(input, targets, writer) {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

fn write_edges<L: LawPolicy>(
    input: &EdgeEdit,
    targets: Targets<L>,
    writer: &mut CandidateWriter<'_, GraphSchema<L>, EditBinding<L>>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    for (remove, edges) in [(true, targets.removed), (false, targets.added)] {
        for (index, (source, target)) in edges.into_iter().enumerate() {
            let source = writer.projected_entity(&source)?;
            let target = writer.projected_entity(&target)?;
            if remove {
                writer.unlink(Precedes::reference(), &source, &target)?;
            } else {
                writer.link(
                    Precedes::reference(),
                    format!("edge-{}-{index}", input.serial),
                    &source,
                    &target,
                )?;
            }
        }
    }
    Ok(())
}
