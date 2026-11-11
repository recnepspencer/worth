//! Installed declarations decide order; source IDs never prove independence.
use super::*;
mod cycle;
use worth_query_installation::facade::{
    WorthQueryOperationGraphReadScope, WorthQueryOperationReadTouchOverlapIndex,
    WorthQueryOperationTouchScope,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredEditStage {
    pub identity: ApplicationEditStageIdentity,
    pub binding: &'static str,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclaredEditParticipant {
    Stage(DeclaredEditStage),
    Producer {
        family: String,
        implementations: Vec<String>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeclaredEditReadScope {
    Graph(WorthQueryOperationGraphReadScope),
    Query(DeclaredQueryRead),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredEditDependency {
    pub writer: DeclaredEditParticipant,
    pub reader: DeclaredEditParticipant,
    pub read_scope: DeclaredEditReadScope,
    pub write_scope: WorthQueryOperationTouchScope,
}
/// A declaration cycle is distinct from Native's actual record conflict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredEditScopeCycle {
    pub stages: Vec<DeclaredEditStage>,
    pub overlaps: Vec<DeclaredEditDependency>,
}
#[derive(Debug)]
pub enum DeclaredEditOrderRefusal {
    Cycle(DeclaredEditScopeCycle),
    Execution(WorthQueryApplicationAttemptDenial),
    /// Authored stages after a producer wave require phase 2.3's qualification.
    StageAfterProducer(DeclaredEditDependency),
}
struct Node<'a> {
    subject: DeclaredEditParticipant,
    contracts: Vec<&'a WorthQueryOperationReadTouchOverlapIndex>,
    queries: Vec<&'a DeclaredQueryRead>,
}
fn overlap(
    reader: &Node<'_>,
    writer: &Node<'_>,
    schema: &worth_query_declaration::facade::application_schema::ApplicationSchemaBindingIdentity,
) -> Option<DeclaredEditDependency> {
    for write in writer
        .contracts
        .iter()
        .flat_map(|contract| contract.touches())
    {
        for contract in &reader.contracts {
            for read in contract.reads() {
                if contract.may_intersect(read, write) {
                    return Some(DeclaredEditDependency {
                        writer: writer.subject.clone(),
                        reader: reader.subject.clone(),
                        read_scope: DeclaredEditReadScope::Graph(read.clone()),
                        write_scope: write.clone(),
                    });
                }
            }
        }
        for query in &reader.queries {
            if reader.contracts[0].query_intersects(schema, &query.contract, write) {
                return Some(DeclaredEditDependency {
                    writer: writer.subject.clone(),
                    reader: reader.subject.clone(),
                    read_scope: DeclaredEditReadScope::Query((*query).clone()),
                    write_scope: write.clone(),
                });
            }
        }
    }
    None
}
pub fn order_authored_stages<S>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    stages: &[CapturedAuthoredEditStage<S>],
) -> Result<Vec<usize>, DeclaredEditOrderRefusal> {
    let Some(first) = stages.first() else {
        return Ok(Vec::new());
    };
    let mut nodes = stages
        .iter()
        .map(|stage| Node {
            subject: DeclaredEditParticipant::Stage(DeclaredEditStage {
                identity: stage.identity,
                binding: stage.binding,
            }),
            contracts: vec![&stage.contracts],
            queries: stage.queries.iter().collect(),
        })
        .collect::<Vec<_>>();
    for producer in stages.iter().flat_map(|stage| &stage.producers) {
        checkpoint(execution)?;
        if let Some(node) = nodes.iter_mut().find(|node| matches!(&node.subject, DeclaredEditParticipant::Producer { family, .. } if family == producer.family)) {
            if let DeclaredEditParticipant::Producer { implementations, .. } = &mut node.subject
                && !implementations.contains(&producer.identity) {
                implementations.push(producer.identity.clone()); node.contracts.push(&producer.contract); node.queries.extend(producer.queries.iter());
            }
        } else {
            nodes.push(Node { subject: DeclaredEditParticipant::Producer { family: producer.family.into(), implementations: vec![producer.identity.clone()] }, contracts: vec![&producer.contract], queries: producer.queries.iter().collect() });
        }
    }
    let mut edges = Vec::new();
    let mut incoming = vec![0usize; nodes.len()];
    for (writer, writes) in nodes.iter().enumerate() {
        for (reader, reads) in nodes.iter().enumerate() {
            checkpoint(execution)?;
            if writer == reader {
                continue;
            }
            if let Some(dependency) = overlap(reads, writes, &first.schema) {
                edges.push((writer, reader, dependency));
                incoming[reader] += 1;
            }
        }
    }
    let mut order = Vec::new();
    while let Some(next) = incoming.iter().enumerate().find_map(|(ordinal, incoming)| {
        (*incoming == 0 && !order.contains(&ordinal)).then_some(ordinal)
    }) {
        checkpoint(execution)?;
        order.push(next);
        for (_, reader, _) in edges.iter().filter(|(writer, _, _)| *writer == next) {
            incoming[*reader] -= 1;
        }
    }
    if order.len() != nodes.len() {
        let cycle = cycle::find(nodes.len(), &edges);
        let overlaps = cycle
            .iter()
            .zip(cycle.iter().cycle().skip(1))
            .map(|(writer, reader)| {
                edges
                    .iter()
                    .find(|(from, to, _)| from == writer && to == reader)
                    .expect("cycle follows actual declared edges")
                    .2
                    .clone()
            })
            .collect();
        return Err(DeclaredEditOrderRefusal::Cycle(DeclaredEditScopeCycle {
            stages: cycle
                .iter()
                .filter_map(|ordinal| match &nodes[*ordinal].subject {
                    DeclaredEditParticipant::Stage(stage) => Some(stage.clone()),
                    _ => None,
                })
                .collect(),
            overlaps,
        }));
    }
    if let Some((_, _, dependency)) = edges
        .into_iter()
        .find(|(writer, reader, _)| *writer >= stages.len() && *reader < stages.len())
    {
        return Err(DeclaredEditOrderRefusal::StageAfterProducer(dependency));
    }
    Ok(order
        .into_iter()
        .filter(|ordinal| *ordinal < stages.len())
        .collect())
}

fn checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), DeclaredEditOrderRefusal> {
    let refused = |cause: worth_execution::WorkCeilingDenial| {
        DeclaredEditOrderRefusal::Execution(
            WorthQueryApplicationAttemptDenial::relational_read_stopped(
                cause.into(),
                "edit envelope dependency admission",
            ),
        )
    };
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)
        })
        .map_err(refused)?
        .0
        .map_err(|cause| refused(worth_execution::WorkCeilingDenial::Stopped(cause)))
}
